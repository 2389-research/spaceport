// ABOUTME: Checks Core opt-in snapshot behavior across saved preference changes.
// ABOUTME: Simulates separate launches without contacting a Core runtime.
use super::*;

#[gpui::test]
fn saved_edits_do_not_change_the_running_app(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        assert!(!enabled(cx));
        install(false, cx);
        assert!(restart_required(true, cx));
        assert!(!enabled(cx));
        assert!(!restart_required(false, cx));
    });
}

#[gpui::test]
fn enabled_session_stays_enabled_until_restart(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        install(true, cx);
        assert!(enabled(cx));
        assert!(restart_required(false, cx));
        assert!(enabled(cx));
    });
}
