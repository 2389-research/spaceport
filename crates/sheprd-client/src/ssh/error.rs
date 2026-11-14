//! Why reaching a host over SSH failed.

use super::SshFailure;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("SSH connection cancelled")]
    Cancelled,
    #[error("SSH bridge closed; check host trust, authentication, and remote Herdr installation")]
    Closed,
    #[error("invalid SSH target (options, controls, and passwords are forbidden)")]
    InvalidTarget,
    #[error("SSH startup output exceeds limit")]
    OutputLimit,
    /// `ssh` closed the bridge before it was ready, for the reason classified.
    #[error("SSH bridge closed: {0}")]
    Refused(#[source] SshFailure),
    #[error("SSH discovery timed out")]
    Timeout,
    #[error("SSH port forwarding is not supported on this platform")]
    Unsupported,
    // keep-sorted end
}

impl Error {
    /// The transport retry/cancellation category `crate::Error::kind` reports.
    pub(crate) fn kind(&self) -> io::ErrorKind {
        match self {
            Self::InvalidTarget => io::ErrorKind::InvalidInput,
            Self::Unsupported => io::ErrorKind::Unsupported,
            Self::Cancelled => io::ErrorKind::Interrupted,
            Self::Closed => io::ErrorKind::UnexpectedEof,
            Self::Refused(failure) => match failure {
                SshFailure::HostKey | SshFailure::Auth => io::ErrorKind::PermissionDenied,
                SshFailure::Unreachable | SshFailure::NoRoute | SshFailure::LocalNetworkDenied => {
                    io::ErrorKind::NotConnected
                }
                SshFailure::HerdrMissing => io::ErrorKind::NotFound,
                SshFailure::Other => io::ErrorKind::UnexpectedEof,
            },
            Self::Timeout => io::ErrorKind::TimedOut,
            Self::OutputLimit => io::ErrorKind::InvalidData,
        }
    }
}
