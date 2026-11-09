// ABOUTME: Exercises the retained bounded shell against an isolated local child.
// ABOUTME: Checks failures, output bounds, timeout fencing and child cleanup without remote access.
use super::*;
use crate::Shell;
use std::process::{Command, Stdio};

fn shell() -> (tempfile::TempDir, Shell) {
    let home = tempfile::tempdir().unwrap();
    let mut command = Command::new("/bin/sh");
    command
        .arg("-s")
        .env_clear()
        .env("HOME", home.path())
        .env("PATH", "/usr/bin:/bin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let shell = Shell::start(command).unwrap();
    (home, shell)
}

#[test]
fn remote_steps_report_failure_and_keep_the_session() {
    let (_home, mut shell) = shell();
    let output = shell
        .run("printf 'a\\nb'; false", Duration::from_secs(5))
        .unwrap();
    assert!(!output.success);
    assert_eq!(output.stdout, "a\nb");
    let output = shell.run("printf ok", Duration::from_secs(5)).unwrap();
    assert!(output.success);
    assert_eq!(output.stdout, "ok");
    assert!(matches!(
        shell.run("sleep 5", Duration::from_millis(200)),
        Err(Error::Timeout)
    ));
    // A step that overran leaves the session unusable rather than out of step.
    assert!(shell.run("true", Duration::from_secs(5)).is_err());
}

#[test]
fn oversized_output_breaks_the_session() {
    let (_home, mut shell) = shell();
    assert!(matches!(
        shell.run("yes x", Duration::from_secs(5)),
        Err(Error::Size)
    ));
    assert!(matches!(
        shell.run("true", Duration::from_secs(5)),
        Err(Error::Unreachable)
    ));
}

#[test]
fn shell_start_reports_a_typed_process_error() {
    let home = tempfile::tempdir().unwrap();
    let command = Command::new(home.path().join("absent-shell"));
    let error = Shell::start(command).err().unwrap();
    assert!(matches!(error, Error::Process { .. }));
    assert!(std::error::Error::source(&error).is_some());
}
