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
            |_| true,
            Duration::ZERO,
        )
        .unwrap();
    assert_eq!(signals, [Signal::TERM, Signal::KILL]);
    assert_eq!(child.try_wait().unwrap().unwrap().code(), Some(9));
    child
        .stop_with(
            |_, _| panic!("retired identity must never receive a signal"),
            Child::try_wait,
            |_| true,
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
        .stop_with(
            kill_process_group,
            Child::try_wait,
            |_| true,
            Duration::ZERO,
        )
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
        |_| true,
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

#[test]
fn an_exited_agent_alone_in_its_group_ends_the_grace_early() {
    let mut child = exited("exit 0");
    let (finished, done) = std::sync::mpsc::sync_channel(1);
    let stopping = std::thread::spawn(move || {
        let mut signals = Vec::new();
        let result = child.stop_with(
            |group, signal| {
                signals.push(signal);
                kill_process_group(group, signal)
            },
            Child::try_wait,
            |_| false,
            Duration::from_secs(600),
        );
        finished.send((result.is_ok(), signals)).unwrap();
    });
    // Bounds a failing run only: the grace above is far longer.
    let (stopped, signals) = done.recv_timeout(Duration::from_secs(10)).unwrap();
    stopping.join().unwrap();
    assert!(stopped);
    assert_eq!(signals, [Signal::TERM, Signal::KILL]);
}

#[test]
fn a_running_agent_keeps_its_grace_even_when_its_group_looks_empty() {
    let mut command = sheprd_client::new_command("/bin/sh");
    command
        .args(["-c", "trap '' TERM; echo ready; while :; do sleep 1; done"])
        .stdout(process::Stdio::piped());
    let mut child = spawn(command).unwrap();
    // TERM must arrive after the trap is set, or the shell simply exits.
    let mut ready = String::new();
    io::BufRead::read_line(
        &mut io::BufReader::new(child.stdout.take().unwrap()),
        &mut ready,
    )
    .unwrap();
    let mut checked = false;
    child
        .stop_with(
            kill_process_group,
            Child::try_wait,
            |_| {
                checked = true;
                false
            },
            Duration::from_millis(100),
        )
        .unwrap();
    assert!(!checked, "the group was listed while its leader still ran");
    assert!(child.try_wait().unwrap().is_some());
}

#[test]
fn group_listing_sees_a_descendant_the_exited_agent_left_behind() {
    let mut alone = exited("exit 0");
    let group = alone.group.unwrap();
    assert!(!others_remain(group));
    alone
        .stop_with(
            kill_process_group,
            Child::try_wait,
            |_| true,
            Duration::ZERO,
        )
        .unwrap();

    let mut left = exited("sleep 30 & exit 0");
    let group = left.group.unwrap();
    let listed = others_remain(group);
    left.stop_with(
        kill_process_group,
        Child::try_wait,
        |_| true,
        Duration::ZERO,
    )
    .unwrap();
    assert!(
        listed,
        "the background sleep was not seen in the agent's group"
    );
}
