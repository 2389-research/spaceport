// ABOUTME: Groups popup pages, captured targets, and their shared window presentation.
// ABOUTME: Keeps runtime authorization separate from popup input and departure paint.
//! The window's menu popups: the pages they show, the state they own, and the
//! workspace operations they carry to the daemon. Input is isolated to the
//! popup while one is open, and every modal action is fenced by the connection
//! it was started under.

mod appear;
mod chrome;
mod colors;
pub(crate) mod devices;
mod mute;
mod open_link;
mod origin;
mod page;
mod page_title;
mod sessions;
mod settings;
mod shortcut_editor;
#[cfg(all(feature = "integration-test", target_os = "linux"))]
pub(crate) use shortcut_editor::verify_native as verify_shortcut_editor;
mod state;
mod static_paint;
mod version_mismatch;
mod whats_new;
mod workspace;
mod worktree_note;
mod worktree_open;
mod worktree_shortcut;

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
#[cfg(test)]
mod worktree_open_tests;

#[cfg(any(windows, test, feature = "integration-test"))]
pub(crate) use state::Cover;
pub(crate) use {
    colors::{accent, danger, online, tint},
    page::{Page, WorkspaceAction},
    state::MenuState,
    version_mismatch::Notice as VersionNotice,
    worktree_shortcut::WorktreeShortcut,
};

pub(crate) use devices::{DeviceScope, SettingsHint, enter as enter_key};
use page::WorkspaceMenuAction;
use state::Submission;
use workspace::WorkspaceTarget;

/// Breathing room between a popup and the window's edges, so a list that had
/// to be clamped still shows that it stops short of the frame.
const MENU_MARGIN: f32 = 8.;

/// The text a workspace dialog shows when the daemon refused its request:
/// the code, which names the refusal, then the daemon's message.
pub(crate) fn refusal(response: &serde_json::Value) -> Option<String> {
    let error = sheprd_client::response::result(response).err()?;
    let sheprd_client::Error::Endpoint { code, .. } = &error else {
        return None;
    };
    Some(format!("{}: {error}", code.as_str()))
}
