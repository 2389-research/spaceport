//! `start`: a failed start's stderr drain ends at the start deadline, so a core
//! the start command left running cannot hold the reader open.
use super::*;
use crate::Error;

/// A start command that fails after writing to stderr, leaving a child that
/// keeps the pipe open, as a core answering `server start` would.
const FAILING_START: &str = "\
[ \"$1 $2 $3\" = 'server start --json' ] || exit 9
/bin/sleep 30 &
printf %s \"$!\" > \"$0.pid\"
printf 'boom' >&2
exit 1
";

/// The child the fixture left holding stderr, recorded so the test can end it.
fn holder_pid(binary: &std::path::Path) -> i32 {
    let path = binary.with_extension("pid");
    let mut pid = String::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    while pid.is_empty() {
        assert!(Instant::now() < deadline, "the fixture recorded no child");
        pid = std::fs::read_to_string(&path).unwrap_or_default();
    }
    pid.parse().unwrap()
}

fn end_holder(pid: i32) {
    if let Some(pid) = rustix::process::Pid::from_raw(pid) {
        let _ = rustix::process::kill_process(pid, rustix::process::Signal::KILL);
    }
}

#[test]
fn failed_start_ends_the_stderr_drain_at_the_deadline() {
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("core");
    crate::test_executable::write(&binary, FAILING_START, 0o700).unwrap();
    let started = Instant::now();
    let result = crate::start::start(&binary, |_| {}, Duration::from_secs(2));
    // The holder outlives the deadline by far, so a drain that waits for EOF
    // returns long after this bound. Only a failing run reads this clock.
    let elapsed = started.elapsed();
    end_holder(holder_pid(&binary));
    assert!(elapsed < Duration::from_secs(10), "{elapsed:?}");
    let Err(Error::StartFailed { status, stderr }) = result else {
        panic!("{result:?}");
    };
    assert_eq!(status.code(), Some(1));
    assert_eq!(stderr, "boom");
}
