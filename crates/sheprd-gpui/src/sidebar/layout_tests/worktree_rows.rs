use super::*;

#[gpui::test]
fn the_workspace_menu_folds_and_unfolds_a_worktree_group(cx: &mut gpui::TestAppContext) {
    use gpui::{Modifiers, MouseButton};
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        crate::bind_keys(cx);
        let view = cx.new(|cx| fixture_window(window, cx));
        cx.observe(&view, |_, _, cx| cx.notify()).detach();
        SidebarFixture(view)
    });
    let view = cx.update(|_, cx| fixture.read(cx).0.clone());
    cx.update(|_, cx| {
        view.update(cx, |view, _| {
            view.live.status = crate::state::ConnectionStatus::Connected;
        })
    });
    cx.simulate_resize(size(px(800.), px(600.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    for (item, icon, collapsed) in [
        (
            "workspace-menu-Collapse group",
            "workspace-menu-icon-Collapse group",
            true,
        ),
        (
            "workspace-menu-Expand group",
            "workspace-menu-icon-Expand group",
            false,
        ),
    ] {
        let parent = cx.debug_bounds("row-agent-launcher").unwrap();
        cx.simulate_mouse_down(parent.center(), MouseButton::Right, Modifiers::default());
        cx.simulate_mouse_up(parent.center(), MouseButton::Right, Modifiers::default());
        cx.update(|window, cx| {
            full_draw(window, cx).clear(cx);
            assert!(view.read(cx).menu.page == Some(crate::menu::Page::Workspace));
        });
        let row = cx
            .debug_bounds(item)
            .unwrap_or_else(|| panic!("missing {item}"));
        assert!(cx.debug_bounds(icon).is_some(), "missing {icon}");
        cx.simulate_click(row.center(), Modifiers::default());
        cx.update(|window, cx| {
            cx.default_global::<TextProbes>().0.clear();
            window.refresh();
            full_draw(window, cx).clear(cx);
            let view = view.read(cx);
            // Folding is the client's own view of the list, not a daemon request.
            assert!(view.menu.page.is_none(), "{item} left the menu open");
            assert_eq!(
                view.endpoints[0].collapsed_repos.contains(REPO_KEY),
                collapsed
            );
            assert_eq!(
                !cx.global::<TextProbes>().0.contains_key("sidebar-child"),
                collapsed,
                "{item} did not change the visible children"
            );
        });
    }
}
