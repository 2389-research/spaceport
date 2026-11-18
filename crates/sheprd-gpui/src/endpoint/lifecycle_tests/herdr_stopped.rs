// ABOUTME: Verifies intentional shutdown through an isolated Herdr transport.
// ABOUTME: Keeps shutdown advice distinct from ordinary connection loss.
use super::*;

#[test]
fn an_explicit_local_shutdown_waits_for_the_user() {
    let (mut endpoint, mut server) = connected_endpoint(LOCAL);
    // The transport stays on its fixture socket. No test dials a personal daemon.
    endpoint.connection.target = ConnectTarget::Local;
    write_message(
        &mut server.stream,
        &ServerMessage::ServerShutdown { reason: None },
        MAX_GRAPHICS_FRAME_SIZE,
    )
    .unwrap();
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.connection.retry_ready() && endpoint.outage().is_some()
    });
    assert_eq!(endpoint.status(), "Herdr stopped");
    assert!(
        !endpoint.may_dial(),
        "shutdown requires an explicit reconnect"
    );
    endpoint.connection = ConnectionBridge::new(endpoint.connection.target.clone());
    endpoint.poll(Instant::now());
    assert_eq!(endpoint.status(), "Herdr stopped");
    assert!(
        !endpoint.may_dial(),
        "fresh state preserves shutdown advice"
    );

    // A user reconnect can dial the fixture socket; never dial Local in a test.
    endpoint.connection.target = ConnectTarget::Socket(server.path.clone());
    endpoint.connect(ConnectOptions::default(), false);
    assert!(matches!(endpoint.link, Link::Dialing { .. }));
}

#[test]
fn ordinary_local_connection_loss_keeps_automatic_reconnect() {
    let (mut endpoint, server) = connected_endpoint(LOCAL);
    endpoint.connection.target = ConnectTarget::Local;
    server.stream.shutdown(std::net::Shutdown::Both).unwrap();
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.connection.retry_ready() && endpoint.outage().is_some()
    });
    assert!(!endpoint.herdr_stopped());
    assert!(endpoint.may_dial());
}

#[test]
fn an_explicit_socket_shutdown_remains_attach_only() {
    let (mut endpoint, mut server) = connected_endpoint(LOCAL);
    write_message(
        &mut server.stream,
        &ServerMessage::ServerShutdown { reason: None },
        MAX_GRAPHICS_FRAME_SIZE,
    )
    .unwrap();
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.connection.retry_ready() && endpoint.outage().is_some()
    });
    assert!(!endpoint.herdr_stopped());
    assert!(endpoint.may_dial());
}

#[test]
fn a_shutdown_waits_for_a_contended_event_reader() {
    let (mut endpoint, mut server) = connected_endpoint(LOCAL);
    endpoint.connection.target = ConnectTarget::Local;
    let inbox = endpoint.connection.inbox.clone();
    let guard = inbox.lock().unwrap();
    write_message(
        &mut server.stream,
        &ServerMessage::ServerShutdown { reason: None },
        MAX_GRAPHICS_FRAME_SIZE,
    )
    .unwrap();
    wait_until(|| {
        endpoint
            .connection
            .handle
            .as_ref()
            .unwrap()
            .is_disconnected()
    });
    endpoint.poll(Instant::now());
    assert!(
        !endpoint.may_dial(),
        "queued shutdown must land before any retry"
    );
    drop(guard);
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.herdr_stopped()
    });
    assert!(!endpoint.may_dial());
}

#[test]
fn remote_bridges_do_not_restart_an_explicitly_stopped_herdr() {
    for target in [
        ConnectTarget::Ssh {
            target: "fixture".into(),
            session: "default".into(),
        },
        ConnectTarget::Wsl {
            distro: "fixture".into(),
            session: "default".into(),
        },
    ] {
        let (mut endpoint, mut server) = connected_endpoint(LOCAL);
        // Keep the actual transport on the fixture; never execute SSH or WSL.
        endpoint.connection.target = target;
        write_message(
            &mut server.stream,
            &ServerMessage::ServerShutdown { reason: None },
            MAX_GRAPHICS_FRAME_SIZE,
        )
        .unwrap();
        wait_until(|| {
            endpoint.poll(Instant::now());
            endpoint.connection.retry_ready()
                && endpoint.connection.handle.is_none()
                && endpoint.outage().is_some()
        });
        assert_eq!(endpoint.status(), "Herdr stopped");
        assert!(!endpoint.may_dial());
    }
}
