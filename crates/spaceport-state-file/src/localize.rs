//! State-file failures as the person using the app reads them, from
//! `error-state-file.ftl`. The system's and the JSON parser's own text stays
//! as it is.
// i18n: migrated

use crate::Error;
use spaceport_i18n::{Localize, fl};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            Self::Io(error) => error.to_string(),
            Self::Json(error) => error.to_string(),
            Self::MissingRoot(path) => fl!(
                "error-state-file-missing-root",
                path = path.display().to_string()
            ),
            Self::Size { path, limit } => {
                fl!(
                    "error-state-file-size",
                    path = path.display().to_string(),
                    limit = *limit
                )
            }
        }
    }
}

#[cfg(test)]
mod tests;
