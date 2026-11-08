#![allow(clippy::unwrap_used)]
use super::*;
use crate::{
    browser::CoreGroupKey,
    control::protocol::{Caller, chat::Reply},
    core_terminals::set_document_for_test_as,
    sidebar::layout_tests::fixture_window,
};

#[gpui::test]
fn core_group_without_terminal_refuses_untargeted_legacy_chat(cx: &mut gpui::TestAppContext) {
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
fn converting_needs_chats_on_and_an_agent_to_move(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, _| view.config.features.acp_chats = false);
    cx.cx.update(|cx| {
        let off = serve(call(ChatOp::Convert { pane: None }), cx);
        assert_eq!(code(&off), Some(ErrorCode::Unsupported));
    });
}
