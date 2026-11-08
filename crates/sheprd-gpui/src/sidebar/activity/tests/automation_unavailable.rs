use super::*;
use crate::sidebar::activity::tests::sections::{local, show};

#[gpui::test]
fn automation_activity_reports_unavailable_without_script_controls(cx: &mut gpui::TestAppContext) {
    let (view, cx) = local(cx);
    show(&view, cx, ActivityPanel::Automations);
    assert!(cx.debug_bounds("automation-activity-unavailable").is_some());
    assert!(cx.debug_bounds("automations-enable-0").is_none());
    assert!(cx.debug_bounds("automations-pause-all-0").is_none());
}
