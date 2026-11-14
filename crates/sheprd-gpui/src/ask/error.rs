// ABOUTME: Names Ask controller and persistence failures before display.
// ABOUTME: Preserves original I/O and decoding causes for diagnostics.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Ask Spaceport is busy")]
    Busy,
    #[error(transparent)]
    Chat(#[from] crate::chat::Error),
    #[error("could not read Ask Spaceport state")]
    Decode(#[from] serde_json::Error),
    #[error("Ask Spaceport helper exited ({status}): {diagnostic}")]
    Helper {
        status: std::process::ExitStatus,
        diagnostic: String,
    },
    #[error("Ask Spaceport saved state is invalid or too large")]
    InvalidState,
    #[error("could not access Ask Spaceport state or helper")]
    Io(#[from] std::io::Error),
    #[error("the Ask Spaceport helper is unavailable")]
    MissingHelper,
    #[error("Ask Spaceport needs this app's observation server")]
    NoBroker,
    #[error("Ask Spaceport has no private state directory")]
    NoState,
    #[error("the Ask Spaceport helper did not become ready")]
    NotReady,
}

pub(super) type Result<T> = std::result::Result<T, Error>;
