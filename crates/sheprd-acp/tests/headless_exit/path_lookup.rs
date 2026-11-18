// ABOUTME: Exercises real helper startup with private trusted and checkout programs.
// ABOUTME: Prevents relative PATH entries from launching a checkout's custom agent.
use super::*;
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

const PROGRAM: &str = "spaceport-fixture-agent";

fn install(dir: &Path, script: &str) {
    let path = dir.join(PROGRAM);
    fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn a_custom_agent_in_relative_path_entries_never_starts() {
    for relative in [".", "", "checkout-bin"] {
        let sandbox = sandbox::Sandbox::new();
        let nested = sandbox.dir.join("checkout-bin");
        fs::create_dir(&nested).unwrap();
        for dir in [&sandbox.dir, &nested] {
            install(dir, "printf invoked > invoked\nread request\nexit 0");
        }
        let mut command = sandbox.command(env!("CARGO_BIN_EXE_sheprd-acp"), "helper.log");
        command
            .env("PATH", relative)
            .args(["--headless", "--no-attach", "--", PROGRAM]);
        let (status, log) = run_command(&sandbox, command);
        assert!(
            !status.success(),
            "relative program ran: {relative:?}: {log}"
        );
        assert!(!sandbox.dir.join("invoked").exists());
        assert!(log.contains(PROGRAM), "missing lookup diagnostic: {log}");
    }
}

#[test]
fn absolute_path_selection_wins_over_a_checkout_program() {
    let sandbox = sandbox::Sandbox::new();
    let trusted = sandbox.dir.join("trusted");
    fs::create_dir(&trusted).unwrap();
    install(&sandbox.dir, "printf invoked > invoked\nexit 7");
    install(&trusted, "read request\nexit 0");
    let mut command = sandbox.command(env!("CARGO_BIN_EXE_sheprd-acp"), "helper.log");
    command
        .env(
            "PATH",
            std::env::join_paths([Path::new("."), &trusted]).unwrap(),
        )
        .args(["--headless", "--no-attach", "--", PROGRAM]);
    let (status, log) = run_command(&sandbox, command);
    assert!(status.success(), "trusted program failed: {log}");
    assert!(!sandbox.dir.join("invoked").exists());
    assert!(log.is_empty(), "unexpected diagnostic: {log}");
}

#[test]
fn an_explicit_custom_relative_path_still_starts() {
    let sandbox = sandbox::Sandbox::new();
    install(
        &sandbox.dir,
        "printf invoked > invoked\nread request\nexit 0",
    );
    let mut command = sandbox.command(env!("CARGO_BIN_EXE_sheprd-acp"), "helper.log");
    command
        .env("PATH", ".")
        .args(["--headless", "--no-attach", "--", &format!("./{PROGRAM}")]);
    let (status, log) = run_command(&sandbox, command);
    assert!(status.success(), "explicit program failed: {log}");
    assert!(sandbox.dir.join("invoked").exists());
}
