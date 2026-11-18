// ABOUTME: Preserves typed Core startup, worker, and placement failures.
// ABOUTME: Names acknowledged terminals whose native views could not be placed.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error(transparent)]
    Core(#[from] sheprd_core_client::Error),
    #[error("No separately installed herdr-core was found on PATH")]
    NotInstalled,
    #[error("Cannot bind this view while a Core start is still in progress")]
    StartPending,
    #[error("Core terminal {0} was created but could not be placed in the window")]
    ViewMissing(sheprd_backend::TerminalId),
    #[error("Could not start a worker for the terminal: {0}")]
    Worker(#[source] std::io::Error),
    #[error("Joined Core {actual}, expected {expected}")]
    WrongCore { expected: String, actual: String },
    // keep-sorted end
}
