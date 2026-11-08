//! Workspace edit failures distinguish concurrent target changes from Core I/O.
//! A timeout keeps its original Core error because its write outcome is unknown.

use uuid::Uuid;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("workspace {0} changed while this edit was pending")]
    Changed(Uuid),
    #[error("the workspace document changed again during a safe rebase")]
    Conflict,
    #[error(transparent)]
    Core(#[from] sheprd_core_client::Error),
    #[error(transparent)]
    Document(#[from] super::super::Error),
    // keep-sorted end
}
