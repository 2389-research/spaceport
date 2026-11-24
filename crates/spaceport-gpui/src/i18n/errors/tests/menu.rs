use super::*;
use spaceport_menu::Error;

#[test]
fn menu_fences_and_refused_sends_are_complete_localized_messages() {
    assert_localized(&[
        Error::EmptyTabName,
        Error::EmptyWorkspaceLabel,
        Error::LocalWorktreeNote,
        Error::StalePane,
        Error::StaleTab,
        Error::StaleWorkspace,
    ]);
}
