//! Saved host history errors: `error-dispatch.ftl`.

use super::Localize;
use crate::{dispatch::Error, i18n::fl};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::InvalidHistory => fl!("error-dispatch-invalid-history"),
            // keep-sorted end
        }
    }
}
