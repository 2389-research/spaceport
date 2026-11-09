#![allow(clippy::unwrap_used)]

use super::{WorkspaceAction, WorkspaceMenuAction, WorkspaceTarget, state::Deletion};
use crate::{SheprdWindow, dialog_input::DialogInput, sidebar, worktree_scripts::ArchiveCheck};
use sheprd_client::Method;

mod close;
mod close_unconfirmed;
mod dialog_layout;
mod en_xa;
mod excluded_creation;
mod excluded_remote_workspace;
mod left_checkout;
mod mute;
mod naming;
mod no_pull_request;
mod removed_checkpoints;
mod removed_orchestrator;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod scripts;
mod targets;
mod tiles;
mod worktree_delete;
mod worktree_delete_keys;
mod worktree_delete_recovery;
// Host scripts, which find and start agents, need a POSIX client.
#[cfg(any(target_os = "linux", target_os = "macos"))]

/// Presses the open workspace dialog's submit, for `endpoint::lifecycle_tests`.
#[cfg(unix)]
pub(crate) fn submit_dialog(
    view: &mut SheprdWindow,
    window: &mut gpui::Window,
    cx: &mut gpui::Context<SheprdWindow>,
) {
    view.submit_workspace_dialog(window, cx);
}

/// Whether the open workspace menu offers a note, for tests outside this module.
pub(crate) fn offers_note(view: &SheprdWindow) -> bool {
    view.workspace_items()
        .iter()
        .any(|(action, _)| *action == WorkspaceMenuAction::Dialog(WorkspaceAction::Note))
}

/// The workspace a menu currently targets, for tests outside this module.
pub(crate) fn target_id(view: &SheprdWindow) -> Option<&str> {
    view.menu.target.as_ref().map(|target| target.id.as_str())
}

// Drives the workspace menu for `endpoint::lifecycle_tests`, which needs POSIX
// sockets and processes and is therefore compiled there only.
#[cfg(unix)]
pub(crate) fn submit_focus_change(
    view: &mut SheprdWindow,
    method: Method,
    window: &mut gpui::Window,
    cx: &mut gpui::Context<SheprdWindow>,
) {
    let action = match method {
        Method::WorkspaceClose => WorkspaceAction::Close,
        Method::WorktreeOpen => WorkspaceAction::OpenWorktree,
        Method::WorktreeRemove => WorkspaceAction::DeleteWorktree,
        _ => panic!("unexpected fixture action"),
    };
    let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
    snapshot.workspaces = sidebar::layout_tests::snapshot(7).workspaces;
    let index = if action == WorkspaceAction::DeleteWorktree {
        4
    } else {
        3
    };
    let target = WorkspaceTarget::new(snapshot, &snapshot.workspaces[index]);
    let close_check = (action == WorkspaceAction::Close).then(|| {
        super::workspace_close::CloseCheck::fixture(
            snapshot,
            &target,
            Some(super::workspace_close::Report {
                dirty: true,
                unpushed: true,
                unknown: false,
            }),
        )
    });
    view.open_menu(window, cx);
    view.menu.target = Some(target);
    view.menu.page = Some(super::Page::Dialog(action));
    if let Some(check) = close_check {
        view.menu.close_check = Some(check);
        view.menu.input = Some(DialogInput::new("close".into()));
    }
    if action == WorkspaceAction::DeleteWorktree {
        view.menu.deletion = Some(Deletion {
            path: Some("/fixture/checkout".into()),
            archive: ArchiveCheck::Read(None),
            ..Deletion::new(None, false)
        });
    }
    if action == WorkspaceAction::OpenWorktree {
        use gpui::AppContext;
        let mut picker =
            super::worktree_open::Picker::new(cx.new(crate::search_input::SearchInput::new));
        picker.pending = Some("list".into());
        view.menu.worktree_open = Some(picker);
        view.menu.apply_worktree_list_response("list", Ok(serde_json::json!({"result": {
            "type": "worktree_list", "source": {"repo_key":"/fixture/agent-launcher/.git", "repo_name":"agent-launcher", "source_workspace_id":"w3"},
            "worktrees": [{"path": "/endpoint/existing checkout ",
                "label": "existing", "is_bare": false, "is_prunable": false, "is_detached": true}]
        }})));
    }
    view.submit_workspace_dialog(window, cx);
    assert!(view.menu.error.is_none() && view.local_error.is_none());
    // Creation and removal both await their correlated response. Removal also
    // keeps the id on the window in case its confirmation is dismissed.
    let pending = match action {
        WorkspaceAction::DeleteWorktree => {
            assert_eq!(view.menu.page, Some(super::Page::Dialog(action)));
            view.removal.as_ref().unwrap().pending.clone()
        }
        WorkspaceAction::OpenWorktree => view.menu.opening.clone(),
        _ => None,
    };
    if pending.is_some() {
        let inbox = view.endpoints[view.selected_endpoint]
            .connection
            .inbox
            .lock()
            .unwrap();
        assert_eq!(
            inbox.dialog_response.as_ref().map(|(id, _)| id),
            pending.as_ref()
        );
    }
    view.dismiss_menu(window, cx);
}

