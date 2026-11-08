//! `herdr-remote`'s root `Error` from `error-remote.ftl`, including
//! Teleport's errors from `error-remote-teleport.ftl`.

mod teleport;

use super::Localize;
use crate::i18n::fl;
use sheprd_remote::Error;

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::Client(error) => error.localize(),
            // The system's own words, as `Json` is its parser's.
            Self::Io(error) => error.to_string(),
            Self::Json(error) => error.to_string(),
            Self::Path(error) => error.localize(),
            Self::PreferencesPath => fl!("error-remote-preferences-path"),
            Self::Teleport(error) => error.localize(),
            // keep-sorted end
        }
    }
}
