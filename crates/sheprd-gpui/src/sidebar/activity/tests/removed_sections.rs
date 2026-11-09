use super::layout::fixture;

#[gpui::test]
fn activity_bar_lists_retained_sections_without_orchestrator_entries(
    cx: &mut gpui::TestAppContext,
) {
    let (_, cx) = fixture(cx);
    assert!(cx.debug_bounds("activity-workspaces").is_some());
    assert!(cx.debug_bounds("activity-overview").is_some());
    assert!(cx.debug_bounds("activity-usage").is_some());
    for selector in [
        "activity-pull-requests",
        "activity-issues",
        "activity-tasks",
    ] {
        assert!(cx.debug_bounds(selector).is_none(), "{selector}");
    }
}
