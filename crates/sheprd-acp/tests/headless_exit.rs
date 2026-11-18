// ABOUTME: Checks the shipped headless helper's exit status with isolated agent processes.
// ABOUTME: Keeps all helper state private and reaps only the test's child.
#![allow(clippy::unwrap_used)]

#[path = "../../test-support/sandbox.rs"]
#[allow(dead_code, clippy::disallowed_methods, clippy::expect_used)]
mod sandbox;

use std::process::Child;

struct Helper(Child);

impl Drop for Helper {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn run(code: u8) -> (std::process::ExitStatus, String) {
    let sandbox = sandbox::Sandbox::new();
    let mut command = sandbox.command(env!("CARGO_BIN_EXE_sheprd-acp"), "helper.log");
    command.args(["--headless", "--no-attach", "--"]);
    #[cfg(unix)]
    command.args(["/bin/sh", "-c", &format!("read request; exit {code}")]);
    #[cfg(windows)]
    command.args(["cmd.exe", "/D", "/S", "/C", &format!("exit /b {code}")]);
    let mut helper = Helper(command.spawn().unwrap());
    let status = helper.0.wait().unwrap();
    let log = std::fs::read_to_string(sandbox.dir.join("helper.log")).unwrap();
    (status, log)
}

#[test]
fn an_agent_failure_makes_the_headless_helper_fail() {
    let (status, log) = run(7);
    assert!(!status.success(), "failed agent reported success: {log}");
    assert!(log.contains("agent"), "missing failure diagnostic: {log}");
}

#[test]
fn a_successful_agent_keeps_a_successful_headless_exit() {
    let (status, log) = run(0);
    assert!(status.success(), "successful agent reported failure: {log}");
    assert!(log.is_empty(), "unexpected diagnostic: {log}");
}
