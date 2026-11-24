// ABOUTME: Re-exports the popup page set and words the workspace menu's artwork.
// ABOUTME: Keeps every importer of `crate::menu::Page` on the same path.
//! What a menu popup is currently showing, as `spaceport-menu` defines it, and
//! the artwork each workspace action's row draws, which is this app's.

pub(crate) use spaceport_menu::page::{Page, WorkspaceAction, WorkspaceMenuAction};

/// Embedded icon for the row, so each action is recognizable before reading.
pub(crate) fn icon(action: WorkspaceMenuAction) -> Option<&'static str> {
    Some(match action {
        WorkspaceMenuAction::Dialog(WorkspaceAction::Rename) => "icons/pencil.svg",
        WorkspaceMenuAction::Dialog(WorkspaceAction::Note) => "icons/note.svg",
        WorkspaceMenuAction::Dialog(WorkspaceAction::NewTab | WorkspaceAction::NewWorkspace) => {
            "icons/plus.svg"
        }
        WorkspaceMenuAction::Collapse => "icons/chevron-up.svg",
        WorkspaceMenuAction::Expand => "icons/chevron-down.svg",
        WorkspaceMenuAction::Mute => crate::icons::MUTED,
        WorkspaceMenuAction::Unmute => "icons/bell.svg",
    })
}
