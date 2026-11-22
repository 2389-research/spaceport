//! The failures the chat model below the views reports as one type.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    /// The `[chat.env]` table a chat's tab would be created with.
    #[error(transparent)]
    Environment(#[from] crate::env::Error),
    /// Discovering the agent conversation already running in a pane.
    #[error(transparent)]
    Existing(#[from] crate::existing::Error),
    // keep-sorted end
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
