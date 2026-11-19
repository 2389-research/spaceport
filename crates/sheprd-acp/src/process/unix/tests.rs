// ABOUTME: Checks real Unix wait status and process-group retirement ordering.
// ABOUTME: Records signals only while the native leader remains waitable.
#![allow(clippy::unwrap_used)]
use super::*;

fn exited(script: &str) -> Child {
    let mut command = sheprd_client::new_command("/bin/sh");
    command.args(["-c", script]);
    let mut child = spawn(command).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while child.try_wait().unwrap().is_none() {
        assert!(Instant::now() < deadline, "fixture did not exit");
        std::thread::sleep(Duration::from_millis(1));
    }
    child
}

#[test]
fn group_signals_precede_the_only_reap_and_never_repeat_after_retirement() {
    let mut child = exited("exit 9");
    let mut signals = Vec::new();
    child
        .stop_with(
            |group, signal| {
                let owned = waitid(
                    WaitId::Pid(group),
                    WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
                );
                assert!(owned.unwrap().is_some(), "every signal precedes the reap");
                signals.push(signal);
                kill_process_group(group, signal)
            },
            Child::try_wait,
            Duration::ZERO,
        )
        .unwrap();
    assert_eq!(signals, [Signal::TERM, Signal::KILL]);
    assert_eq!(child.try_wait().unwrap().unwrap().code(), Some(9));
    child
        .stop_with(
            |_, _| panic!("retired identity must never receive a signal"),
            Child::try_wait,
            Duration::ZERO,
        )
        .unwrap();
    assert_eq!(child.try_wait().unwrap().unwrap().code(), Some(9));
}

#[test]
fn signaled_exit_status_survives_observation_and_cleanup() {
    let mut child = exited("kill -TERM $$");
    let status = child.try_wait().unwrap().unwrap();
    assert_eq!(status.code(), None);
    assert_eq!(status.signal(), Some(15));
    assert!(!status.core_dumped());
    child
        .stop_with(kill_process_group, Child::try_wait, Duration::ZERO)
        .unwrap();
    assert_eq!(child.try_wait().unwrap(), Some(status));
}

fn observation_failure(kind: io::ErrorKind) {
    let mut child = exited("exit 9");
    let mut signals = Vec::new();
    let mut fail = true;
    let result = child.stop_with(
        |group, signal| {
            signals.push(signal);
            kill_process_group(group, signal)
        },
        |child| {
            if std::mem::take(&mut fail) {
                Err(io::Error::from(kind))
            } else {
                child.try_wait()
            }
        },
        Duration::from_millis(1),
    );
    let retired = child.group.is_none();
    child.stop();
    assert!(
        result.is_ok(),
        "observation error skipped owned cleanup: {result:?}"
    );
    assert!(retired);
    assert_eq!(signals, [Signal::TERM, Signal::KILL]);
}

#[test]
fn interrupted_observation_still_kills_and_reaps_the_owned_group() {
    observation_failure(io::ErrorKind::Interrupted);
}

#[test]
fn failed_observation_still_kills_and_reaps_the_owned_group() {
    observation_failure(io::ErrorKind::Other);
}
