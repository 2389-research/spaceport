pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Why a saved tab or layout could not be read. Every one of them is a file
/// this app wrote, so a build that cannot read one starts without it rather
/// than failing the window.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("Invalid saved browser tabs")]
    InvalidBrowserTabs,
    #[error("Browser tabs open only http and https addresses with a host, of at most 8 KiB.")]
    InvalidBrowserUrl,
    #[error("An old source tab must name an absolute file path without control characters")]
    InvalidCodeFile,
    #[error("Invalid saved editor groups")]
    InvalidGroupLayouts,
    #[error(
        "Local pages must be a plain file in a folder that is not hidden and does not hold your home directory."
    )]
    InvalidLocalPage,
    #[error("A saved review tab names a checkout that is not a local absolute path.")]
    InvalidReviewCheckout,
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    StateFile(#[from] sheprd_state_file::Error),
    // keep-sorted end
}
