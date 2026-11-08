//! Errors in a Spaceport workspace document keep invalid data distinct from missing targets.
//! The caller decides whether to refresh, refuse an edit, or show a diagnostic.

use uuid::Uuid;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("workspace document repeats a terminal reference")]
    DuplicateTerminalRef,
    #[error("workspace document repeats workspace {0}")]
    DuplicateWorkspace(Uuid),
    #[error("workspace document has an invalid Core identity")]
    InvalidCoreId,
    #[error("workspace label is empty, too long, or has control characters")]
    InvalidLabel,
    #[error("workspace document has an invalid terminal reference")]
    InvalidTerminalRef,
    #[error("workspace document has an invalid workspace identity")]
    InvalidWorkspaceId,
    #[error("workspace document contains malformed JSON fields")]
    Json(#[from] serde_json::Error),
    #[error("workspace does not contain this terminal reference")]
    MissingTerminalRef,
    #[error("workspace {0} is missing")]
    MissingWorkspace(Uuid),
    #[error("workspace document schema {0} is newer than this client")]
    NewerSchema(u64),
    #[error("workspace document schema {0} is unsupported")]
    UnsupportedSchema(u64),
    // keep-sorted end
}
