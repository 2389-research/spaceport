pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Why usage could not be observed on a host. No provider is read here, so
/// the only failures are the host shell's.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error(transparent)]
    Shell(#[from] crate::shell::Error),
    // keep-sorted end
}
