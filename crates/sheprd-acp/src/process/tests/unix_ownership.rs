// ABOUTME: Verifies Unix agent status observation preserves native process ownership.
// ABOUTME: Keeps the group leader waitable until group cleanup and final reaping.
use super::*;
use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};

#[test]
fn the_run_owner_stops_the_agent_while_a_waiter_still_holds_a_handle() {
    let dir = tempfile::tempdir().unwrap();
    let spawned = spawn(
        &OsString::from("/bin/sh"),
        &[OsString::from("-c"), OsString::from("exec sleep 30")],
        &[],
        dir.path(),
    )
    .unwrap();
    let guard = spawned.process.stop_on_drop();
    let watched = spawned.process.clone();
    let (finished, receiver) = std::sync::mpsc::sync_channel(1);
    let waiter = thread::spawn(move || finished.send(watched.wait()).unwrap());
    drop(guard);
    let result = receiver.recv_timeout(Duration::from_secs(10));
    // Always clean the fixture before asserting a failing guard contract.
    spawned.process.stop();
    waiter.join().unwrap();
    let status = result
        .expect("the run owner left the agent and waiter alive")
        .unwrap();
    assert!(!status.success());
}

#[test]
fn observing_agent_exit_keeps_its_identity_until_group_cleanup() {
    let dir = tempfile::tempdir().unwrap();
    let spawned = spawn(
        &OsString::from("/bin/sh"),
        &[OsString::from("-c"), OsString::from("exit 7")],
        &[],
        dir.path(),
    )
    .unwrap();
    let pid = Pid::from_raw(spawned.process.lock().id().try_into().unwrap()).unwrap();
    let status = spawned.process.wait().unwrap();
    assert_eq!(status.code(), Some(7));
    let owned = waitid(
        WaitId::Pid(pid),
        WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
    );
    assert!(
        owned.is_ok_and(|status| status.is_some_and(|status| status.exit_status() == Some(7))),
        "status observation must leave the leader waitable: {owned:?}"
    );
    spawned.process.stop();
    assert!(matches!(
        waitid(
            WaitId::Pid(pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT
        ),
        Err(rustix::io::Errno::CHILD)
    ));
    assert_eq!(spawned.process.wait().unwrap(), status);
    spawned.process.stop();
    assert_eq!(spawned.process.wait().unwrap(), status);
}

#[test]
fn concurrent_stops_and_wait_keep_the_observed_exit_status() {
    let dir = tempfile::tempdir().unwrap();
    let spawned = spawn(
        &OsString::from("/bin/sh"),
        &[OsString::from("-c"), OsString::from("exit 7")],
        &[],
        dir.path(),
    )
    .unwrap();
    let expected = spawned.process.wait().unwrap();
    let (finished, receiver) = std::sync::mpsc::channel();
    let mut workers = Vec::new();
    for stop in [true, true, false] {
        let process = spawned.process.clone();
        let finished = finished.clone();
        workers.push(thread::spawn(move || {
            if stop {
                process.stop();
            }
            finished.send(process.wait()).unwrap();
        }));
    }
    for _ in &workers {
        assert_eq!(
            receiver
                .recv_timeout(Duration::from_secs(10))
                .unwrap()
                .unwrap(),
            expected
        );
    }
    for worker in workers {
        worker.join().unwrap();
    }
}

#[test]
fn the_last_owner_cleans_up_and_reaps_an_observed_agent() {
    let dir = tempfile::tempdir().unwrap();
    let spawned = spawn(
        &OsString::from("/bin/sh"),
        &[OsString::from("-c"), OsString::from("exit 7")],
        &[],
        dir.path(),
    )
    .unwrap();
    let pid = Pid::from_raw(spawned.process.lock().id().try_into().unwrap()).unwrap();
    assert_eq!(spawned.process.wait().unwrap().code(), Some(7));
    drop(spawned);
    assert!(matches!(
        waitid(
            WaitId::Pid(pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT
        ),
        Err(rustix::io::Errno::CHILD)
    ));
}
