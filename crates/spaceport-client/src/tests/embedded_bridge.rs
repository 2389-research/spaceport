// ABOUTME: Exercises the authenticated embedded SSH boundary using a socket pair.
// ABOUTME: Checks compatibility negotiation and the remote health requirement.
use super::*;

#[test]
fn embedded_bridge_negotiates_then_refuses_daemon_without_health_checks() {
    let (stream, mut peer) = Stream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let (guard, released) = std::sync::mpsc::channel::<()>();
    let (selected, path) = std::sync::mpsc::channel();
    let client = connect_embedded_bridge_with_path(
        "phone",
        ConnectOptions::default(),
        move |_, _| Ok((stream, guard)),
        move |accepted| selected.send(accepted).unwrap(),
    )
    .unwrap();
    peer.write_all(b"{}\nherdr-bridge-path:/old/herdr\nherdr-remote-output-ready:1\n")
        .unwrap();
    let mut skip = [0; 5];
    peer.read_exact(&mut skip).unwrap();
    assert_eq!(&skip, b"skip\n");
    peer.write_all(b"{\"endpoint_protocol_generation\":1,\"endpoint_capabilities\":[\"surface_interest\",\"presentation_effects_fence\",\"health_check\"]}\nherdr-bridge-path:/install dir/herdr\nherdr-remote-output-ready:1\n").unwrap();
    let mut choice = [0; 7];
    peer.read_exact(&mut choice).unwrap();
    assert_eq!(&choice, b"accept\n");
    assert_eq!(
        path.recv_timeout(Duration::from_secs(1)).unwrap(),
        "/install dir/herdr"
    );
    assert!(matches!(
        receive(&mut peer),
        ClientMessage::EndpointControl { .. }
    ));
    assert!(matches!(
        released.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty)
    ));
    let mut welcome: Value = serde_json::from_str(WELCOME).unwrap();
    welcome["methods"] = json!(["client_shell.surface.set"]);
    welcome["capabilities"] = json!(["surface_interest", "presentation_effects_fence"]);
    send(
        &mut peer,
        ServerMessage::EndpointControl {
            kind: ENDPOINT_WELCOME_KIND.into(),
            data: welcome.to_string(),
        },
    );
    assert!(matches!(event(&client), ClientEvent::VersionMismatch(_)));
    assert!(matches!(
        released.recv_timeout(Duration::from_secs(1)),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected)
    ));
    let ClientEvent::Disconnected { reason, .. } = event(&client) else {
        panic!("disconnect");
    };
    assert_eq!(
        reason,
        Error::MissingHealthCheck {
            server_version: "0.8.2".into()
        }
        .to_string()
    );
}

#[test]
fn roster_bridge_requests_an_inactive_surface_at_hello() {
    let (stream, mut peer) = Stream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let client = observe_embedded_bridge("phone", move |_, _| Ok((stream, ()))).unwrap();
    observer_banner(&mut peer, true);
    let socket = b"/private/session/herdr-client.sock\n";
    let mut selected = vec![0; socket.len()];
    peer.read_exact(&mut selected).unwrap();
    assert_eq!(&selected, socket);
    let ClientMessage::EndpointControl { data, .. } = receive(&mut peer) else {
        panic!("hello")
    };
    let hello: EndpointClientHello = serde_json::from_str(&data).unwrap();
    client.handle.disconnect();
    assert!(!hello.surface_active);
}

#[test]
fn roster_bridge_refuses_stopped_and_missing_sessions_before_accept() {
    let (stream, mut peer) = Stream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let client = observe_embedded_bridge("phone", move |_, _| Ok((stream, ()))).unwrap();
    observer_banner(&mut peer, false);
    let ClientEvent::Disconnected { reason, .. } = event(&client) else {
        panic!("disconnect")
    };
    assert_eq!(reason, sessions::Error::NotRunning.to_string());
    assert_eq!(peer.read(&mut [0; 1]).unwrap(), 0);
}

fn observer_banner(peer: &mut Stream, running: bool) {
    let client = json!({"endpoint_protocol_generation":1,"endpoint_capabilities":["surface_interest","presentation_effects_fence","health_check"]});
    let server = json!({"session":"phone","running":running,"endpoint_compatible":true,"socket":"/private/session/herdr.sock"});
    writeln!(
        peer,
        "{client}\nherdr-observer:{server}\nherdr-remote-output-ready:1"
    )
    .unwrap();
}
