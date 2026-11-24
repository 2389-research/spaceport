//! The menu model's own failures, which this app words: the snapshot fences a
//! menu keeps and the requests a dialog refuses to send.

use super::Localize;
use crate::i18n::fl;
use spaceport_menu::Error;

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            Self::EmptyTabName => fl!("error-empty-tab-name"),
            Self::EmptyWorkspaceLabel => fl!("error-empty-workspace-label"),
            Self::LocalWorktreeNote => fl!("error-local-worktree-note"),
            Self::StalePane => fl!("error-stale-pane"),
            Self::StaleTab => fl!("error-stale-tab"),
            Self::StaleWorkspace => fl!("error-stale-workspace"),
        }
    }
}
