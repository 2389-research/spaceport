//! Showing another host: `error-endpoint.ftl`, and the status bar's own
//! line from `endpoint.ftl`.

use super::Localize;
use crate::{endpoint::Error, i18n::fl};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::NoHost { id } => fl!("error-endpoint-no-host", id = id.as_str()),
            Self::SurfaceSwitchingUnsupported => fl!("endpoint-surface-switching-unsupported"),
            // keep-sorted end
        }
    }
}
