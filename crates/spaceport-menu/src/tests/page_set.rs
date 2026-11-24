use crate::page::{Page, WorkspaceAction, WorkspaceMenuAction};

#[test]
fn context_menus_open_at_the_pointer_and_dialogs_do_not() {
    for page in [
        Page::Workspace,
        Page::Tab,
        Page::Pane,
        Page::MoveTab,
        Page::PaneProcesses,
        Page::Host,
    ] {
        assert!(page.pointer_anchored(), "{page:?}");
    }
    for page in [
        Page::Palette,
        Page::Keybinds,
        Page::Dialog(WorkspaceAction::Rename),
        Page::About,
    ] {
        assert!(!page.pointer_anchored(), "{page:?}");
    }
}

#[test]
fn the_pane_menus_pages_all_name_one_pane() {
    for page in [
        Page::Pane,
        Page::RenamePane,
        Page::MovePane,
        Page::PaneProcesses,
        Page::KillProcesses,
    ] {
        assert!(page.pane_menu(), "{page:?}");
    }
    for page in [Page::Tab, Page::RenameTab, Page::Group, Page::Palette] {
        assert!(!page.pane_menu(), "{page:?}");
    }
}

#[test]
fn only_the_mute_toggle_draws_in_the_header() {
    assert!(WorkspaceMenuAction::Mute.in_header());
    assert!(WorkspaceMenuAction::Unmute.in_header());
    for action in [
        WorkspaceMenuAction::Dialog(WorkspaceAction::Rename),
        WorkspaceMenuAction::Dialog(WorkspaceAction::Note),
        WorkspaceMenuAction::Collapse,
        WorkspaceMenuAction::Expand,
    ] {
        assert!(!action.in_header(), "{action:?}");
    }
}
