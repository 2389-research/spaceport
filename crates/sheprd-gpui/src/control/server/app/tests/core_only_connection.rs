// ABOUTME: Refuses legacy connection controls when a client window has no legacy endpoint.
// ABOUTME: Keeps explicit control replies honest without starting or detaching another runtime.
use super::*;

#[gpui::test]
fn core_only_connection_controls_report_unsupported(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_core_window);
    let id = cx.update(|window, cx| {
        view.update(cx, |view, _| {
            view.endpoints.clear();
            view.live.snapshot = None;
        });
        window.window_handle().window_id().as_u64()
    });
    for (op, name) in [
        (WindowOp::Detach, "detach"),
        (WindowOp::Reconnect, "reconnect"),
    ] {
        let response = cx.cx.update(|cx| {
            window(
                Call {
                    caller: Caller {
                        window: Some(id),
                        ..Caller::default()
                    },
                    op,
                },
                cx,
            )
        });
        assert_eq!(code(&response), Some(ErrorCode::Unsupported));
        let response = cx.cx.update(|cx| {
            command(
                Call {
                    caller: Caller {
                        window: Some(id),
                        ..Caller::default()
                    },
                    op: CommandOp::Run { name: name.into() },
                },
                cx,
            )
        });
        assert_eq!(code(&response), Some(ErrorCode::Unsupported));
    }
}
