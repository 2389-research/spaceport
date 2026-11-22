use std::{io, path::PathBuf};

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Why a state file could not be read or written. Every one of them is a
/// file this app owns: a saved value this build cannot read falls back to
/// the caller's default rather than reaching a window.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    /// The path had no folder to write into, which a state directory's own
    /// file never does.
    #[error("state file {} has no folder", .0.display())]
    MissingRoot(PathBuf),
    #[error("{} exceeds {limit} bytes", path.display())]
    Size { path: PathBuf, limit: u64 },
    // keep-sorted end
}
