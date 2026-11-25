//! Every variant of the menu model's own failure words itself.

use super::*;
use spaceport_i18n::assertions::assert_localized;

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
