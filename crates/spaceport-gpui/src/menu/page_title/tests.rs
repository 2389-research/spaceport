// ABOUTME: Checks accessible popup roles and localized page names.
// ABOUTME: Covers retained dialogs and menus in the pseudo locale.
use super::*;

#[test]
fn context_menus_are_menus_and_questions_are_dialogs() {
    assert_eq!(role(Page::Workspace), Role::Menu);
    assert_eq!(role(Page::Host), Role::Menu);
    assert_eq!(role(Page::Dialog(WorkspaceAction::Rename)), Role::Dialog);
    assert_eq!(
        name(Page::Dialog(WorkspaceAction::Rename)),
        "Rename workspace"
    );
    assert_eq!(role(Page::Keybinds), Role::Dialog);
}

/// Under `en-XA` every page's accessible name is a translated message. The
/// names are read by screen readers rather than drawn, so the leak test's
/// text recorder cannot see them.
#[test]
fn every_page_title_is_a_message_under_en_xa() {
    crate::i18n::leak_tests::under_en_xa("page titles en-XA", |_, _| {
        let dialogs = [
            WorkspaceAction::Rename,
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
            Page::ConfirmClose,
            Page::Devices,
            Page::DeviceTheme,
            Page::ForwardPort,
            Page::Group,
            Page::Host,
            Page::Keybinds,
            Page::KillProcesses,
            Page::Menu,
            Page::MovePane,
            Page::MoveTab,
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
        ];
        let leaks: Vec<_> = pages
            .into_iter()
            .chain(dialogs)
            .chain(crate::usage::registry::all().map(Page::Usage))
            .map(name)
            .filter(|name| !spaceport_i18n::pseudo::is_marked(name))
            .collect();
        assert!(leaks.is_empty(), "{leaks:#?}");
    });
}
