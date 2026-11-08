//! `herdr-vcs` failures: Git operations and GitHub requests.

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error(transparent)]
    Git(#[from] crate::git::Error),
    #[error(transparent)]
    GitHub(#[from] sheprd_github::Error),
    // keep-sorted end
}
