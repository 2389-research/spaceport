/// Why a chat's page could not be shown.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("no folder for pages")]
    Folder,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("not a file path")]
    NotFile,
    #[error("the page is too large")]
    TooLarge,
    // keep-sorted end
}
