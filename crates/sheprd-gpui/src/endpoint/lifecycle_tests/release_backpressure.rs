//! A release that cannot be queued is backpressure, not a dead transport.
use super::*;

#[test]
fn a_full_queue_keeps_a_background_release_deferred_until_it_drains() {
    let (endpoint, _server) = connected_endpoint("ssh:release-backpressure");
    // Nothing drains the queue until the gate opens and the worker connects.
    let (stream, _peer) = sheprd_client::Stream::pair().unwrap();
    let (open, gate) = std::sync::mpsc::channel::<()>();
    let client = sheprd_client::connect_with_connector(
        endpoint.connection.target.clone(),
        ConnectOptions::default(),
        true,
        move |_, _| {
            let _ = gate.recv();
            Ok(stream)
        },
    )
    .unwrap();
    let boot = snapshot().boot_id;
    let mut queued = 0;
    while client.handle.set_focus(&boot, true).is_ok() {
        queued += 1;
    }
    assert!(queued > 0, "the fixture must fill the real queue");
    assert!(matches!(
        client.handle.set_focus(&boot, true),
        Err(sheprd_client::Error::Full)
    ));
    let mut release = Release {
        inbox: endpoint.connection.inbox.clone(),
        drained: endpoint.connection.drained.clone(),
        phase: ReleasePhase::Deferred(client.handle.clone()),
        boot,
    };
    assert!(!release.resolved());
    assert!(
        matches!(release.phase, ReleasePhase::Deferred(_)),
        "a momentarily full queue leaves the release to retry"
    );
    assert!(
        !client.handle.is_disconnected(),
        "a momentarily full queue must not retire the transport"
    );

    // The worker connects and drains the backlog, rejecting each command for
    // want of a snapshot, so the last rejection proves the queue is empty.
    open.send(()).unwrap();
    for _ in 0..queued {
        client.events.recv_timeout(Duration::from_secs(5)).unwrap();
    }
    assert!(
        !release.resolved(),
        "only the acknowledgement resolves a queued release"
    );
    assert!(matches!(release.phase, ReleasePhase::Sent(_)));
    assert!(!client.handle.is_disconnected());
}
