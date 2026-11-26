//! What a screen reader calls each popup page.

use crate::page::{Page, WorkspaceAction};
use crate::page_title::{name, role};
use gpui::Role;

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
