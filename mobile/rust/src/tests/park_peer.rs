// ABOUTME: Observes focus and surface transitions through the real ordered client worker.
// ABOUTME: Uses a private socket pair and the protocol's published handshake fixture.
use super::*;
use serde_json::{Value, json};
use spaceport_client::{ConnectTarget, Stream, connect_with_connector};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

fn send(peer: &mut Stream, message: ServerMessage) {
    write_message(peer, &message, MAX_GRAPHICS_FRAME_SIZE).unwrap();
}

#[track_caller]
fn read(peer: &mut Stream) -> ClientMessage {
    read_message(peer, MAX_FRAME_SIZE).unwrap()
}

fn request(peer: &mut Stream, active: bool) -> String {
    let ClientMessage::ClientShellEndpointRequest { request, .. } = read(peer) else {
        panic!("expected a surface-interest request")
    };
    let body: Value = serde_json::from_str(&request).unwrap();
    assert_eq!(body["method"], "client_shell.surface.set");
    assert_eq!(body["params"]["active"], active);
    body["id"].as_str().unwrap().into()
}

fn reply(peer: &mut Stream, id: &str, result: Value) {
    send(
        peer,
        ServerMessage::ClientShellEndpointResponseChunk {
            boot_id: "boot-v1".into(),
            request_id: id.into(),
            final_chunk: true,
            data: serde_json::to_vec(&json!({"id":id, "result":result})).unwrap(),
        },
    );
}

fn interest(active: bool) -> Value {
    json!({"type":"client_shell_surface_set", "active":active, "projection_revision":7})
}

fn peer() -> (Arc<MobileSession>, Stream) {
    peer_with_surface(surface())
}

fn peer_with_surface(frame: PaneSurfaceFrame) -> (Arc<MobileSession>, Stream) {
    let options = geometry(frame.frame.width, frame.frame.height, 8, 16).unwrap();
    let (stream, mut peer) = Stream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let client = connect_with_connector(
        ConnectTarget::Socket("private-pair".into()),
        options,
        true,
        move |_, _| Ok(stream),
    )
    .unwrap();
    let session = MobileSession::from_client(
        client,
        Arc::new(Mutex::new(Projection::new(options))),
        None,
        Arc::default(),
    )
    .unwrap();
    assert!(matches!(
        read(&mut peer),
        ClientMessage::EndpointControl { .. }
    ));
    let mut welcome: Value = serde_json::from_str(include_str!(
        "../../../../crates/spaceport-protocol/tests/fixtures/endpoint-welcome-v1.json"
    ))
    .unwrap();
    welcome["server_version"] = json!("0.9.5");
    welcome["methods"] = json!([
        "client_shell.surface.set",
        "pane.copy_motion",
        "pane.copy_search",
        "pane.selection.read"
    ]);
    welcome["capabilities"] = json!([
        "surface_interest",
        "presentation_effects_fence",
        "not-for-the-clipboard"
    ]);
    send(
        &mut peer,
        ServerMessage::EndpointControl {
            kind: endpoint::ENDPOINT_WELCOME_KIND.into(),
            data: welcome.to_string(),
        },
    );
    send(
        &mut peer,
        ServerMessage::EndpointControl {
            kind: endpoint::ENDPOINT_SNAPSHOT_KIND.into(),
            data: serde_json::to_string(&snapshot()).unwrap(),
        },
    );
    send(&mut peer, ServerMessage::PaneSurface(frame));
    assert!(matches!(
        read(&mut peer),
        ClientMessage::ClientShellFocus { focused: true }
    ));
    (session, peer)
}

#[test]
fn diagnostic_capabilities_contain_only_known_handshake_fields() {
    let (session, _socket) = peer();
    assert_eq!(
        session.diagnostic_capabilities().as_deref(),
        Some("presentation_effects_fence, surface_interest")
    );
}

