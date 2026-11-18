// ABOUTME: Checks Windows agent shim descendants, stop coordination, and exit status.
// ABOUTME: Owns exact fixture handles and bounds every wait without shared daemons.
use super::*;
use std::{
    io::{Read, Write},
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::PathBuf,
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{
        OpenProcess, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE, TerminateProcess, WaitForSingleObject,
    },
};

const FIXTURE: &str = "process::windows_tests::agent_fixture";
const TIMEOUT: Duration = Duration::from_secs(5);

struct Descendant(OwnedHandle);

struct Stop(Option<AgentProcess>);

impl Drop for Stop {
    fn drop(&mut self) {
        if let Some(process) = &self.0 {
            process.stop();
        }
    }
}

impl Descendant {
    #[allow(unsafe_code)]
    fn open(pid: u32) -> Self {
        // Std has no safe API to open an existing descendant's process handle.
        // SAFETY: the fixture writes its own PID before waiting. This opens an
        // owned kernel handle; later checks and cleanup never target a PID.
        let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE | PROCESS_TERMINATE, 0, pid) };
        assert!(!handle.is_null(), "{}", std::io::Error::last_os_error());
        // SAFETY: OpenProcess returned a valid handle whose ownership moves here.
        Self(unsafe { OwnedHandle::from_raw_handle(handle) })
    }

    #[allow(unsafe_code)]
    fn exited(&self, timeout: Duration) -> bool {
        // SAFETY: the owned process handle outlives this bounded wait.
        let status =
            unsafe { WaitForSingleObject(self.0.as_raw_handle(), timeout.as_millis() as u32) };
        assert!(matches!(status, WAIT_OBJECT_0 | WAIT_TIMEOUT));
        status == WAIT_OBJECT_0
    }
}

impl Drop for Descendant {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: only this exact fixture process handle is terminated. Reap it
        // before the fixture directory disappears, even when an assertion fails.
        unsafe {
            TerminateProcess(self.0.as_raw_handle(), 1);
            WaitForSingleObject(self.0.as_raw_handle(), 5_000);
        }
    }
}

fn until(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + TIMEOUT;
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "fixture did not reach its checkpoint"
        );
        thread::sleep(POLL);
    }
}

fn descendant(dir: &Path, name: &str) -> Descendant {
    let path = dir.join(name);
    let mut pid = None;
    until(|| {
        pid = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| s.trim().parse().ok());
        pid.is_some()
    });
    Descendant::open(pid.unwrap())
}

fn fixture(exit_parent: bool) -> (tempfile::TempDir, Spawned, Descendant, Descendant, Stop) {
    let dir = tempfile::tempdir().unwrap();
    let shim = dir.path().join("agent.cmd");
    let executable = std::env::current_exe().unwrap();
    // Exercise the cmd.exe -> npm-like shim -> agent -> tool ownership chain.
    std::fs::write(
        &shim,
        format!(
            "@echo off\r\n\"{}\" --exact {FIXTURE} --ignored --nocapture\r\n",
            executable.display()
        ),
    )
    .unwrap();
    let root = dir.path().to_str().unwrap();
    let spawned = spawn(
        &shim.into_os_string(),
        &[],
        &[
            ("SPACEPORT_PROCESS_FIXTURE", root),
            ("SPACEPORT_PROCESS_ROLE", "agent"),
        ],
        dir.path(),
    )
    .unwrap();
    let stop = Stop(Some(spawned.process.clone()));
    let agent = descendant(dir.path(), "agent.pid");
    let tool = descendant(dir.path(), "tool.pid");
    if exit_parent {
        std::fs::write(dir.path().join("exit-agent"), "exit").unwrap();
        until(|| spawned.process.lock().try_wait().unwrap().is_some());
    }
    (dir, spawned, agent, tool, stop)
}

