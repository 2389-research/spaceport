// ABOUTME: Verifies poisoned connection state remains observable as disconnected.
// ABOUTME: Keeps polling nonblocking and old inboxes fenced after replacement.
use super::*;

fn poison(inbox: &Arc<Mutex<LiveState>>) {
    let inbox = inbox.clone();
    assert!(
        std::thread::spawn(move || {
            let mut state = inbox.lock().unwrap();
            state.status = ConnectionStatus::Connected;
            state.dirty = false;
            panic!("fixture reducer panic");
        })
        .join()
        .is_err()
    );
}

#[test]
fn poisoned_inbox_delivers_one_disconnected_update() {
    let bridge = bridge();
    let sound_cancel = bridge.sound_cancel.clone();
    poison(&bridge.inbox);
    assert!(bridge.has_update(), "poison must wake the window");
    assert_eq!(bridge.reports_connected(), Some(false));
    assert!(!bridge.retry_ready(), "consume failure before retrying");
    let update = bridge.take_update().unwrap();
    assert_eq!(update.status, ConnectionStatus::Disconnected);
    assert!(update.notices.is_some());
    assert!(sound_cancel.load(Ordering::Acquire));
    assert!(!bridge.inbox.is_poisoned());
    assert!(bridge.take_update().is_none());
    assert!(bridge.retry_ready());
}

#[test]
fn poisoned_old_inbox_cannot_change_a_replacement() {
    let mut bridge = bridge();
    let old = bridge.inbox.clone();
    bridge.detach(false);
    poison(&old);
    assert_eq!(
        bridge.take_update().unwrap().status,
        ConnectionStatus::Detached
    );
    assert_eq!(bridge.reports_connected(), Some(false));
    assert!(bridge.take_update().is_none());
}

#[test]
fn poison_does_not_erase_an_explicit_shutdown() {
    let bridge = bridge();
    bridge.inbox.lock().unwrap().apply(
        ClientEvent::Message(spaceport_client::protocol::ServerMessage::ServerShutdown {
            reason: None,
        }),
        &crate::state::labels(),
    );
    poison(&bridge.inbox);
    assert!(bridge.take_update().unwrap().server_stopped);
}

fn reader(bridge: &ConnectionBridge) -> reader::Reader {
    // The fixture creates only a worker; it never discovers a personal daemon.
    let client = connect_with_connector(
        ConnectTarget::Socket("/unused-reader-test.sock".into()),
        ConnectOptions::default(),
        true,
        |_, _| Err(std::io::Error::other("fixture transport closed")),
    )
    .unwrap();
    reader_for(bridge, client.handle)
}

fn reader_for(bridge: &ConnectionBridge, handle: ClientHandle) -> reader::Reader {
    bridge.drained.store(false, Ordering::Release);
    reader::Reader {
        drained: bridge.drained.clone(),
        handle,
        inbox: bridge.inbox.clone(),
        integrations: bridge.integrations.clone(),
        links: bridge.links.clone(),
        scrollback: bridge.scrollback.clone(),
        #[cfg(unix)]
        wake: applied::wake(),
    }
}

#[test]
fn reader_end_reports_disconnect_and_fails_pending_answers() {
    let bridge = bridge();
    bridge.inbox.lock().unwrap().status = ConnectionStatus::Connected;
    bridge.integrations.lock().unwrap().pending = Some(("pending".into(), None));
    bridge
        .scrollback
        .lock()
        .unwrap()
        .send(|| Ok("scroll".into()))
        .unwrap();
    reader(&bridge).read([]);
    assert!(bridge.drained.load(Ordering::Acquire));
    assert!(!bridge.retry_ready());
    let update = bridge.take_update().unwrap();
    assert_eq!(update.status, ConnectionStatus::Disconnected);
    assert_eq!(update.notices.text(), Some("connection lost"));
    assert!(matches!(
        bridge
            .integrations
            .lock()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .1,
        Some(Err(crate::Error::NotConnected))
    ));
    assert!(matches!(
        bridge.scrollback.lock().unwrap().take("scroll"),
        Some(Err(spaceport_client::Error::Disconnected))
    ));
    assert!(bridge.retry_ready());
}

#[test]
fn reader_unwind_recovers_poison_and_marks_drained_last() {
    let bridge = bridge();
    let (release, released) = std::sync::mpsc::channel();
    let client = connect_with_connector(
        ConnectTarget::Socket("/unused-panicking-reader.sock".into()),
        ConnectOptions::default(),
        true,
        move |_, _| {
            released
                .recv_timeout(std::time::Duration::from_secs(3))
                .unwrap();
            Err(std::io::Error::other("fixture transport closed"))
        },
    )
    .unwrap();
    let handle = client.handle.clone();
    assert!(!handle.is_disconnected(), "the worker is still running");
    let reader = reader_for(&bridge, client.handle);
    let inbox = bridge.inbox.clone();
    let events = std::iter::from_fn(move || -> Option<ClientEvent> {
        let mut state = inbox.lock().unwrap();
        state.status = ConnectionStatus::Connected;
        state.dirty = false;
        panic!("fixture reducer panic");
    });
    assert!(
        std::thread::spawn(move || reader.read(events))
            .join()
            .is_err()
    );
    assert!(bridge.drained.load(Ordering::Acquire));
    assert!(handle.is_disconnected());
    assert!(!bridge.inbox.is_poisoned());
    assert_eq!(
        bridge.take_update().unwrap().status,
        ConnectionStatus::Disconnected
    );
    release.send(()).unwrap();
    assert!(
        client
            .events
            .recv_timeout(std::time::Duration::from_secs(3))
            .unwrap_err()
            .is_disconnected()
    );
}

