use super::*;

#[gpui::test]
fn signed_in_workspace_menu_keeps_actions_without_a_pr_card(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.github = crate::github::Auth::connected_fixture();
            view.live.status = crate::state::ConnectionStatus::Connected;
            view.open_workspace_menu("w3", Default::default(), window, cx);
        });
        sidebar::layout_tests::full_draw(window, cx).clear(cx);
    });
    assert!(cx.debug_bounds("workspace-menu-Rename").is_some());
    assert!(cx.debug_bounds("workspace-pr").is_none());
}
