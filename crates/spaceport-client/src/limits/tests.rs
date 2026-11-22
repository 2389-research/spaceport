#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The bounded child wait: an exit, a missed deadline, and a stop, each
//! against a child the test controls rather than a clock of its own. The
//! fixtures are `/bin/sh` and `cat`; the wait itself has no platform branch.

use super::wait::{Exit, Stop};
use super::*;
use std::{
    process::{Child, Stdio},
    time::{Duration, Instant},
};

/// Far longer than any test runs: a wait that ends at all ended on the child's
/// own exit, never on this bound.
const BOUND: Duration = Duration::from_secs(60);

/// A child that cannot exit on its own: it reads a standard input this test
/// holds open for as long as the wait under test lasts.
fn blocked() -> Child {
    let mut command = crate::new_command("cat");
    command.stdin(Stdio::piped()).stdout(Stdio::null());
    command.spawn().unwrap()
}

/// Whether the wait reaped the child: a reaped child reports its status
/// instead of running.
fn reaped(child: &mut Child) -> bool {
    matches!(child.try_wait(), Ok(Some(_)))
}

#[test]
fn a_child_that_exits_reports_its_status() {
    let mut command = crate::new_command("/bin/sh");
    command.args(["-c", "exit 3"]);
    let mut child = command.spawn().unwrap();
    let ended = wait::exit(&mut child, Instant::now() + BOUND, || false).unwrap();
    let Exit::Exited(status) = ended else {
        panic!("expected an exit, not {ended:?}");
    };
    assert_eq!(status.code(), Some(3));
}

#[test]
fn a_passed_deadline_kills_and_reaps_the_child() {
    let mut child = blocked();
    let ended = wait::exit(&mut child, Instant::now(), || false).unwrap();
    assert_eq!(ended, Exit::Stopped(Stop::TimedOut));
    assert!(reaped(&mut child), "a timed-out child was left running");
}

#[test]
fn a_stop_kills_and_reaps_the_child() {
    let mut child = blocked();
    let ended = wait::exit(&mut child, Instant::now() + BOUND, || true).unwrap();
    assert_eq!(ended, Exit::Stopped(Stop::Cancelled));
    assert!(reaped(&mut child), "a stopped child was left running");
}
