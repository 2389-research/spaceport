use super::*;
use crate::sidebar::{activity::tests::layout::fixture, layout_tests::full_draw};

#[gpui::test]
fn automation_activity_reports_unavailable_without_script_controls(cx: &mut gpui::TestAppContext) {
    let (view, cx) = fixture(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.show_activity_panel(ActivityPanel::Automations, window, cx)
        });
        full_draw(window, cx).clear(cx);
    });
    assert!(cx.debug_bounds("automation-activity-unavailable").is_some());
    assert!(cx.debug_bounds("automations-enable-0").is_none());
    assert!(cx.debug_bounds("automations-pause-all-0").is_none());
}
