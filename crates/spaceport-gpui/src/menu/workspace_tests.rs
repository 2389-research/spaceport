#![allow(clippy::unwrap_used)]

use super::{WorkspaceAction, WorkspaceMenuAction, WorkspaceTarget};
use crate::{SpaceportWindow, dialog_input::DialogInput, sidebar};

mod dialog_layout;
mod en_xa;
mod excluded_creation;
mod excluded_remote_workspace;
mod group_fold;
mod mute;
mod no_pull_request;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod scripts;
mod targets;
mod tiles;
// Host scripts, which find and start agents, need a POSIX client.
#[cfg(any(target_os = "linux", target_os = "macos"))]
/// Presses the open workspace dialog's submit, for `endpoint::lifecycle_tests`.
#[cfg(unix)]
pub(crate) fn submit_dialog(
    view: &mut SpaceportWindow,
    window: &mut gpui::Window,
    cx: &mut gpui::Context<SpaceportWindow>,
) {
    view.submit_workspace_dialog(window, cx);
}

/// Whether the open workspace menu offers a note, for tests outside this module.
pub(crate) fn offers_note(view: &SpaceportWindow) -> bool {
    view.workspace_items()
        .iter()
        .any(|item| item.action == WorkspaceMenuAction::Dialog(WorkspaceAction::Note))
}

/// The workspace a menu currently targets, for tests outside this module.
pub(crate) fn target_id(view: &SpaceportWindow) -> Option<&str> {
    view.menu.target.as_ref().map(|target| target.id.as_str())
}

pub(crate) fn check_menu_interactions(
    view: &gpui::Entity<SpaceportWindow>,
    cx: &mut gpui::VisualTestContext,
) {
    use gpui::{Modifiers, point, px};
    let selection = |view: &SpaceportWindow| {
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
                &*items[0].selector().leak(),
                &*items[1].selector().leak(),
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
        (page == super::Page::Workspace).then(|| view.workspace_items()[1].action)
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
