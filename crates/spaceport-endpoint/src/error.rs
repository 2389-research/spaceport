//! Why the saved devices could not be read or kept.

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    /// The client's own storage step, which names the file and the cause.
    ///
    /// The wrapper keeps the client's error as its source, so a failure can
    /// still be told apart by the step behind it.
    #[error("{0}")]
    Client(#[from] spaceport_client::Error),
    /// The worker that would read or write the catalog could not be started.
    #[error("{0}")]
    Worker(#[from] std::io::Error),
    // keep-sorted end
}
