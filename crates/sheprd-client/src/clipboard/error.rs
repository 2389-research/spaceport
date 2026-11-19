// ABOUTME: Describes failures preparing or sending clipboard content to a terminal.
// ABOUTME: Preserves retry and cancellation categories independently of message wording.

use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("Another clipboard paste is in progress. Wait for it to finish.")]
    Busy,
    #[error("clipboard image upload cancelled")]
    Cancelled,
    #[error("This terminal connection cannot paste this clipboard content.")]
    InputTarget,
    #[error("Preparing the clipboard content took too long. It was not pasted. Try pasting again.")]
    PreparationTimeout,
    #[error(
        "Spaceport could not finish sending clipboard input. Some input may have arrived. Choose Terminal → Reconnect."
    )]
    WriteTimeout,
    // keep-sorted end
}

impl Error {
    /// The transport retry/cancellation category `crate::Error::kind` reports.
    pub(crate) fn kind(&self) -> io::ErrorKind {
        match self {
            Self::Busy => io::ErrorKind::WouldBlock,
            Self::Cancelled => io::ErrorKind::Interrupted,
            Self::PreparationTimeout | Self::WriteTimeout => io::ErrorKind::TimedOut,
            Self::InputTarget => io::ErrorKind::InvalidData,
        }
    }
}
