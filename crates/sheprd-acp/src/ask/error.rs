// ABOUTME: Retains typed failures for the restricted app-owned helper.
// ABOUTME: Refuses uncertain persistence and unsupported runtime profiles.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Ask Spaceport I/O failed")]
    Io(#[from] std::io::Error),
    #[error("Ask Spaceport checkpoint encoding failed")]
    Json(#[from] serde_json::Error),
    #[error("Ask Spaceport state is invalid or belongs to another conversation")]
    InvalidState,
    #[error("Ask Spaceport checkpoint exceeded its byte bound")]
    TooLarge,
    #[error("another helper owns this Ask Spaceport conversation")]
    Owned,
    #[error(
        "The last Ask Spaceport request has an unknown outcome. Start New Chat; it will not be resent"
    )]
    UnknownOutcome,
    #[error("Pi is not installed")]
    MissingPi,
    #[error("Ask Spaceport requires Pi 1.5.2; found {0}. Other versions are unqualified")]
    UnsupportedVersion(String),
    #[error("Pi did not prove the restricted observation role before startup timed out")]
    RoleNotReady,
    #[error("Pi version inspection timed out")]
    VersionTimeout,
}
