use super::*;

#[gpui::test]
fn legacy_workspace_menu_cannot_close_or_delete_a_checkout(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    for workspace in ["w3", "w4"] {
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.live.status = crate::state::ConnectionStatus::Connected;
                view.dismiss_menu(window, cx);
                view.open_workspace_menu(workspace, Default::default(), window, cx);
            });
            sidebar::layout_tests::full_draw(window, cx).clear(cx);
        });
        assert!(cx.debug_bounds("workspace-menu-Rename").is_some());
        for removed in [
            "workspace-menu-Close",
            "workspace-menu-Close group",
            "workspace-menu-Delete worktree checkout",
        ] {
            assert!(cx.debug_bounds(removed).is_none(), "{workspace}: {removed}");
        }
    }
}
