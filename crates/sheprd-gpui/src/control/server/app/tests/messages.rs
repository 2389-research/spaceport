// ABOUTME: Opens warning history through an actual application control request.
// ABOUTME: Keeps the named window distinct from another window's warning owner.
use super::*;
use crate::{menu::Page, notice::Cause, sidebar::layout_tests::fixture_window};

#[gpui::test]
fn messages_command_opens_only_the_named_windows_history(cx: &mut gpui::TestAppContext) {
    let (first, id) = {
        let (view, cx) = cx.add_window_view(fixture_window);
        view.update(cx, |view, _| {
            view.notices
                .show(Cause::Command, "First window failure".into());
        });
        let id = cx.update(|window, _| window.window_handle().window_id().as_u64());
        (view, id)
    };
    let (second, cx) = cx.add_window_view(fixture_window);
    second.update(cx, |view, _| {
        view.notices
            .show(Cause::Command, "Second window failure".into());
    });
    let reply = cx.cx.update(|cx| {
        command(
            Call {
                caller: Caller {
                    window: Some(id),
                    ..Caller::default()
                },
                op: CommandOp::Run {
                    name: "messages".into(),
                },
            },
            cx,
        )
    });
    assert!(matches!(reply, Response::Done { .. }), "{reply:?}");
    first.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(Page::Messages));
        assert_eq!(view.notices.history().count(), 1);
        assert_eq!(
            view.notices.history().next_back().unwrap().text.as_ref(),
            "First window failure"
        );
    });
    second.read_with(cx, |view, _| assert!(view.menu.page.is_none()));
}
