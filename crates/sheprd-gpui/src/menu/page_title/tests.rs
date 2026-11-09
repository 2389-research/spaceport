use super::*;

#[test]
fn context_menus_are_menus_and_questions_are_dialogs() {
    assert_eq!(Page::Workspace.accessible_role(), Role::Menu);
    assert_eq!(Page::Host.accessible_role(), Role::Menu);
    assert_eq!(
        Page::Dialog(WorkspaceAction::DeleteWorktree).accessible_role(),
        Role::Dialog
    );
    assert_eq!(
        Page::Dialog(WorkspaceAction::DeleteWorktree).accessible_name(),
        "Delete worktree"
    );
    assert_eq!(Page::Keybinds.accessible_role(), Role::Dialog);
}

/// Under `en-XA` every page's accessible name is a translated message. The
/// names are read by screen readers rather than drawn, so the leak test's
/// text recorder cannot see them.
#[test]
fn every_page_title_is_a_message_under_en_xa() {
    crate::i18n::leak_tests::under_en_xa("page titles en-XA", |_, _| {
        let dialogs = [
            WorkspaceAction::Rename,
            WorkspaceAction::Close,
            WorkspaceAction::OpenWorktree,
            WorkspaceAction::DeleteWorktree,
            WorkspaceAction::NewTab,
            WorkspaceAction::NewWorkspace,
            WorkspaceAction::Note,
        ]
        .map(Page::Dialog);
        let pages = [
            Page::About,
            Page::AddDevice,
            Page::AddWsl,
            Page::AgentSkill,
            Page::Application,
            Page::AppUpdate,
            Page::ConfirmClose,
            Page::Devices,
            Page::DeviceTheme,
            Page::ForwardPort,
            Page::Git,
            Page::GitCommit,
            Page::GitHub,
            Page::Group,
            Page::Host,
            Page::Keybinds,
            Page::KillProcesses,
            Page::Menu,
            Page::MovePane,
            Page::MoveTab,
            Page::OpenCheckout,
            Page::OpenLink,
            Page::Palette,
            Page::Pane,
            Page::PaneProcesses,
            Page::RemoveDevice,
            Page::RemoveWsl,
            Page::RenameDevice,
            Page::RenamePane,
            Page::RenameTab,
            Page::Tab,
            Page::Themes,
            Page::Update,
            Page::VersionMismatch,
            Page::Workspace,
            Page::WorktreeScript,
        ];
        let leaks: Vec<_> = pages
            .into_iter()
            .chain(dialogs)
            .chain(crate::usage::registry::all().map(Page::Usage))
            .map(Page::accessible_name)
            .filter(|name| !sheprd_i18n::pseudo::is_marked(name))
            .collect();
        assert!(leaks.is_empty(), "{leaks:#?}");
    });
}