#[test]
fn parking_releases_focus_and_surface_then_refocuses_after_verified_new_frame() {
    let (session, mut peer) = peer();
    session.park();
    assert!(matches!(
        read(&mut peer),
        ClientMessage::ClientShellFocus { focused: false }
    ));
    let release = request(&mut peer, false);
    // Resume can race a still-unacknowledged release. The worker keeps FIFO order.
    let activation = session.begin_resume("boot-v1".into()).unwrap();
    let mut old = surface();
    old.surface_revision += 1;
    send(&mut peer, ServerMessage::PaneSurface(old));
    reply(&mut peer, &release, interest(false));
    assert_eq!(request(&mut peer, true), activation);
    reply(&mut peer, &activation, interest(true));
    let mut next = surface();
    next.surface_revision += 2;
    send(&mut peer, ServerMessage::PaneSurface(next));
    let deadline = Instant::now() + Duration::from_secs(2);
    while !session
        .check_resume("boot-v1".into(), activation.clone())
        .unwrap()
    {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert!(matches!(
        read(&mut peer),
        ClientMessage::ClientShellFocus { focused: true }
    ));
    assert!(session.poll(0).unwrap().input_target.is_some());
}

#[test]
fn parked_roster_waits_for_the_acknowledged_snapshot_without_claiming_focus() {
    let (session, mut peer) = peer();
    session.park();
    assert!(matches!(
        read(&mut peer),
        ClientMessage::ClientShellFocus { focused: false }
    ));
    let release = request(&mut peer, false);
    let observation = session.begin_parked_roster("boot-v1".into()).unwrap();
    reply(&mut peer, &release, interest(false));
    assert_eq!(request(&mut peer, false), observation);
    let mut acknowledged = interest(false);
    acknowledged["projection_revision"] = json!(8);
    reply(&mut peer, &observation, acknowledged);
    let mut latest = snapshot();
    latest.revision = 8;
    latest.workspaces[0].label = "observed while parked".into();
    send(
        &mut peer,
        ServerMessage::EndpointControl {
            kind: endpoint::ENDPOINT_SNAPSHOT_KIND.into(),
            data: serde_json::to_string(&latest).unwrap(),
        },
    );
    let view = session
        .wait_parked_roster("boot-v1".into(), observation.clone())
        .unwrap();
    assert_eq!(view.workspaces[0].label, "observed while parked");
    assert!(view.terminal.is_none() && view.input_target.is_none());
    assert!(matches!(
        session.wait_parked_roster("boot-v1".into(), observation),
        Err(MobileError::StaleView)
    ));
    let activation = session.begin_resume("boot-v1".into()).unwrap();
    assert_eq!(request(&mut peer, true), activation);
}

#[test]
fn cancelling_a_parked_roster_check_keeps_normal_resume_available() {
    let (session, mut peer) = peer();
    session.park();
    assert!(matches!(
        read(&mut peer),
        ClientMessage::ClientShellFocus { focused: false }
    ));
    let release = request(&mut peer, false);
    let observation = session.begin_parked_roster("boot-v1".into()).unwrap();
    reply(&mut peer, &release, interest(false));
    assert_eq!(request(&mut peer, false), observation);
    session.cancel_parked_roster(observation.clone());
    assert!(matches!(
        session.wait_parked_roster("boot-v1".into(), observation.clone()),
        Err(MobileError::StaleView)
    ));
    let activation = session.begin_resume("boot-v1".into()).unwrap();
    reply(&mut peer, &observation, json!({}));
    assert_eq!(request(&mut peer, true), activation);
    assert!(matches!(
        session.begin_parked_roster("boot-v1".into()),
        Err(MobileError::StaleView)
    ));
}

#[test]
fn refusal_to_release_disconnects_the_hidden_client() {
    let (session, mut peer) = peer();
    session.park();
    assert!(matches!(
        read(&mut peer),
        ClientMessage::ClientShellFocus { focused: false }
    ));
    let release = request(&mut peer, false);
    reply(&mut peer, &release, interest(true));
    let deadline = Instant::now() + Duration::from_secs(2);
    while !session.poll(0).unwrap().disconnected {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
}

#[test]
fn an_established_peer_eof_is_distinct_from_a_protocol_failure_or_local_close() {
    fn wait(session: &MobileSession) {
        let until = Instant::now() + Duration::from_secs(2);
        while !session.poll(0).unwrap().disconnected {
            assert!(Instant::now() < until);
            std::thread::yield_now();
        }
    }
    let (session, socket) = peer();
    drop(socket);
    wait(&session);
    assert!(session.transport_lost());

    let (session, mut socket) = peer();
    send(&mut socket, ServerMessage::PaneSurface(surface())); // duplicate surface revision
    wait(&session);
    assert!(!session.transport_lost());

    let (session, _socket) = peer();
    session.disconnect();
    assert!(!session.transport_lost());
}

#[test]
fn losing_a_peer_with_an_outstanding_focus_fence_requires_explicit_reselection() {
    let (session, socket) = peer();
    assert!(!session.focus_interrupted());
    let pane = session.poll(0).unwrap().focused_pane.unwrap();
    session.focus_pane("boot-v1".into(), pane).unwrap();
    session.park();
    drop(socket);
    let deadline = Instant::now() + Duration::from_secs(2);
    while !session.poll(0).unwrap().disconnected {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert!(session.focus_interrupted());
    assert!(session.poll(0).unwrap().input_target.is_none());
}

#[path = "history_peer.rs"]
mod history;
