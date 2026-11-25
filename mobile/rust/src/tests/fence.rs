// ABOUTME: Exercises a barrier Herdr's queue refused, and one it refused outright.
// ABOUTME: Checks the phone retries what never went out and reports what did fail.
use super::*;
use spaceport_activation::Failure;

fn fenced() -> Projection {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    state.surface(Arc::new(surface()));
    state
}

#[test]
fn a_barrier_the_queue_refused_keeps_its_destination_and_the_connection() {
    let mut state = fenced();
    state.start_fence(
        "choose".into(),
        OwnedNavigationTarget::Pane("other".into()),
        Err(spaceport_client::Error::Full),
    );
    assert!(!state.disconnected, "a full queue is not a reason to drop");
    assert!(state.navigating(), "input stays fenced while it waits");
    assert!(state.fence_failure().is_none());
    assert!(matches!(
        state.retry_barrier(),
        Some((boot, Some(OwnedNavigationTarget::Pane(id)))) if boot == "boot-v1" && id == "other"
    ));
}

#[test]
fn a_barrier_offered_again_settles_once_its_revision_arrives() {
    let mut state = fenced();
    state.start_fence(
        "choose".into(),
        OwnedNavigationTarget::Pane("w1:p1".into()),
        Err(spaceport_client::Error::Full),
    );
    let Some((boot, focus)) = state.retry_barrier() else {
        panic!("the refused barrier is waiting to be offered again")
    };
    state.barrier_queued(Ok("barrier".into()), boot, focus);
    assert!(state.retry_barrier().is_none(), "it entered the queue now");
    state.receipt(
        "barrier",
        &serde_json::json!({"result":{"type":"client_shell_surface_set","active":true,"projection_revision":7}}),
    );
    assert!(
        state.navigating(),
        "the acknowledgement alone is not a frame to show"
    );
    state.surface(Arc::new(surface()));
    assert!(!state.navigating(), "the acknowledged surface is presented");
    assert!(state.target().is_some());
}

#[test]
fn a_barrier_refused_outright_is_reported_without_dropping_the_connection() {
    let mut state = fenced();
    state.start_fence(
        "choose".into(),
        OwnedNavigationTarget::Pane("other".into()),
        Err(spaceport_client::Error::Disconnected),
    );
    assert!(!state.disconnected);
    assert!(state.error.is_some(), "the refusal is shown");
    assert!(
        matches!(state.fence_failure(), Some(Failure::Queue)),
        "the failure is the queue's, and the window words it"
    );
}
