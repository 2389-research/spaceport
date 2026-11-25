// ABOUTME: Checks a barrier the queue refused is retried and one it took expires.
// ABOUTME: Covers a settled destination and the projection an acknowledgement unlocks.
use super::*;
use crate::{Failure, Fence, OwnedNavigationTarget, SurfaceActivation, TIMEOUT, Transition};
use std::time::{Duration, Instant};

fn activation(request: &str, focus: Option<OwnedNavigationTarget>) -> SurfaceActivation {
    SurfaceActivation {
        request: request.into(),
        boot: "boot-v1".into(),
        focus,
        active: true,
    }
}

fn pane(id: &str) -> Option<OwnedNavigationTarget> {
    Some(OwnedNavigationTarget::Pane(id.into()))
}

fn queue(fence: &mut Fence, result: Result<String, spaceport_client::SendError>, now: Instant) {
    fence.transition(Transition::Queued {
        result,
        boot: "boot-v1".into(),
        focus: pane("w1:p1"),
        active: true,
        now,
    });
}

#[test]
fn an_open_fence_holds_nothing_back() {
    assert!(!Fence::Open.fenced());
    assert!(Fence::Open.ready(&snapshot(), &surface(7)));
}

#[test]
fn a_barrier_the_queue_refused_is_offered_again_rather_than_failing() {
    let mut fence = Fence::Open;
    queue(
        &mut fence,
        Err(spaceport_client::Error::Full),
        Instant::now(),
    );
    assert_eq!(
        fence,
        Fence::Retry {
            boot: "boot-v1".into(),
            focus: pane("w1:p1"),
        }
    );
    assert!(fence.fenced(), "input stays off while the barrier waits");
    assert!(
        fence.deadline().is_none(),
        "a barrier that was never sent cannot expire"
    );
}

#[test]
fn a_settled_destination_is_dropped_from_a_barrier_the_queue_refused() {
    let mut fence = Fence::Open;
    queue(
        &mut fence,
        Err(spaceport_client::Error::Full),
        Instant::now(),
    );
    fence.transition(Transition::Settled);
    assert_eq!(
        fence,
        Fence::Retry {
            boot: "boot-v1".into(),
            focus: None,
        }
    );
}

#[test]
fn a_barrier_refused_outright_fails_without_a_retry() {
    let mut fence = Fence::Open;
    queue(
        &mut fence,
        Err(spaceport_client::Error::Disconnected),
        Instant::now(),
    );
    assert!(matches!(
        fence,
        Fence::Failed {
            request: None,
            cause: Failure::Queue,
        }
    ));
}

#[test]
fn a_submitted_barrier_expires_and_is_never_offered_again() {
    let mut fence = Fence::Open;
    let now = Instant::now();
    queue(&mut fence, Ok("barrier".into()), now);
    assert_eq!(fence.deadline(), Some(now + TIMEOUT));
    fence.transition(Transition::Expire(now + TIMEOUT - Duration::from_millis(1)));
    assert!(!fence.failed());
    fence.transition(Transition::Expire(now + TIMEOUT));
    assert!(matches!(
        fence,
        Fence::Failed {
            cause: Failure::Timeout,
            ..
        }
    ));
    assert!(fence.fenced());
}

#[test]
fn a_settled_fence_drops_a_destination_the_barrier_is_still_waiting_on() {
    let mut fence = Fence::for_test(activation("barrier", pane("w1:p2")), Some(8));
    assert!(
        !fence.ready(&snapshot(), &surface(8)),
        "the snapshot has not focused the pane it was aimed at"
    );
    fence.transition(Transition::Settled);
    assert!(
        fence
            .request()
            .is_some_and(|request| request.focus.is_none()),
        "the barrier is still out, but its destination is no evidence"
    );
    assert!(matches!(fence, Fence::Acknowledged { .. }));
    assert!(
        fence.ready(&snapshot(), &surface(8)),
        "the barrier's own acknowledgement is enough once it settles"
    );
}

#[test]
fn an_acknowledgement_unlocks_only_the_projection_it_named() {
    let fence = Fence::for_test(activation("barrier", None), Some(8));
    assert_eq!(fence.revision(), Some(8));
    let snapshot = snapshot();
    assert!(fence.ready(&snapshot, &surface(8)));
    assert!(!fence.ready(&snapshot, &surface(7)), "an earlier frame");

    let mut replaced = snapshot.clone();
    replaced.boot_id = "boot-v2".into();
    assert!(!fence.ready(&replaced, &surface(8)), "another daemon boot");
}

#[test]
fn a_boot_that_changed_under_the_barrier_fails_it() {
    let mut fence = Fence::for_test(activation("barrier", None), Some(8));
    fence.transition(Transition::Boot("boot-v2"));
    assert!(matches!(
        fence,
        Fence::Failed {
            cause: Failure::BootChanged,
            ..
        }
    ));
}

#[test]
fn a_receipt_for_another_request_cannot_acknowledge_this_barrier() {
    let mut fence = Fence::Open;
    queue(&mut fence, Ok("barrier".into()), Instant::now());
    fence.transition(Transition::Response {
        request: "someone-else",
        result: Some(&acknowledgement(true, 8)),
    });
    assert!(matches!(fence, Fence::Sent { .. }), "still unanswered");
    fence.transition(Transition::Response {
        request: "barrier",
        result: None,
    });
    assert!(matches!(
        fence,
        Fence::Failed {
            cause: Failure::InvalidResponse,
            ..
        }
    ));
}
