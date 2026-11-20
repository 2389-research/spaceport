// ABOUTME: Checks the shipped headless helper's exit status with isolated agent processes.
// ABOUTME: Keeps all helper state private and reaps only the test's child.
#![allow(clippy::unwrap_used)]

#[path = "../../test-support/sandbox.rs"]
#[allow(dead_code, clippy::disallowed_methods, clippy::expect_used)]
mod sandbox;

#[cfg(unix)]
#[path = "../../test-support/executable.rs"]
#[expect(
    clippy::disallowed_methods,
    reason = "shared Unix-only fixture writer runs /bin/sh; no Windows console can open"
)]
mod test_executable;

use std::{
    process::{Child, Command},
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
#[path = "headless_exit/path_lookup.rs"]
mod path_lookup;

struct Helper(Child);

impl Drop for Helper {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// The helper's exit, with a failure-only bound: a helper that never observes
/// its agent fails the test instead of holding the suite.
impl Helper {
    fn wait(&mut self) -> std::process::ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = self.0.try_wait().unwrap() {
                return status;
            }
            assert!(Instant::now() < deadline, "headless helper did not exit");
            thread::sleep(Duration::from_millis(10));
        }
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
    run_command(&sandbox, command)
}

fn run_command(
    sandbox: &sandbox::Sandbox,
    mut command: Command,
) -> (std::process::ExitStatus, String) {
    let mut helper = Helper(command.spawn().unwrap());
    let status = helper.wait();
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
