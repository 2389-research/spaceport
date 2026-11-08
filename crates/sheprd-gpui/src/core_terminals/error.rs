#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error(transparent)]
    Core(#[from] sheprd_core_client::Error),
    #[error("No separately installed herdr-core was found on PATH")]
    NotInstalled,
    #[error("Could not start a worker for the terminal: {0}")]
    Worker(#[source] std::io::Error),
    // keep-sorted end
}
