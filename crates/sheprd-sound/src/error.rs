//! Why a notification sound did not play or its settings did not load.

use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Sound configuration exceeds 1 MiB")]
    ConfigSize,
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("Could not load sound settings from {}: {source}", path.display())]
    Path {
        path: PathBuf,
        #[source]
        source: Box<Error>,
    },
    #[error("{0}")]
    Toml(#[from] toml::de::Error),
}

impl Error {
    /// This failure, as one concerning `path`.
    pub(crate) fn at_path(self, path: &Path) -> Self {
        Self::Path {
            path: path.to_owned(),
            source: Box::new(self),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
