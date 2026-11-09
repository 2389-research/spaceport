use super::*;

#[gpui::test]
fn worktree_rows_mark_uncommitted_work(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|cx| fixture_window(window, cx));
        cx.observe(&view, |_, _, cx| cx.notify()).detach();
        SidebarFixture(view)
    });
    let view = cx.update(|_, cx| fixture.read(cx).0.clone());
    cx.simulate_resize(size(px(800.), px(600.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let clean_label = cx.debug_bounds("name-sidebar-child").unwrap();
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            let now = std::time::Instant::now();
            let input = |branch: &str| crate::pull_request::Input {
                checkout: None,
                repo_key: Some(REPO_KEY.into()),
                branch: branch.into(),
            };
            // Both the child and its parent have uncommitted work.
            view.git
                .seed_probe(input("worktree/sidebar-child"), true, now);
            view.git.seed_probe(input("develop"), true, now);
            // Answered and clean: no mark, and no column reserved for one.
            view.git.seed_probe(
                input("worktree/sidebar-child-with-a-long-readable-branch-name"),
                false,
                now,
            );
            cx.notify();
        })
    });
    cx.update(|window, cx| {
        window.refresh();
        full_draw(window, cx).clear(cx);
    });
    let row = cx.debug_bounds("row-sidebar-child").unwrap();
    let badge = cx.debug_bounds("badge-sidebar-child").unwrap();
    let dot = cx.debug_bounds("dirty-sidebar-child").unwrap();
    assert_eq!(dot.size.width, px(12.));
    assert_eq!(dot.size.height, px(12.));
    // The mark leads the badge column, still flush against the row's edge.
    assert!(badge.left() <= dot.left() && dot.right() <= badge.right());
    assert_eq!(badge.right(), row.right() - px(12.));
    assert!(cx.debug_bounds("name-sidebar-child").unwrap().right() <= dot.left());
    assert!(clean_label.size.width > cx.debug_bounds("name-sidebar-child").unwrap().size.width);
    // The parent also earns a badge column.
    let head = cx.debug_bounds("badge-agent-launcher").unwrap();
    let head_dot = cx.debug_bounds("dirty-agent-launcher").unwrap();
    assert_eq!(
        head.right(),
        cx.debug_bounds("row-agent-launcher").unwrap().right() - px(12.)
    );
    assert!(head.left() <= head_dot.left() && head_dot.right() <= head.right());
    assert!(
        cx.debug_bounds("dirty-sidebar-child-with-a-long-readable-branch-name")
            .is_none(),
        "a clean checkout is not marked"
    );
}

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
            assert_eq!(view.collapsed_repos.contains(REPO_KEY), collapsed);
            assert_eq!(
                !cx.global::<TextProbes>().0.contains_key("sidebar-child"),
                collapsed,
                "{item} did not change the visible children"
            );
        });
    }
}