#[test]
fn stopping_a_windows_shim_ends_its_agent_and_tool() {
    let (_dir, spawned, agent, tool, _stop) = fixture(false);
    assert!(!agent.exited(Duration::ZERO));
    assert!(!tool.exited(Duration::ZERO));
    let waiting = spawned.process.clone();
    let (done, finished) = std::sync::mpsc::channel();
    let waiter = thread::spawn(move || {
        let _ = done.send(waiting.wait());
    });
    spawned.process.stop();
    assert!(agent.exited(TIMEOUT), "agent outlived its shim");
    assert!(tool.exited(TIMEOUT), "tool outlived its shim");
    assert!(!finished.recv_timeout(TIMEOUT).unwrap().unwrap().success());
    waiter.join().unwrap();
    spawned.process.stop();
}

#[test]
fn stopping_an_exited_windows_shim_still_ends_its_tool() {
    let (_dir, spawned, agent, tool, _stop) = fixture(true);
    assert!(spawned.process.wait().unwrap().success());
    assert!(agent.exited(TIMEOUT));
    assert!(!tool.exited(Duration::ZERO));
    spawned.process.stop();
    assert!(tool.exited(TIMEOUT), "tool outlived its exited shim");
    assert!(
        spawned.process.wait().unwrap().success(),
        "stop changed the cached success status"
    );
}

#[test]
fn dropping_the_last_windows_process_owner_ends_its_tree() {
    let (_dir, spawned, agent, tool, mut stop) = fixture(false);
    stop.0 = None;
    drop(spawned);
    assert!(agent.exited(TIMEOUT), "agent outlived its process owner");
    assert!(tool.exited(TIMEOUT), "tool outlived its process owner");
}

#[test]
fn windows_process_keeps_stdio_environment_and_exit_status() {
    let dir = tempfile::tempdir().unwrap();
    let mut spawned = spawn(
        &OsString::from("cmd.exe"),
        &[
            "/D".into(),
            "/S".into(),
            "/C".into(),
            "set /p line=& echo %SPACEPORT_PROCESS_VALUE%& echo diagnostic 1>&2& exit /b 7".into(),
        ],
        &[("SPACEPORT_PROCESS_VALUE", "fixture-value")],
        dir.path(),
    )
    .unwrap();
    let _stop = Stop(Some(spawned.process.clone()));
    spawned.input.write_all(b"input\r\n").unwrap();
    drop(spawned.input);
    let (output, read) = std::sync::mpsc::channel();
    let reader = thread::spawn(move || {
        let mut contents = String::new();
        spawned.output.read_to_string(&mut contents).unwrap();
        output.send(contents).unwrap();
    });
    let output = read.recv_timeout(TIMEOUT).unwrap();
    reader.join().unwrap();
    assert!(output.contains("fixture-value"));
    assert_eq!(spawned.process.wait().unwrap().code(), Some(7));
    until(|| {
        spawned
            .process
            .stderr_tail()
            .iter()
            .any(|line| line.trim() == "diagnostic")
    });
    spawned.process.stop();
    assert_eq!(spawned.process.wait().unwrap().code(), Some(7));
}

#[test]
#[ignore = "launched only by an isolated process fixture"]
fn agent_fixture() {
    let root = PathBuf::from(std::env::var_os("SPACEPORT_PROCESS_FIXTURE").unwrap());
    let role = std::env::var("SPACEPORT_PROCESS_ROLE").unwrap();
    let mut tool = if role == "agent" {
        let mut command = sheprd_client::new_command(std::env::current_exe().unwrap());
        command
            .args(["--exact", FIXTURE, "--ignored", "--nocapture"])
            .env("SPACEPORT_PROCESS_ROLE", "tool");
        Some(command.spawn().unwrap())
    } else {
        None
    };
    std::fs::write(
        root.join(format!("{role}.pid")),
        std::process::id().to_string(),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if role == "agent" && root.join("exit-agent").exists() {
            return;
        }
        thread::sleep(POLL);
    }
    if let Some(tool) = &mut tool {
        let _ = tool.kill();
        let _ = tool.wait();
    }
}
