//! What a screen reader calls each popup page. Keyboard focus stays on the
//! popup's overlay while it is open, so the overlay carries the page's role
//! and title, and the page's own rows and fields sit under it.

// i18n: migrated
use crate::page::{Page, WorkspaceAction};
use gpui::{Role, SharedString};
use spaceport_i18n::fl;

/// A list of commands is a menu; anything that asks or edits is a dialog.
pub fn role(page: Page) -> Role {
    match page {
        Page::Application
        | Page::Group
        | Page::Host
        | Page::Menu
        | Page::Pane
        | Page::Tab
        | Page::CoreWorkspaceActions
        | Page::Workspace => Role::Menu,
        _ => Role::Dialog,
    }
}

/// The page's title, in the current language.
pub fn name(page: Page) -> SharedString {
    SharedString::from(match page {
        Page::About => fl!("menu-page-about-herdr-ui"),
        Page::AddDevice => fl!("menu-page-add-device"),
        Page::AddWsl => fl!("menu-page-add-wsl-distribution"),
        Page::AgentSkill => fl!("menu-page-install-the-agent-skill"),
        Page::Application => fl!("menu-page-menu"),
        Page::Update => fl!("menu-page-update"),
        Page::CoreWorkspaceActions => fl!("core-workspace-actions"),
        Page::CoreWorkspaceCreate => fl!("menu-page-new-workspace"),
        Page::CoreWorkspaceDelete => fl!("core-workspace-delete-title"),
        Page::CoreWorkspaceRename => fl!("core-workspace-rename-title"),
        Page::ConfirmClose => fl!("menu-page-close"),
        Page::Devices => fl!("menu-page-devices"),
        Page::DeviceTags => fl!("device-tags-page"),
        Page::DeviceTheme => fl!("menu-page-device-theme"),
        Page::Dialog(action) => match action {
            WorkspaceAction::Rename => fl!("menu-page-rename-workspace"),
            WorkspaceAction::NewTab => fl!("menu-page-new-tab"),
            WorkspaceAction::NewWorkspace => fl!("menu-page-new-workspace"),
            WorkspaceAction::Note => fl!("menu-page-worktree-note"),
        },
        Page::ForwardPort => fl!("menu-page-forward-port"),
        Page::Group => fl!("menu-page-tab-actions"),
        Page::Host => fl!("menu-page-device"),
        Page::Keybinds => fl!("menu-page-keyboard-shortcuts"),
        Page::KillProcesses => fl!("menu-page-end-processes"),
        Page::Menu => fl!("menu-page-menu"),
        Page::MovePane => fl!("menu-page-move-pane"),
        Page::MoveTab => fl!("menu-page-move-tab"),
        Page::OpenLink => fl!("menu-page-open-link"),
        Page::Palette => fl!("menu-page-command-palette"),
        Page::Pane => fl!("menu-page-pane"),
        Page::PaneProcesses => fl!("menu-page-processes"),
        Page::RemoveDevice => fl!("menu-page-remove-device"),
        Page::RemoveWsl => fl!("menu-page-remove-wsl-distribution"),
        Page::RenameDevice => fl!("menu-page-rename-device"),
        Page::RenamePane => fl!("menu-page-rename-pane"),
        Page::RenameTab => fl!("menu-page-rename-tab"),
        Page::Tab => fl!("menu-page-tab"),
        Page::Themes => fl!("menu-page-themes"),
        Page::Usage(provider) => fl!("menu-page-usage", provider = provider.name()),
        Page::VersionMismatch => fl!("menu-page-version-mismatch"),
        Page::Workspace => fl!("menu-page-workspace"),
    })
}
