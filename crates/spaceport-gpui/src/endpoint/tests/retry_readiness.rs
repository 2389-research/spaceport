// ABOUTME: Holds redial until the previous reader and its final projection settle.
// ABOUTME: Exercises inbox ownership without dialing any transport or daemon.
use super::*;

fn waiting_endpoint() -> (tempfile::TempDir, Endpoint) {
    let root = tempfile::tempdir().unwrap();
    let endpoint = Endpoint::new(
        "retry-fixture".into(),
        "Retry fixture".into(),
        ConnectTarget::Socket(root.path().join("private.sock")),
        true,
    );
    assert!(endpoint.connection.take_update().is_some());
    (root, endpoint)
}

#[test]
fn a_clean_socket_endpoint_can_dial_after_its_initial_projection() {
    let (_root, endpoint) = waiting_endpoint();
    assert!(endpoint.may_dial());
}

#[test]
fn redial_waits_for_the_previous_event_reader_to_drain() {
    let (_root, endpoint) = waiting_endpoint();
    endpoint.connection.drained.store(false, Ordering::Release);
    assert!(
        !endpoint.may_dial(),
        "the old reader can still deliver shutdown"
    );
    endpoint.connection.drained.store(true, Ordering::Release);
    assert!(endpoint.may_dial());
}

#[test]
fn redial_waits_for_the_final_inbox_projection_to_be_consumed() {
    let (_root, endpoint) = waiting_endpoint();
    endpoint.connection.inbox.lock().unwrap().apply(
        ClientEvent::Disconnected {
            reason: "owned fixture ended".into(),
            ssh: None,
        },
        &crate::state::labels(),
    );
    assert!(
        !endpoint.may_dial(),
        "the final inbox update is still unread"
    );
    assert!(endpoint.connection.take_update().is_some());
    assert!(endpoint.may_dial());
}

#[test]
fn a_contended_inbox_cannot_authorize_redial() {
    let (_root, endpoint) = waiting_endpoint();
    let inbox = endpoint.connection.inbox.clone();
    let guard = inbox.lock().unwrap();
    // The readiness query must not block behind this held guard.
    assert!(
        !endpoint.may_dial(),
        "a locked inbox has unknown retry readiness"
    );
    drop(guard);
    assert!(endpoint.may_dial());
}
