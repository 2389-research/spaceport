// ABOUTME: Describes host targets the installed Herdr CLI cannot address.
// ABOUTME: Keeps unsupported routes distinct from development-session refusal.

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "Host scripts need a local session or SSH host; custom socket and WSL endpoints are unsupported"
    )]
    UnsupportedHost,
    /// The CLI would address the release session with the same name.
    #[error("Host scripts cannot reach a development (--dev) session")]
    DevelopmentSession,
}