pub(crate) fn check_menu_interactions(
    view: &gpui::Entity<SheprdWindow>,
    cx: &mut gpui::VisualTestContext,
) {
    use gpui::{Modifiers, point, px};
    let selection = |view: &SheprdWindow| {
        if view.menu.page == Some(super::Page::Workspace) {
            view.menu.workspace_selected.and_then(|selected| {
                view.workspace_menu_actions()
                    .iter()
                    .position(|action| *action == selected)
            })
        } else {
            view.menu.selected
        }
    };
    let (page, count, first, second) = cx.update(|_, cx| {
        let view = view.read(cx);
        assert_eq!(view.menu.selected, None);
        if view.menu.page == Some(super::Page::Workspace) {
            // The first two actions in keyboard order, whichever tiles they are.
            let items = view.workspace_items();
            (
                super::Page::Workspace,
                items.len(),
                // Debug selectors are static; leaking two test labels is fine.
                &*format!("workspace-menu-{}", items[0].1.english()).leak(),
                &*format!("workspace-menu-{}", items[1].1.english()).leak(),
            )
        } else {
            (
                super::Page::Menu,
                view.menu_items().len(),
                "menu-settings",
                "menu-shortcuts",
            )
        }
    });
    cx.simulate_keystrokes("enter");
    cx.update(|_, cx| {
        assert!(view.read(cx).menu.page == Some(page));
        assert_eq!(view.read(cx).menu.selected, None);
        assert!(view.read(cx).menu.input.is_none());
    });
    let first = cx.debug_bounds(first).unwrap().center();
    let second = cx.debug_bounds(second).unwrap().center();
    let outside = point(px(790.), px(590.));
    for (position, selected) in [(second, Some(1)), (first, Some(0)), (outside, None)] {
        cx.simulate_mouse_move(position, None, Modifiers::default());
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(selection(view.read(cx)), selected);
        });
    }
    // Leaving the hovered row also leaves Enter inert.
    cx.simulate_keystrokes("enter");
    cx.update(|_, cx| assert!(view.read(cx).menu.page == Some(page)));
    for (keys, selected) in [
        ("up", count - 1),
        ("down", 0),
        ("up", count - 1),
        ("down down", 1),
    ] {
        cx.simulate_keystrokes(keys);
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(selection(view.read(cx)), Some(selected));
        });
    }
    cx.simulate_mouse_move(first, None, Modifiers::default());
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        assert_eq!(selection(view.read(cx)), Some(0));
    });
    // Keyboard selection replaces hover even while the pointer stays on the first row.
    cx.simulate_keystrokes("down");
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        assert_eq!(selection(view.read(cx)), Some(1));
    });
    cx.simulate_mouse_move(second, None, Modifiers::default());
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        assert_eq!(selection(view.read(cx)), Some(1));
    });
    cx.simulate_mouse_move(outside, None, Modifiers::default());
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        assert_eq!(selection(view.read(cx)), None);
    });
    cx.simulate_keystrokes("down");
    cx.update(|_, cx| assert_eq!(selection(view.read(cx)), Some(0)));
    cx.simulate_mouse_move(second, None, Modifiers::default());
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        assert_eq!(selection(view.read(cx)), Some(1));
    });
    let second_action = cx.update(|_, cx| {
        let view = view.read(cx);
        (page == super::Page::Workspace).then(|| view.workspace_items()[1].0)
    });
    cx.simulate_keystrokes("enter");
    cx.update(|_, cx| {
        let opened = view.read(cx).menu.page;
        match second_action {
            Some(WorkspaceMenuAction::Dialog(action)) => {
                assert!(opened == Some(super::Page::Dialog(action)))
            }
            Some(_) => assert!(opened != Some(page), "the second action did not run"),
            None => assert!(opened == Some(super::Page::Keybinds)),
        }
    });
    cx.simulate_mouse_move(outside, None, Modifiers::default());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let anchor = view.menu.anchor;
            let target = view.menu.target.as_ref().map(|target| target.id.clone());
            view.dismiss_menu(window, cx);
            if let Some(target) = target {
                view.open_workspace_menu(&target, anchor, window, cx);
            } else {
                view.open_menu(window, cx);
            }
            assert_eq!(view.menu.selected, None);
        });
        window.draw(cx).clear(cx);
    });
}
