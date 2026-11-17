// ABOUTME: Verifies unsent retries and submitted activation receipts stay distinct.
// ABOUTME: Covers boot fencing, fatal queue failures, and late acknowledgements.
use super::*;
use activation::{Failure, Fence, Transition};
use std::time::Instant;

fn queue(fence: &mut Fence, result: Result<String, sheprd_client::SendError>) {
    fence.transition(Transition::Queued {
        result,
        boot: "boot".into(),
        focus: Some(crate::NavigationTarget::Pane("target".into())),
        active: true,
        now: Instant::now(),
    });
}

#[test]
fn full_retains_the_unsent_target_until_retry_or_boot_replacement() {
    let mut fence = Fence::Open;
    queue(&mut fence, Err(sheprd_client::Error::Full));
    assert!(
        matches!(&fence, Fence::Retry { boot, focus: Some(crate::NavigationTarget::Pane(id)), .. }
        if boot == "boot" && id == "target")
    );
    assert!(fence.pending());
    assert!(fence.fenced());
    assert!(fence.deadline().is_none());
    fence.transition(Transition::Expire(
        Instant::now() + std::time::Duration::from_secs(60),
    ));
    assert!(matches!(fence, Fence::Retry { .. }));
    fence.transition(Transition::Boot("replacement"));
    assert!(matches!(
        fence,
        Fence::Failed {
            cause: Failure::BootChanged,
            ..
        }
    ));
}

#[test]
fn submitted_barriers_expire_without_becoming_retryable() {
    let mut fence = Fence::Open;
    queue(&mut fence, Ok("barrier".into()));
    fence.transition(Transition::Expire(fence.deadline().unwrap()));
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
fn fatal_queue_errors_cannot_be_retried_as_full() {
    let mut fence = Fence::Open;
    queue(&mut fence, Err(sheprd_client::Error::Disconnected));
    assert!(matches!(
        fence,
        Fence::Failed {
            cause: Failure::Queue,
            ..
        }
    ));
    assert!(fence.pending());
}

#[test]
fn submitted_or_rejected_barriers_ignore_unrelated_and_late_receipts() {
    let mut fence = Fence::Open;
    queue(&mut fence, Ok("barrier".into()));
    let submitted = fence.clone();
    let result = serde_json::json!({"type": "client_shell_surface_set", "active": true, "projection_revision": 7});
    fence.transition(Transition::Response {
        request: "unrelated",
        result: Some(&result),
    });
    assert_eq!(fence, submitted);
    fence.transition(Transition::Rejected("barrier"));
    let rejected = fence.clone();
    fence.transition(Transition::Response {
        request: "barrier",
        result: Some(&result),
    });
    assert_eq!(fence, rejected);
    assert!(fence.failed());
}

#[test]
fn selecting_an_inactive_receipt_renews_its_budget_without_losing_the_receipt() {
    let mut fence = Fence::Open;
    fence.transition(Transition::Queued {
        result: Ok("release".into()),
        boot: "boot".into(),
        focus: None,
        active: false,
        now: Instant::now() - std::time::Duration::from_secs(60),
    });
    fence.transition(Transition::Response {
        request: "release",
        result: Some(&serde_json::json!({"type": "client_shell_surface_set", "active": false, "projection_revision": 7})),
    });
    let deadline = Instant::now() + std::time::Duration::from_secs(5);
    fence.transition(Transition::Select(deadline));
    assert_eq!(fence.request().unwrap().request, "release");
    assert_eq!(fence.revision(), Some(7));
    assert_eq!(fence.deadline(), Some(deadline));
    assert!(fence.fenced());
}

#[test]
fn selection_budgets_cannot_replace_a_newer_active_submission() {
    let mut fence = Fence::Open;
    queue(&mut fence, Ok("active".into()));
    let submitted = fence.clone();
    fence.transition(Transition::Select(Instant::now()));
    assert_eq!(fence, submitted);
}
