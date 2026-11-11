// ABOUTME: Checks update commands refuse unsupported application builds.
// ABOUTME: Keeps CLI success tied to an actual native update check.
use super::*;

#[gpui::test]
fn development_update_check_is_unsupported(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let response = command(
            call(CommandOp::Run {
                name: "check-for-updates".into(),
            }),
            cx,
        );
        assert_eq!(code(&response), Some(ErrorCode::Unsupported));
    });
}

#[cfg(all(target_os = "macos", feature = "macos-updater"))]
#[test]
fn update_check_busy_is_reported_as_busy() {
    assert_eq!(
        code(&update_error(crate::updates::Error::Busy)),
        Some(ErrorCode::Busy)
    );
}
