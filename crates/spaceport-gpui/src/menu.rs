//! The window's menu popups: the pages they show, the state they own, and the
//! workspace operations they carry to the daemon. Input is isolated to the
//! popup while one is open, and every modal action is fenced by the connection
//! it was started under.

mod appear;
pub(crate) mod chrome;
mod colors;
pub(crate) mod devices;
pub(crate) mod labels;
mod mute;
mod open_link;
mod page;
mod sessions;
mod settings;
mod shortcut_editor;
#[cfg(all(feature = "integration-test", target_os = "linux"))]
pub(crate) use shortcut_editor::verify_native as verify_shortcut_editor;
mod state;
mod version_mismatch;
mod whats_new;
mod workspace;
mod worktree_note;

#[cfg(test)]
mod account_route_tests;
#[cfg(test)]
mod announcement_tests;
#[cfg(test)]
mod en_xa_tests;
#[cfg(test)]
mod font_size_tests;
#[cfg(test)]
mod footer_items_tests;
#[cfg(test)]
mod git_read_removal_tests;
#[cfg(test)]
mod reset_tests;
#[cfg(test)]
mod sessions_tests;
#[cfg(test)]
mod version_mismatch_tests;
#[cfg(test)]
pub(crate) mod workspace_tests;

#[cfg(any(
    test,
    all(
        feature = "integration-test",
        any(target_os = "macos", target_os = "linux")
    )
))]
pub(crate) use state::Cover;
pub(crate) use {
    colors::{accent, danger, online, tint},
    page::{Page, WorkspaceAction, icon as workspace_action_icon},
    state::MenuState,
    version_mismatch::Notice as VersionNotice,
};

use crate::direction::Logical;
pub(crate) use devices::{SettingsHint, enter as enter_key};
use page::WorkspaceMenuAction;
use workspace::WorkspaceTarget;

/// Shared trailing action geometry for pickers and workspace context menus.
fn action_icon(
    path: &'static str,
    selector: String,
    color: gpui::Rgba,
    hover_color: gpui::Rgba,
) -> gpui::Div {
    use gpui::{prelude::*, *};
    div()
        .group("menu-action-icon")
        .size(px(24.))
        .flex_none()
        .row()
        .items_center()
        .justify_center()
        .rounded(px(crate::config::corners::CONTROL))
        .child(
            svg()
                .path(path)
                .size(px(14.))
                // GPUI skips SVG painting without an explicit color on the SVG;
                // the parent's text color is not inherited by SVG painting.
                .text_color(color)
                .group_hover("menu-action-icon", move |s| s.text_color(hover_color))
                .debug_selector(move || selector.clone()),
        )
}
