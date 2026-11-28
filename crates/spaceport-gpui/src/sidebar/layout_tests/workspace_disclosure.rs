// ABOUTME: Exercises the workspace disclosure through the rendered sidebar.
// ABOUTME: Checks the list, search field, and saved split across both clicks.
use super::*;

#[gpui::test]
fn workspace_disclosure_gives_agents_the_space_and_restores_the_split(
    cx: &mut gpui::TestAppContext,
) {
    for searchable in [true, false] {
        let (_, cx) = cx.add_window_view(|window, cx| {
            let mut view = fixture_window(window, cx);
            view.config.show_sidebar_search = searchable;
            view.sidebar_split = Some(0.65);
            view
        });
        cx.simulate_resize(size(px(640.), px(600.)));
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        let agents = cx.debug_bounds("agents-section").unwrap();
        let click = |cx: &mut gpui::VisualTestContext| {
            let button = cx.debug_bounds("workspaces-disclosure").unwrap();
            cx.simulate_click(button.center(), Modifiers::none());
            cx.update(|window, cx| full_draw(window, cx).clear(cx));
        };
        click(cx);
        assert!(cx.debug_bounds("spaces-scroll").is_none());
        assert!(cx.debug_bounds("sidebar-search").is_none());
        assert!(cx.debug_bounds("sidebar-split-resize").is_none());
        assert!(cx.debug_bounds("agents-section").unwrap().size.height > agents.size.height);
        assert!(cx.debug_bounds("sidebar-menu").is_some());
        click(cx);
        assert!(cx.debug_bounds("spaces-scroll").is_some());
        assert_eq!(cx.debug_bounds("sidebar-search").is_some(), searchable);
        assert_eq!(cx.debug_bounds("agents-section").unwrap(), agents);
    }
}
