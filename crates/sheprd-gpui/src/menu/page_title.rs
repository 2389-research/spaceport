//! What a screen reader calls each popup page. Keyboard focus stays on the
//! popup's overlay while it is open, so the overlay carries the page's role
//! and title, and the page's own rows and fields sit under it.

// i18n: migrated
use super::{Page, WorkspaceAction};
use crate::i18n::fl;
use gpui::{Role, SharedString};

impl Page {
    /// A list of commands is a menu; anything that asks or edits is a dialog.
    pub(crate) fn accessible_role(self) -> Role {
        match self {
            Self::Application
            | Self::Group
            | Self::Host
            | Self::Menu
            | Self::Profile
            | Self::Pane
            | Self::Tab
            | Self::CoreWorkspaceActions
            | Self::Workspace => Role::Menu,
            _ => Role::Dialog,
        }
    }

    pub(crate) fn accessible_name(self) -> SharedString {
        SharedString::from(match self {
            Self::About => fl!("menu-page-about-herdr-ui"),
            Self::AddDevice => fl!("menu-page-add-device"),
            Self::AddWsl => fl!("menu-page-add-wsl-distribution"),
            Self::AgentSkill => fl!("menu-page-install-the-agent-skill"),
            Self::Application => fl!("menu-page-menu"),
            Self::AppUpdate | Self::Update => fl!("menu-page-update"),
            Self::CoreWorkspaceActions => fl!("core-workspace-actions"),
            Self::CoreWorkspaceCreate => fl!("menu-page-new-workspace"),
            Self::CoreWorkspaceDelete => fl!("core-workspace-delete-title"),
            Self::CoreWorkspaceRename => fl!("core-workspace-rename-title"),
            Self::ConfirmClose => fl!("menu-page-close"),
            Self::Devices => fl!("menu-page-devices"),
            Self::DeviceTags => fl!("device-tags-page"),
            Self::DeviceTheme => fl!("menu-page-device-theme"),
            Self::Dialog(action) => match action {
                WorkspaceAction::Rename => fl!("menu-page-rename-workspace"),
                WorkspaceAction::OpenWorktree => fl!("menu-page-open-worktree"),
                WorkspaceAction::NewTab => fl!("menu-page-new-tab"),
                WorkspaceAction::NewWorkspace => fl!("menu-page-new-workspace"),
                WorkspaceAction::Note => fl!("menu-page-worktree-note"),
            },
            Self::ForwardPort => fl!("menu-page-forward-port"),
            Self::Group => fl!("menu-page-tab-actions"),
            Self::Host => fl!("menu-page-device"),
            Self::Keybinds => fl!("menu-page-keyboard-shortcuts"),
            Self::KillProcesses => fl!("menu-page-end-processes"),
            Self::Menu => fl!("menu-page-menu"),
            Self::MovePane => fl!("menu-page-move-pane"),
            Self::MoveTab => fl!("menu-page-move-tab"),
            Self::OpenLink => fl!("menu-page-open-link"),
            Self::Palette => fl!("menu-page-command-palette"),
            Self::Pane => fl!("menu-page-pane"),
            Self::PaneProcesses => fl!("menu-page-processes"),
            Self::Profile => fl!("menu-page-profile"),
            Self::RemoveDevice => fl!("menu-page-remove-device"),
            Self::RemoveWsl => fl!("menu-page-remove-wsl-distribution"),
            Self::RenameDevice => fl!("menu-page-rename-device"),
            Self::RenamePane => fl!("menu-page-rename-pane"),
            Self::RenameTab => fl!("menu-page-rename-tab"),
            Self::Tab => fl!("menu-page-tab"),
            Self::Themes => fl!("menu-page-themes"),
            Self::Usage(provider) => fl!("menu-page-usage", provider = provider.name()),
            Self::VersionMismatch => fl!("menu-page-version-mismatch"),
            Self::Workspace => fl!("menu-page-workspace"),
        })
    }
}

#[cfg(test)]
mod tests;
