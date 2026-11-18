// ABOUTME: Checks chat control requests and exact agent or terminal targets.
// ABOUTME: Opts into Core for the Core group case while retaining legacy request tests.
#![allow(clippy::unwrap_used)]
use super::*;
mod creation;
use crate::{
    browser::CoreGroupKey,
    control::protocol::{Caller, chat::Reply},
    core_terminals::set_document_for_test_as,
    sidebar::layout_tests::fixture_window,
};

#[gpui::test]
fn core_group_without_terminal_refuses_untargeted_legacy_chat(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| crate::core_feature::install(true, cx));
    let group = uuid::Uuid::from_u128(71);
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.cx.update(|cx| {
        set_document_for_test_as(cx, "hub-a", sheprd_core_client::wire::Doc {
            value: serde_json::json!({"schema_version":1,"workspaces":[{"id":group,"label":"One"}]}),
            version: 3,
        });
        let key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
        view.update(cx, |view, cx| assert!(view.show_core_group(&key, cx)));
        let refused = serve(call(ChatOp::New { agent: "claude".into(), pane: None }), cx);
        assert!(matches!(refused, Response::Error { code: ErrorCode::Unsupported, .. }));
    });
}

fn call(op: ChatOp) -> Call<ChatOp> {
    Call {
        caller: Caller::default(),
        op,
    }
}

fn serve(call: Call<ChatOp>, cx: &mut App) -> Response {
    let address = crate::control::socket::test_address();
    let server = crate::control::socket::Server::bind(&address.path).unwrap();
    let path = address.path.clone();
    let result = std::thread::spawn(move || {
        crate::control::socket::call(&path, &crate::control::protocol::Request::Chat(call))
    });
    let incoming = server.next().unwrap();
    let mut state = crate::control::server::State::default();
    crate::control::server::serve(incoming, &mut state, cx);
    assert!(
        state.watches.0.is_empty(),
        "fixture expects an immediate refusal or observation"
    );
    result.join().unwrap().unwrap()
}

fn code(response: &Response) -> Option<ErrorCode> {
    match response {
        Response::Error { code, .. } => Some(*code),
        _ => None,
    }
}

#[gpui::test]
fn chat_requests_need_a_window(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        assert_eq!(
            code(&serve(call(ChatOp::Status), cx)),
            Some(ErrorCode::NoWindow)
        );
    });
}

#[gpui::test]
fn a_window_without_chats_lists_none_and_shows_none(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = cx.add_window_view(fixture_window);
    cx.cx.update(|cx| {
        assert_eq!(
            serve(call(ChatOp::Status), cx),
            Response::Chat {
                reply: Reply::Chats { chats: Vec::new() }
            }
        );
        let show = serve(
            call(ChatOp::View {
                switch: crate::control::protocol::app::Switch::On,
                pane: Some("w1:p1".into()),
            }),
            cx,
        );
        assert_eq!(code(&show), Some(ErrorCode::NotFound));
    });
}

#[gpui::test]
fn a_new_chat_names_a_known_agent_and_needs_herdr_acp(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.cx.update(|cx| {
        let unknown = serve(
            call(ChatOp::New {
                agent: "cursor".into(),
                pane: None,
            }),
            cx,
        );
        assert_eq!(code(&unknown), Some(ErrorCode::InvalidRequest));
    });
    view.update(cx, |view, _| {
        view.chats.program = crate::chat::Program::Missing;
    });
    cx.cx.update(|cx| {
        let missing = serve(
            call(ChatOp::New {
                agent: "claude".into(),
                pane: None,
            }),
            cx,
        );
        assert_eq!(code(&missing), Some(ErrorCode::Unsupported));
    });
}

#[gpui::test]
fn creation_never_falls_back_from_the_named_window_to_another_panes_window(
    cx: &mut gpui::TestAppContext,
) {
    let first = cx.add_window(fixture_window);
    let second = cx.add_window(fixture_window);
    let peer = crate::window::MockPeer::advertising(&["tab.create"]);
    cx.update(|cx| {
        first
            .update(cx, |view, _, _| view.live.snapshot = None)
            .unwrap();
        second
            .update(cx, |view, _, cx| peer.prepare_local_tab(view, cx))
            .unwrap();
        let response = serve(
            Call {
                caller: Caller {
                    window: Some(first.window_id().as_u64()),
                    ..Caller::default()
                },
                op: ChatOp::New {
                    agent: "claude".into(),
                    pane: Some("w1:p1".into()),
                },
            },
            cx,
        );
        assert_eq!(code(&response), Some(ErrorCode::NotFound));
        for handle in [first, second] {
            assert!(
                !handle
                    .read_with(cx, |view, _| view.chat_starting())
                    .unwrap()
            );
        }
    });
}

#[test]
fn lost_creation_receipts_and_timeouts_never_claim_success() {
    let (report, receipt) = std::sync::mpsc::sync_channel(1);
    drop(report);
    let response = Pending::Herdr(receipt).poll("claude", 7).unwrap();
    assert_eq!(code(&response), Some(ErrorCode::Failed));
    assert!(
        matches!(response, Response::Error {message, ..} if message.contains("unknown") && message.contains("before retrying"))
    );
    let timeout = crate::Error::from(crate::chat::Error::from(crate::chat::StartError::Timeout));
    assert_eq!(code(&creation_error(&timeout)), Some(ErrorCode::Timeout));
    assert!(
        matches!(creation_error(&crate::Error::StaleConnection), Response::Error {message, ..} if message.contains("unknown") && message.contains("before retrying"))
    );
}

fn core_failure(error: crate::core_terminals::Error) -> Response {
    let (report, receipt) = std::sync::mpsc::sync_channel(1);
    report.send(Err(error)).unwrap();
    Pending::Core(receipt).poll("claude", 7).unwrap()
}

#[test]
fn core_creation_errors_preserve_timeout_and_unknown_outcome_classifications() {
    use sheprd_core_client::{Error, Method};
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
        Error::Backend(sheprd_backend::Error::EmptyTerminalId),
    ] {
        assert_eq!(core_failure(error.into()), unknown(ErrorCode::Failed));
    }
}

#[test]
fn known_core_refusals_worker_failures_and_unplaced_ids_keep_their_diagnostics() {
    for error in [
        crate::core_terminals::Error::Core(sheprd_core_client::Error::Refused {
            method: sheprd_core_client::Method::TerminalRun,
            kind: None,
            message: "fixture refusal".into(),
            hint: None,
        }),
        crate::core_terminals::Error::Worker(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "fixture worker refusal",
        )),
        crate::core_terminals::Error::ViewMissing(
            sheprd_backend::TerminalId::new("created-but-unplaced").unwrap(),
        ),
    ] {
        let expected = Response::error(ErrorCode::Failed, error.to_string());
        assert_eq!(core_failure(error), expected);
    }
}
