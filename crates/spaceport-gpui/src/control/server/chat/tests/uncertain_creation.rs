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
        spaceport_client::Error::Endpoint {
            code: "denied".to_owned().into(),
            message: "fixture refusal".into(),
        },
    )))
    .into();
    let expected = Response::error(ErrorCode::Failed, error.to_string());
    assert_eq!(creation_error(&error), expected);
}

#[test]
fn a_lost_answer_to_a_submitted_start_reports_an_unknown_outcome() {
    for transport in [
        spaceport_client::Error::RequestTimeout,
        spaceport_client::Error::SocketClosed,
    ] {
        let error: crate::Error =
            crate::chat::Error::from(crate::chat::StartError::Request(Arc::new(transport))).into();
        assert_eq!(creation_error(&error), unknown(ErrorCode::Failed));
    }
}

#[test]
fn a_transport_failure_after_the_tab_was_made_reports_an_unknown_outcome() {
    for transport in [
        spaceport_client::Error::RequestTimeout,
        spaceport_client::Error::Full,
    ] {
        let error: crate::Error = crate::Error::Client(transport);
        assert_eq!(creation_error(&error), unknown(ErrorCode::Failed));
    }
}

#[gpui::test]
fn malformed_success_on_the_daemon_wire_returns_unknown_over_the_control_socket(
    cx: &mut gpui::TestAppContext,
) {
    use crate::{
        control::{protocol::Request, socket},
        window::MockPeer,
    };
    use spaceport_client::protocol::{ClientMessage, ClientPaneInputEvent};
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
                view.endpoints[0].connection.target = spaceport_client::ConnectTarget::Local;
                *view.endpoints[0].live_mut() = view.live.clone();
                view.chats.program = crate::chat::Program::Found("/fixture/spaceport-acp".into());
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
                    inbox.apply(response, crate::state::describe, crate::state::localize);
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

#[test]
fn lost_creation_receipts_and_timeouts_never_claim_success() {
    let (report, receipt) = std::sync::mpsc::sync_channel(1);
    drop(report);
    let response = Pending::Herdr(receipt).poll("claude", 7).unwrap();
    assert_eq!(code(&response), Some(ErrorCode::Failed));
    assert!(
        matches!(response, Response::Error { message, .. } if message.contains("unknown") && message.contains("before retrying"))
    );
    let timeout = crate::Error::from(crate::chat::Error::from(crate::chat::StartError::Timeout));
    assert_eq!(code(&creation_error(&timeout)), Some(ErrorCode::Timeout));
    assert_eq!(
        creation_error(&crate::Error::StaleConnection),
        unknown(ErrorCode::Failed)
    );
}

fn core_failure(error: crate::core_terminals::Error) -> Response {
    let (report, receipt) = std::sync::mpsc::sync_channel(1);
    report.send(Err(error)).unwrap();
    Pending::Core(receipt).poll("claude", 7).unwrap()
}

#[test]
fn core_creation_errors_preserve_timeout_and_unknown_outcome_classifications() {
    use spaceport_core_client::{Error, Method};
    assert_eq!(
        core_failure(Error::Timeout(Method::TerminalRun).into()),
        unknown(ErrorCode::Timeout)
    );
    for error in [
        Error::Lost,
        Error::Invalid(
            Method::TerminalRun,
            serde_json::from_str::<serde_json::Value>("{").unwrap_err(),
        ),
        Error::Backend(spaceport_backend::Error::EmptyTerminalId),
    ] {
        assert_eq!(core_failure(error.into()), unknown(ErrorCode::Failed));
    }
}

#[test]
fn known_core_refusals_worker_failures_and_unplaced_ids_keep_their_diagnostics() {
    for error in [
        crate::core_terminals::Error::Core(spaceport_core_client::Error::Refused {
            method: spaceport_core_client::Method::TerminalRun,
            kind: None,
            message: "fixture refusal".into(),
            hint: None,
        }),
        crate::core_terminals::Error::Worker(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "fixture worker refusal",
        )),
        crate::core_terminals::Error::ViewMissing(
            spaceport_backend::TerminalId::new("created-but-unplaced").unwrap(),
        ),
    ] {
        let expected = Response::error(ErrorCode::Failed, error.to_string());
        assert_eq!(core_failure(error), expected);
    }
}
