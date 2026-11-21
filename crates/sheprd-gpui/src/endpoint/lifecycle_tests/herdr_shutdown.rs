// ABOUTME: Preserves deliberate daemon shutdown through the real client event stream.
// ABOUTME: Checks retry policy without dialing local, session, SSH, or WSL targets.
use super::*;

const REASON: &str = "owned fixture intentionally stopped";

fn finish_peer(endpoint: &mut Endpoint, server: &mut Server, shutdown: bool) {
    assert!(endpoint.serving(), "the fixture must finish its handshake");
    if shutdown {
        write_message(
            &mut server.stream,
            &ServerMessage::ServerShutdown {
                reason: Some(REASON.into()),
            },
            MAX_FRAME_SIZE,
        )
        .unwrap();
    } else {
        server.stream.shutdown(std::net::Shutdown::Write).unwrap();
    }
    wait_until(|| endpoint.connection.drained.load(Ordering::Acquire));
    // Raw polling never starts transport, including when the policy identity changes.
    endpoint.poll(Instant::now());
    assert!(endpoint.connection.handle.is_none());
    assert!(!endpoint.live.status.is_connected());
    assert!(endpoint.live.snapshot.is_none());
    assert!(!endpoint.connection.inbox.lock().unwrap().dirty);
    assert!(endpoint.outage().is_some());
}

fn shutdown_holds_retry(target: ConnectTarget) {
    let (mut endpoint, mut server) = connected_endpoint(LOCAL);
    // The running worker already owns the original private Socket target.
    endpoint.connection.target = target;
    finish_peer(&mut endpoint, &mut server, true);
    assert_eq!(endpoint.outage(), Some(REASON));
    assert_eq!(endpoint.status(), "Herdr stopped");
    assert!(!endpoint.may_dial(), "shutdown requires explicit recovery");
}

#[test]
fn local_shutdown_requires_explicit_recovery() {
    shutdown_holds_retry(ConnectTarget::Local);
}

#[test]
fn named_session_shutdown_requires_explicit_recovery() {
    shutdown_holds_retry(ConnectTarget::Session {
        name: "owned-fixture".into(),
        development: false,
    });
}

#[test]
fn ssh_shutdown_requires_explicit_recovery() {
    shutdown_holds_retry(ConnectTarget::Ssh {
        target: "fixture@unused.invalid".into(),
        session: "owned-fixture".into(),
    });
}

#[test]
fn wsl_shutdown_requires_explicit_recovery() {
    shutdown_holds_retry(ConnectTarget::Wsl {
        distro: "unused-fixture".into(),
        session: "owned-fixture".into(),
    });
}

#[test]
fn explicit_socket_shutdown_keeps_attach_only_retry() {
    let (mut endpoint, mut server) = connected_endpoint(LOCAL);
    finish_peer(&mut endpoint, &mut server, true);
    assert_eq!(endpoint.outage(), Some(REASON));
    assert_eq!(endpoint.status(), "reconnecting");
    assert!(endpoint.may_dial());
}

#[test]
fn ordinary_local_transport_loss_keeps_retry() {
    let (mut endpoint, mut server) = connected_endpoint(LOCAL);
    endpoint.connection.target = ConnectTarget::Local;
    finish_peer(&mut endpoint, &mut server, false);
    assert_eq!(endpoint.status(), "reconnecting");
    assert!(endpoint.may_dial());
}

#[gpui::test]
fn explicit_reconnect_recovers_a_stopped_endpoint_through_a_new_handshake(
    cx: &mut gpui::TestAppContext,
) {
    use rustix::event::{PollFd, PollFlags, Timespec, poll};

    let (mut endpoint, mut old_server) = connected_endpoint(LOCAL);
    endpoint.connection.target = ConnectTarget::Local;
    finish_peer(&mut endpoint, &mut old_server, true);
    let stopped_status = endpoint.status();
    let stopped_may_dial = endpoint.may_dial();
    let stopped_reason = endpoint.outage().map(str::to_owned);

    let root = tempfile::Builder::new()
        .prefix("h")
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    let path = root.path().join("s");
    let listener = UnixListener::bind(&path).unwrap();
    listener.set_nonblocking(true).unwrap();
    // Repoint before the view exists. Even a failing baseline can dial only this socket.
    endpoint.connection.target = ConnectTarget::Socket(path.clone());
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    view.update(cx, |view, cx| {
        view.endpoints = vec![endpoint];
        view.selected_endpoint = 0;
        view.options = ConnectOptions::default();
        view.reconnect(cx);
    });

    let mut watched = [PollFd::new(&listener, PollFlags::IN)];
    let timeout = Timespec {
        tv_sec: 3,
        tv_nsec: 0,
    };
    assert_eq!(poll(&mut watched, Some(&timeout)).unwrap(), 1);
    assert!(watched[0].revents().contains(PollFlags::IN));
    let mut new_server = Server::accepted(listener.accept().unwrap().0, path);
    let mut replacement = snapshot();
    replacement.boot_id = "explicit-recovery-boot".into();
    welcome_peer(&mut new_server, &replacement, None);

    view.update(cx, |view, _| {
        let endpoint = &mut view.endpoints[0];
        wait_until(|| {
            endpoint.poll(Instant::now());
            endpoint.serving() || endpoint.connection.drained.load(Ordering::Acquire)
        });
        assert!(endpoint.serving(), "the replacement handshake must succeed");
        assert_eq!(endpoint.status(), "online");
        assert_eq!(
            endpoint.live.snapshot.as_ref().unwrap().boot_id,
            replacement.boot_id
        );
        assert_eq!(endpoint.outage(), None);
    });
    // First prove recovery, then require the earlier deliberate-stop policy.
    assert_eq!(stopped_reason.as_deref(), Some(REASON));
    assert_eq!(stopped_status, "Herdr stopped");
    assert!(!stopped_may_dial);
}
