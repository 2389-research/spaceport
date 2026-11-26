//! The menu model's own failures, which this app words: the snapshot fences a
//! menu keeps and the requests a dialog refuses to send.

use crate::Error;
use spaceport_i18n::{Localize, fl};

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

#[cfg(test)]
mod tests;
