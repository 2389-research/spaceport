pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Why a host script or a teleport could not finish. Each
/// part keeps its own error; this gathers them for callers that run several.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("{0}")]
    Client(#[from] sheprd_client::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Path(#[from] sheprd_client::local_path::PathError),
    #[error("preferences path has no parent")]
    PreferencesPath,
    #[error(transparent)]
    Teleport(#[from] crate::teleport::Error),
    // keep-sorted end
}
