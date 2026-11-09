use std::io;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Why a provider's usage could not be read. The display text is what the
/// panel shows, so none of it carries account details.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("{0}")]
    Client(#[from] sheprd_client::Error),
    #[error("Could not reach the usage service from this host.")]
    Connect,
    #[error("This account has no plan with usage limits to show.")]
    NoPlan,
    #[error("Could not {operation}.")]
    Process {
        operation: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("Usage is rate limited.")]
    RateLimited,
    #[error("Usage output exceeded the size limit.")]
    Size,
    #[error("Usage service returned HTTP {0}.")]
    Status(u16),
    #[error("Usage check timed out.")]
    Timeout,
    #[error("Usage unavailable")]
    Unavailable,
    #[error("Could not reach this host over SSH to read usage.")]
    Unreachable,
    /// WSL shells start through `wsl.exe`, which only a Windows client has.
    #[error("WSL distributions can only be reached from Windows.")]
    WslHostUnsupported,
    // keep-sorted end
}