#[test]
fn reader_end_preserves_the_ordered_disconnect_reason() {
    let bridge = bridge();
    reader(&bridge).read([
        ClientEvent::Message(spaceport_client::protocol::ServerMessage::ServerShutdown {
            reason: None,
        }),
        ClientEvent::Disconnected {
            reason: "intentional stop".into(),
            ssh: None,
        },
    ]);
    let update = bridge.take_update().unwrap();
    assert!(update.server_stopped);
    assert_eq!(update.notices.text(), Some("intentional stop"));
    assert_eq!(update.status, ConnectionStatus::Disconnected);
}

#[test]
fn queued_welcome_cannot_revive_a_poisoned_generation() {
    let bridge = bridge();
    let reader = reader(&bridge);
    poison(&bridge.inbox);
    assert!(bridge.has_update());
    let welcome = serde_json::from_str(include_str!(
        "../../../../spaceport-protocol/tests/fixtures/endpoint-welcome-v1.json"
    ))
    .unwrap();
    reader.read([ClientEvent::Connected(welcome)]);
    assert_eq!(
        bridge.take_update().unwrap().status,
        ConnectionStatus::Disconnected
    );
    assert!(bridge.retry_ready());
}

#[test]
fn reader_end_waits_for_inbox_before_marking_drained() {
    let bridge = bridge();
    let reader = reader(&bridge);
    let mut held = bridge.inbox.lock().unwrap();
    held.status = ConnectionStatus::Connected;
    held.dirty = false;
    let (finished, done) = std::sync::mpsc::channel();
    let task = std::thread::spawn(move || {
        reader.read([]);
        finished.send(()).unwrap();
    });
    assert!(!bridge.drained.load(Ordering::Acquire));
    assert!(!bridge.has_update());
    assert_eq!(bridge.reports_connected(), None);
    assert!(bridge.take_update().is_none());
    assert!(!bridge.retry_ready());
    drop(held);
    done.recv_timeout(std::time::Duration::from_secs(3))
        .unwrap();
    task.join().unwrap();
    assert!(bridge.drained.load(Ordering::Acquire));
    assert_eq!(
        bridge.take_update().unwrap().status,
        ConnectionStatus::Disconnected
    );
}

#[test]
fn retired_reader_cannot_disconnect_the_replacement() {
    let mut bridge = bridge();
    let reader = reader(&bridge);
    bridge.detach(false);
    reader.read([ClientEvent::Disconnected {
        reason: "old reader failure".into(),
        ssh: None,
    }]);
    let update = bridge.take_update().unwrap();
    assert_eq!(update.status, ConnectionStatus::Detached);
    assert!(update.notices.is_none());
}

#[cfg(unix)]
#[test]
fn reader_preserves_shutdown_order_through_a_real_socket() {
    use spaceport_client::protocol::{endpoint::*, *};
    use std::time::Duration;

    let bridge = bridge();
    let (stream, mut peer) = spaceport_client::Stream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    peer.set_write_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let client = connect_with_connector(
        ConnectTarget::Socket("/unused-paired-socket.sock".into()),
        ConnectOptions::default(),
        true,
        move |_, _| Ok(stream),
    )
    .unwrap();
    let reader = reader_for(&bridge, client.handle.clone());
    let (finished, done) = std::sync::mpsc::channel();
    let task = std::thread::spawn(move || {
        reader.read(&client.events);
        finished.send(()).unwrap();
    });
    assert!(matches!(
        read_message::<_, ClientMessage>(&mut peer, MAX_FRAME_SIZE).unwrap(),
        ClientMessage::EndpointControl { .. }
    ));
    for (kind, data) in [
        (
            ENDPOINT_WELCOME_KIND,
            include_str!("../../../../spaceport-protocol/tests/fixtures/endpoint-welcome-v1.json"),
        ),
        (
            ENDPOINT_SNAPSHOT_KIND,
            include_str!("../../../../spaceport-protocol/tests/fixtures/endpoint-snapshot-v1.json"),
        ),
    ] {
        write_message(
            &mut peer,
            &ServerMessage::EndpointControl {
                kind: kind.into(),
                data: data.into(),
            },
            MAX_GRAPHICS_FRAME_SIZE,
        )
        .unwrap();
    }
    write_message(
        &mut peer,
        &ServerMessage::ServerShutdown { reason: None },
        MAX_GRAPHICS_FRAME_SIZE,
    )
    .unwrap();
    done.recv_timeout(Duration::from_secs(3)).unwrap();
    task.join().unwrap();
    assert!(bridge.drained.load(Ordering::Acquire));
    let update = bridge.take_update().unwrap();
    assert_eq!(update.status, ConnectionStatus::Disconnected);
    assert!(update.server_stopped);
    assert!(update.snapshot.is_none());
    assert!(update.notices.is_some());
}
