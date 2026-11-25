//! Why the keymap could not be read. The variants name what could not be read;
//! the app words them (`docs/i18n.md`), and these messages are for logs and
//! `Display`.

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error(transparent)]
    Daemon(#[from] crate::daemon::Error),
    // keep-sorted end
}
