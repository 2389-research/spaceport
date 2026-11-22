use super::*;

#[gpui::test]
fn legacy_window_without_a_focused_workspace_keeps_its_terminal(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|_, cx| {
        let view = view.read(cx);
        assert!(view.selected_core_group().is_none());
        assert!(view.active_browser_key(cx).is_none());
        let group = view.group_slots()[0].id;
        assert_eq!(view.group_shown(group, cx), crate::browser::Shown::Terminal);
    });
}
