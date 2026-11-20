// ABOUTME: Verifies terminal delivery when connection setup unwinds.
// ABOUTME: Keeps worker cancellation and bounded event delivery observable.
use super::*;

#[test]
fn connector_panic_reports_disconnect_before_closing_events() {
    let client = connect_with_connector(
        ConnectTarget::Local,
        ConnectOptions::default(),
        true,
        |_, _| panic!("fixture connector panic"),
    )
    .unwrap();
    let event = client.events.recv_timeout(Duration::from_secs(3));
    assert!(
        matches!(event, Ok(ClientEvent::Disconnected { ssh: None, .. })),
        "worker must publish its terminal failure: {event:?}"
    );
    assert!(matches!(
        client.events.recv_timeout(Duration::from_secs(3)),
        Err(crossbeam_channel::RecvTimeoutError::Disconnected)
    ));
    assert!(client.handle.is_disconnected());
}

#[test]
fn cancelled_connector_panic_closes_without_replaying_events() {
    let (entered, started) = bounded(1);
    let (release, released) = bounded(1);
    let client = connect_with_connector(
        ConnectTarget::Local,
        ConnectOptions::default(),
        true,
        move |_, _| {
            entered.send(()).unwrap();
            released.recv_timeout(Duration::from_secs(3)).unwrap();
            panic!("cancelled fixture connector panic");
        },
    )
    .unwrap();
    started.recv_timeout(Duration::from_secs(3)).unwrap();
    client.handle.disconnect();
    release.send(()).unwrap();
    assert!(matches!(
        client.events.recv_timeout(Duration::from_secs(3)),
        Err(crossbeam_channel::RecvTimeoutError::Disconnected)
    ));
    assert!(client.handle.is_disconnected());
}
