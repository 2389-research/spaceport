//! The fences a menu's model answers with: a target a newer snapshot no
//! longer holds, and the two names a dialog refuses to send. The app words
//! them (`docs/i18n.md`); these messages are for logs and `Display`.

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("Enter a tab name.")]
    EmptyTabName,
    #[error("Workspace label must not be empty.")]
    EmptyWorkspaceLabel,
    #[error("Worktree notes stay in this app and are never sent to the daemon")]
    LocalWorktreeNote,
    #[error("The original pane changed or no longer exists. Cancel and try again.")]
    StalePane,
    #[error("The original tab changed or no longer exists. Cancel and try again.")]
    StaleTab,
    #[error("Workspace is no longer available. Dismiss and reopen the menu.")]
    StaleWorkspace,
    // keep-sorted end
}
