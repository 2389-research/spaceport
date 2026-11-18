// ABOUTME: Routes CLI workspace termination into the shared native confirmation.
// ABOUTME: Rejects foreign or stale caller sessions before opening that confirmation.
use super::*;
use crate::{menu::Page, sidebar::layout_tests::fixture_window, state::ConnectionStatus};

#[gpui::test]
fn a_workspace_close_shortcut_explains_a_disconnected_refusal(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update_in(cx, |view, window, cx| {
        view.live.status = ConnectionStatus::Disconnected;
        view.command(Command::CloseWorkspace, window, cx);
        assert!(view.menu.page.is_none());
        assert!(
            view.flash.is_some(),
            "a refused workspace command needs visible feedback"
        );
    });
}

#[gpui::test]
fn run_workspace_close_opens_consent_and_refuses_a_foreign_named_session(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.status = ConnectionStatus::Connected;
        view
    });
    let id = cx.cx.update(|cx| main_windows(cx)[0].window_id().as_u64());
    let caller = Caller {
        window: Some(id),
        workspace_id: Some("w3".into()),
        ..Caller::default()
    };
    cx.cx.update(|cx| {
        let response = command(
            Call {
                caller: caller.clone(),
                op: CommandOp::Run {
                    name: "close_workspace".into(),
                },
            },
            cx,
        );
        assert!(matches!(response, Response::Done { .. }), "{response:?}");
        assert_eq!(view.read(cx).menu.page, Some(Page::ConfirmClose));
    });
    view.update_in(cx, |view, window, cx| view.dismiss_menu(window, cx));
    cx.cx.update(|cx| {
        for caller in [
            Caller {
                daemon_socket: Some("/fixture/foreign.sock".into()),
                ..caller.clone()
            },
            Caller {
                workspace_id: Some("absent".into()),
                ..caller.clone()
            },
        ] {
            let response = command(
                Call {
                    caller,
                    op: CommandOp::Run {
                        name: "close_workspace".into(),
                    },
                },
                cx,
            );
            assert!(
                matches!(
                    response,
                    Response::Error {
                        code: ErrorCode::WorkspaceNotFound,
                        ..
                    }
                ),
                "{response:?}"
            );
            assert!(view.read(cx).menu.page.is_none());
        }
    });
}
