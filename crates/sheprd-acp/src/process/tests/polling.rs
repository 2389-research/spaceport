// ABOUTME: Checks interrupted polling without pretending that a native signal caused the error.
// ABOUTME: Uses a real owned child and preserves non-interruption errors and waitable identity.
use super::*;
use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};
use std::{cell::Cell, io, time::Instant};

#[test]
fn an_injected_interruption_retries_the_real_owned_child_without_idling() {
    let dir = tempfile::tempdir().unwrap();
    let spawned = spawn(
        &OsString::from("/bin/sh"),
        &[OsString::from("-c"), OsString::from("exit 7")],
        &[],
        dir.path(),
    )
    .unwrap();
    let _cleanup = spawned.process.stop_on_drop();
    let pid = Pid::from_raw(spawned.process.lock().id().try_into().unwrap()).unwrap();
    let calls = Cell::new(0);
    let deadline = Instant::now() + Duration::from_secs(10);
    let result = wait_with(
        || {
            calls.set(calls.get() + 1);
            if calls.get() == 1 {
                Err(io::Error::from(io::ErrorKind::Interrupted))
            } else {
                spawned.process.lock().try_wait()
            }
        },
        || {
            assert!(
                calls.get() > 1,
                "an interrupted observation must retry before sleeping"
            );
            assert!(
                Instant::now() < deadline,
                "the real private child never exited"
            );
            thread::sleep(POLL);
        },
    );
    let owned = result.as_ref().ok().map(|_| {
        waitid(
            WaitId::Pid(pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
        )
    });
    spawned.process.stop();
    let status = result.expect("interrupted polling must continue to the real child's status");
    assert_eq!(status.code(), Some(7));
    assert!(
        owned
            .unwrap()
            .unwrap()
            .is_some_and(|status| status.exit_status() == Some(7)),
        "polling must leave the exact leader waitable until cleanup"
    );
}

#[test]
fn a_non_interruption_poll_error_keeps_its_original_os_cause() {
    let dir = tempfile::tempdir().unwrap();
    let error = std::fs::File::open(dir.path().join("absent")).unwrap_err();
    let raw = error.raw_os_error();
    let mut error = Some(error);
    let result = wait_with(
        || Err(error.take().unwrap()),
        || panic!("a fatal polling error must not sleep or retry"),
    );
    let error = result.unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert_eq!(error.raw_os_error(), raw);
}
