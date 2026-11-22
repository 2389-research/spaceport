// ABOUTME: Exercises device CLI drains against real retained pipe descriptors.
// ABOUTME: Releases fixture writers before asserting bounded completion.
#![allow(clippy::unwrap_used)]
use super::*;
use std::io::Write;

#[cfg(unix)]
use crate::test_sandbox as sandbox;

#[test]
fn a_retained_device_output_writer_cannot_hold_completion_forever() {
    let (pipe, writer) = std::io::pipe().unwrap();
    let reader = capture(pipe, Retain::Prefix(16)).unwrap();
    let (sent, done) = std::sync::mpsc::sync_channel(1);
    let waiting = std::thread::spawn(move || {
        sent.send(finish_output(reader, Instant::now())).unwrap();
    });
    let result = done.recv_timeout(Duration::from_secs(1));
    drop(writer);
    waiting.join().unwrap();
    assert!(
        matches!(result, Ok(Err(crate::Error::Devices(Error::SetupTimeout)))),
        "device output exceeded its deadline: {result:?}"
    );
}

#[test]
fn device_output_keeps_its_bounded_prefix_and_finishes_after_eof() {
    let (pipe, mut writer) = std::io::pipe().unwrap();
    let reader = capture(pipe, Retain::Prefix(4)).unwrap();
    writer.write_all(b"first and discarded tail").unwrap();
    drop(writer);
    assert_eq!(
        finish_output(reader, Instant::now() + Duration::from_secs(1)).unwrap(),
        b"firs"
    );
}

#[cfg(unix)]
#[test]
fn a_device_command_keeps_its_final_error_after_progress_output() {
    let sandbox = sandbox::Sandbox::new();
    let mut child = sandbox
        .command(std::env::current_exe().unwrap(), "device-output.log")
        .args([
            "--exact",
            "menu::devices::setup::tests::output_deadline::device_error_fixture",
            "--ignored",
            "--nocapture",
        ])
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("isolated device output fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(
        status.success(),
        "isolated device output fixture failed: {status}\n{}",
        std::fs::read_to_string(sandbox.dir.join("device-output.log")).unwrap()
    );
}

#[cfg(unix)]
#[test]
#[ignore = "isolated device CLI producer fixture"]
fn device_error_fixture() {
    let (status, stdout, stderr) = run_bounded_untraced(
        std::ffi::OsStr::new("/bin/sh"),
        &["-c", "printf 'result\\n'; printf 'progress progress progress progress\\nfinal failure\\n' >&2; exit 1"],
        Duration::from_secs(2),
        16,
    ).unwrap();
    assert!(!status.success());
    assert_eq!(stdout, b"result\n");
    assert_eq!(last_line(&stderr), "final failure");
}

#[test]
fn device_output_that_closes_at_the_deadline_still_arrives() {
    let (pipe, mut writer) = std::io::pipe().unwrap();
    let reader = capture(pipe, Retain::Prefix(16)).unwrap();
    writer.write_all(b"done").unwrap();
    drop(writer);
    assert_eq!(finish_output(reader, Instant::now()).unwrap(), b"done");
}
