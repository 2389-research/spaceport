// ABOUTME: Checks intentional shutdown messages from an isolated real Herdr session.
// ABOUTME: Stops and reaps only the exact named session created by this test.
use super::*;
use sheprd_client::protocol::ServerMessage;

#[test]
#[ignore = "requires explicit HERDR_TEST_BINARY; creates an isolated named session"]
fn named_session_shutdown_arrives_before_disconnect() {
    let binary = daemon_binary();
    let mut daemon = Daemon {
        sandbox: Sandbox::new(),
        child: None,
    };
    let name = "shutdown-test";
    daemon.child = Some(
        daemon
            .sandbox
            .command(&binary, "daemon.log")
            .args(["--session", name, "server"])
            .spawn()
            .unwrap(),
    );
    let socket = daemon
        .sandbox
        .dir
        .join("config/herdr/sessions")
        .join(name)
        .join("herdr-client.sock");
    let deadline = Instant::now() + TIMEOUT;
    while sheprd_client::Stream::connect(&socket).is_err() {
        assert!(daemon.child.as_mut().unwrap().try_wait().unwrap().is_none());
        assert!(Instant::now() < deadline, "named session did not start");
        thread::sleep(Duration::from_millis(20));
    }
    let client = connect(ConnectTarget::Socket(socket), ConnectOptions::default()).unwrap();
    loop {
        let event = client
            .events
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap();
        if matches!(event, ClientEvent::Connected(_)) {
            break;
        }
        assert!(!matches!(event, ClientEvent::Disconnected { .. }));
    }
    assert!(
        daemon
            .sandbox
            .command(&binary, "stop.log")
            .args(["session", "stop", "--json", "--", name])
            .status()
            .unwrap()
            .success()
    );
    let mut shutdown = false;
    loop {
        match client
            .events
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap()
        {
            ClientEvent::Message(ServerMessage::ServerShutdown { .. }) => shutdown = true,
            ClientEvent::Disconnected { .. } => {
                assert!(shutdown, "explicit shutdown must precede disconnect");
                break;
            }
            _ => {}
        }
    }
    while daemon.child.as_mut().unwrap().try_wait().unwrap().is_none() {
        assert!(Instant::now() < deadline, "named session did not stop");
        thread::sleep(Duration::from_millis(20));
    }
    daemon.child = None;
}
