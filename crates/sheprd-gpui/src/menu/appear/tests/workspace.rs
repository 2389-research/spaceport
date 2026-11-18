// ABOUTME: Keeps workspace popovers and forms visible after their active targets reset.
// ABOUTME: Checks departing worktree results without retaining their search owner.
use super::*;
use crate::{sidebar::layout_tests::fixture_window, state::ConnectionStatus};
use gpui::{ScrollDelta, ScrollEvent, TestAppContext};

fn hold_departure(view: &mut SheprdWindow) {
    let later = motion::now() + std::time::Duration::from_secs(3600);
    view.menu.appear.observe(None, None, later);
    let leaving = view
        .menu
        .appear
        .leaving
        .as_mut()
        .expect("workspace page leaves");
    leaving.motion.set(true, later, true);
    leaving.motion.set(false, later, false);
}

#[gpui::test]
fn workspace_pages_keep_their_body_after_target_reset(cx: &mut TestAppContext) {
    use crate::menu::WorkspaceAction;
    let mut changed = Vec::new();
    for action in [
        None,
        Some(WorkspaceAction::Rename),
        Some(WorkspaceAction::NewTab),
        Some(WorkspaceAction::NewWorkspace),
        Some(WorkspaceAction::Note),
    ] {
        let (view, cx) = cx.add_window_view(fixture_window);
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.live.status = ConnectionStatus::Connected;
                view.open_workspace_menu("w3", point(px(500.), px(200.)), window, cx);
                if let Some(action) = action {
                    view.open_workspace_dialog(action, window, cx);
                    if let Some(input) = view.menu.input.as_mut() {
                        input.replace(None, "departure draft", false, None);
                    }
                    view.menu.error = Some("Synthetic endpoint failure".into());
                }
            });
            window.draw(cx).clear(cx);
        });
        let selector = if action.is_some() {
            "dialog-header"
        } else {
            "workspace-menu-name"
        };
        let before = cx.debug_bounds(selector).expect("live workspace content");
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.menu.appear.live = true;
                view.dismiss_menu(window, cx);
                assert!(view.menu.target.is_none());
                assert!(view.menu.input.is_none());
                hold_departure(view);
            });
            window.draw(cx).clear(cx);
        });
        if cx.debug_bounds(selector) != Some(before) {
            changed.push((action, before, cx.debug_bounds(selector)));
        }
    }
    assert!(
        changed.is_empty(),
        "departing workspace content changed: {changed:?}"
    );
}

#[gpui::test]
fn worktree_results_leave_scrolled_after_search_owner_drops(cx: &mut TestAppContext) {
    use crate::menu::{WorkspaceAction, worktree_open::Picker};
    use gpui::AppContext;
    let (view, cx) = cx.add_window_view(fixture_window);
    let search = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.live.status = ConnectionStatus::Connected;
            view.open_workspace_menu("w3", point(px(500.), px(200.)), window, cx);
            view.open_workspace_dialog(WorkspaceAction::OpenWorktree, window, cx);
            let mut picker = Picker::new(cx.new(crate::search_input::SearchInput::new));
            let search = picker.search.downgrade();
            picker.pending = Some("departure-list".into());
            view.menu.worktree_open = Some(picker);
            let rows: Vec<_> = (0..80).map(|index| serde_json::json!({
                "path": format!("/synthetic/checkout-{index}"), "label": format!("checkout-{index}"),
                "is_bare": false, "is_prunable": false, "is_detached": true
            })).collect();
            view.menu.apply_worktree_list_response("departure-list", Ok(serde_json::json!({"result": {
                "type":"worktree_list", "source": {"repo_key":"/synthetic/.git", "repo_name":"synthetic", "source_workspace_id":"w3"},
                "worktrees":rows
            }})));
            search
        })
    });
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    let row = cx
        .debug_bounds("open-worktree-row-5")
        .expect("live worktree row");
    cx.simulate_event(ScrollEvent {
        position: row.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-80.))),
        ..Default::default()
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let before = cx
        .debug_bounds("open-worktree-row-5")
        .expect("scrolled worktree row");
    assert!(before.top() < row.top());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            assert!(view.menu.worktree_open.is_none());
            hold_departure(view);
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    cx.update(|_, _| {
        assert!(
            search.upgrade().is_none(),
            "departure releases the native search owner"
        )
    });
    let after = cx
        .debug_bounds("open-worktree-row-5")
        .expect("departing worktree row");
    assert!(
        (after.top() - before.top()).abs() < px(1.),
        "scroll position remains visible"
    );
}

#[gpui::test]
fn long_workspace_content_keeps_its_shell_scroll_when_leaving(cx: &mut TestAppContext) {
    use crate::menu::WorkspaceAction;
    let mut changed = Vec::new();
    for action in [None, Some(WorkspaceAction::Rename)] {
        let (view, cx) = cx.add_window_view(fixture_window);
        cx.simulate_resize(gpui::size(px(320.), px(240.)));
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.live.status = ConnectionStatus::Connected;
                view.open_workspace_menu("w3", point(px(10.), px(10.)), window, cx);
                if let Some(action) = action {
                    view.open_workspace_dialog(action, window, cx);
                    view.menu.error = Some("Synthetic endpoint reason. ".repeat(80));
                } else {
                    view.menu.target.as_mut().expect("workspace target").label =
                        "Long workspace label ".repeat(80);
                }
            });
            window.draw(cx).clear(cx);
        });
        let selector = if action.is_some() {
            "dialog-header"
        } else {
            "workspace-menu-name"
        };
        let before = cx.debug_bounds(selector).expect("live content");
        let panel = cx.debug_bounds("menu-panel").expect("scrollable shell");
        cx.simulate_event(ScrollEvent {
            position: panel.center(),
            delta: ScrollDelta::Pixels(point(px(0.), px(-80.))),
            ..Default::default()
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let scrolled = cx.debug_bounds(selector).expect("scrolled content");
        assert!(scrolled.top() < before.top(), "the shell actually scrolls");
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.menu.appear.live = true;
                view.dismiss_menu(window, cx);
                hold_departure(view);
            });
            window.refresh();
            window.draw(cx).clear(cx);
        });
        if cx.debug_bounds(selector) != Some(scrolled) {
            changed.push((action, scrolled, cx.debug_bounds(selector)));
        }
    }
    assert!(
        changed.is_empty(),
        "departing shell scroll changed: {changed:?}"
    );
}

#[gpui::test]
fn worktree_status_keeps_its_scroll_when_leaving(cx: &mut TestAppContext) {
    use crate::menu::WorkspaceAction;
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.simulate_resize(gpui::size(px(320.), px(500.)));
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.live.status = ConnectionStatus::Connected;
            view.open_workspace_menu("w3", Point::default(), window, cx);
            view.open_workspace_dialog(WorkspaceAction::OpenWorktree, window, cx);
            view.menu.error = Some("Synthetic worktree endpoint reason. ".repeat(80));
        });
        window.draw(cx).clear(cx);
    });
    let before = cx.debug_bounds("dialog-error").expect("live status");
    let status = cx
        .debug_bounds("open-worktree-status")
        .expect("scrollable status");
    cx.simulate_event(ScrollEvent {
        position: status.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-40.))),
        ..Default::default()
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let scrolled = cx.debug_bounds("dialog-error").expect("scrolled status");
    assert!(scrolled.top() < before.top(), "the status actually scrolls");
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            hold_departure(view);
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert_eq!(
        cx.debug_bounds("dialog-error"),
        Some(scrolled),
        "departing status keeps its captured scroll"
    );
}
