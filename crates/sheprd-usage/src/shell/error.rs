// ABOUTME: Why a bounded host shell could not start or finish a step.
// ABOUTME: Worded about the shell alone, since any feature may run steps through it.
use std::io;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The step a [`Error::Process`] names when the shell itself did not start.
pub const START: &str = "start the host shell";

/// Why a host shell could not start or finish a step. The display text is
/// what a window shows beneath the feature's own failure.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("{0}")]
    Client(#[from] sheprd_client::Error),
    #[error("Could not {operation}.")]
    Process {
        operation: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("Host shell output exceeded the size limit.")]
    Size,
    #[error("Host shell step timed out.")]
    Timeout,
    #[error("Could not reach a shell on this host.")]
    Unreachable,
    /// WSL shells start through `wsl.exe`, which only a Windows client has.
    #[error("WSL distributions can only be reached from Windows.")]
    WslHostUnsupported,
    // keep-sorted end
}
