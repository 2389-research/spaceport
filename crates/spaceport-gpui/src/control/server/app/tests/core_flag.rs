// ABOUTME: Refuses Core-only control actions before dispatch while Core is disabled.
// ABOUTME: Keeps the CLI's unsupported response distinct from missing-window errors.
use super::*;

#[gpui::test]
fn core_commands_refuse_before_looking_for_a_window(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        for name in ["new_core_terminal", "mirror_core_pane"] {
            let response = command(call(CommandOp::Run { name: name.into() }), cx);
            assert!(
                matches!(response, Response::Error { code: ErrorCode::Unsupported, message }
                if message == crate::core_feature::disabled_message())
            );
        }
        assert_eq!(
            code(&command(
                call(CommandOp::Run {
                    name: "toggle_sidebar".into()
                }),
                cx
            )),
            Some(ErrorCode::NoWindow)
        );
    });
}

#[gpui::test]
fn enabled_core_commands_reach_normal_window_resolution(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        crate::core_feature::install(true, cx);
        assert_eq!(
            code(&command(
                call(CommandOp::Run {
                    name: "new_core_terminal".into()
                }),
                cx
            )),
            Some(ErrorCode::NoWindow)
        );
    });
}
