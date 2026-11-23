pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Why a value is not an address a page may be opened from: a scheme other
/// than http or https, no host, whitespace or a control character in it, or
/// more than 8 KiB of it.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Browser tabs open only http and https addresses with a host, of at most 8 KiB.")]
    InvalidUrl,
}
