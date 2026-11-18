//! Starting or talking to the Herdr daemon: `error-daemon.ftl`.

use super::Localize;
use crate::{daemon::Error, i18n::fl};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::Cancelled => fl!("error-daemon-cancelled"),
            // The system's words for how the daemon ended.
            Self::Exited(status) => fl!("error-daemon-exited", status = status.to_string()),
            Self::Spawn { source } => fl!("error-daemon-spawn", error = source.to_string()),
            Self::Timeout(path) => fl!("error-daemon-timeout", path = path.display().to_string()),
            // keep-sorted end
        }
    }
}
