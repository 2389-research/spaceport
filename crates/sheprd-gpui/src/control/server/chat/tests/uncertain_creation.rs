// ABOUTME: Reports malformed Herdr creation acknowledgments as uncertain outcomes.
// ABOUTME: Preserves known refusals and exercises the real control response boundary.
use super::*;
use std::sync::Arc;

#[test]
fn a_success_without_chat_ids_reports_unknown_outcome_and_inspection_guidance() {
    let (report, receipt) = std::sync::mpsc::sync_channel(1);
    report
        .send(Err(Arc::new(
            crate::chat::Error::from(crate::chat::StartError::Response).into(),
        )))
        .unwrap();
    assert_eq!(
        Pending::Herdr(receipt).poll("claude", 7),
        Some(unknown(ErrorCode::Failed))
    );
}

#[test]
fn an_explicit_herdr_refusal_keeps_its_known_failure() {
    let error: crate::Error = crate::chat::Error::from(crate::chat::StartError::Request(Arc::new(
        crate::Error::Client(sheprd_client::Error::Endpoint {
            code: "denied".to_owned().into(),
            message: "fixture refusal".into(),
        }),
    )))
    .into();
    let expected = Response::error(ErrorCode::Failed, error.to_string());
    assert_eq!(creation_error(&error), expected);
}

#[gpui::test]
fn malformed_success_on_the_daemon_wire_returns_unknown_over_the_control_socket(
    cx: &mut gpui::TestAppContext,
) {
    use crate::{
        control::{protocol::Request, socket},
        window::MockPeer,
    };
    use sheprd_client::protocol::{ClientMessage, ClientPaneInputEvent};
    for mut malformed in [
        serde_json::json!({"result":{"type":"tab_created","tab":{"tab_id":"w1:t9"}}}),
        serde_json::json!({"result":{"type":"tab_created","root_pane":{"pane_id":"w1:p9"}}}),
    ] {
        let mut peer = MockPeer::advertising(&["tab.create"]);
        let (view, cx) = cx.add_window_view(fixture_window);
        let (window_id, boot) = cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                peer.prepare_local_tab(view, cx);
                view.config.features.acp_chats = true;
                view.live.local_daemon_peer = true;
                view.endpoints[0].connection.target = sheprd_client::ConnectTarget::Local;
                view.endpoints[0].live = view.live.clone();
                view.chats.program = crate::chat::Program::Found("/fixture/sheprd-acp".into());
                (
                    window.window_handle().window_id().as_u64(),
                    view.live.snapshot.as_ref().unwrap().boot_id.clone(),
                )
            })
        });
        let address = socket::test_address();
        let server = socket::Server::bind(&address.path).unwrap();
        let path = address.path.clone();
        let mut request = call(ChatOp::New {
            agent: "claude".into(),
            pane: None,
        });
        request.caller.window = Some(window_id);
        let waiting = std::thread::spawn(move || socket::call(&path, &Request::Chat(request)));
        let incoming = server.next().unwrap();
        let mut state = crate::control::server::State::default();
        cx.cx
            .update(|cx| crate::control::server::serve(incoming, &mut state, cx));
        let request = peer.request();
        assert_eq!(request["method"], "tab.create");
        malformed["id"] = request["id"].clone();
        let response = peer.respond(&boot, request["id"].as_str().unwrap(), &malformed);
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                {
                    let mut inbox = view.endpoints[0].connection.inbox.lock().unwrap();
                    inbox.apply(response);
                    view.live = inbox.clone();
                }
                view.poll_chats(window, cx);
                assert!(!view.chat_starting());
            })
        });
        cx.cx.update(|cx| state.watches.serve(cx));
        assert_eq!(waiting.join().unwrap().unwrap(), unknown(ErrorCode::Failed));

        // Fence the real connection to prove no replay or launch input preceded completion.
        peer.client
            .handle
            .send_input(
                &boot,
                "w1:p1",
                [ClientPaneInputEvent::TextCommit("creation-fence".into())],
            )
            .unwrap();
        loop {
            match peer.receive() {
                ClientMessage::ClientShellPaneInput { events, .. } => {
                    assert_eq!(
                        events,
                        [ClientPaneInputEvent::TextCommit("creation-fence".into())]
                    );
                    break;
                }
                ClientMessage::ClientShellEndpointRequest { request, .. } => {
                    panic!("creation replayed after an uncertain success: {request}")
                }
                _ => {}
            }
        }
    }
}
