//! State-file failures, from `error-state-file.ftl`. The system's and the
//! JSON parser's own text stays as it is.

use super::Localize;
use crate::i18n::fl;
use spaceport_state_file::Error;

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
