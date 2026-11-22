//! Why a notification sound did not play or its settings did not load.

use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Audio playback cancelled")]
    Cancelled,
    #[error("Sound configuration exceeds 1 MiB")]
    ConfigSize,
    #[error("Could not decode MP3 sound: {0}")]
    Decode(#[from] rodio::decoder::DecoderError),
    #[error("Could not open audio output: {0}")]
    Device(#[from] rodio::DeviceSinkError),
    #[error("Could not read sound file {}: {source}", path.display())]
    File {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Sound must be a regular file of at most 16 MiB")]
    FileSize,
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{}: {source}", path.display())]
    Path {
        path: PathBuf,
        #[source]
        source: Box<Error>,
    },
    #[error("Audio output failed: {0}")]
    Stream(#[from] rodio::cpal::StreamError),
    #[error("Audio playback timed out")]
    Timeout,
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

    /// What failed, without the message: a log line says which part of sound
    /// broke while leaving out what a message can quote. A settings load
    /// failure names the file's home path and, for a TOML error, the offending
    /// line of the user's own config, so the worker logs this instead.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::ConfigSize => "config-size",
            Self::Decode(_) => "decode",
            Self::Device(_) => "device",
            Self::File { .. } => "file",
            Self::FileSize => "file-size",
            Self::Io(_) => "io",
            // The path is the context; the failure is the source's kind.
            Self::Path { source, .. } => source.kind(),
            Self::Stream(_) => "stream",
            Self::Timeout => "timeout",
            Self::Toml(_) => "toml",
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
