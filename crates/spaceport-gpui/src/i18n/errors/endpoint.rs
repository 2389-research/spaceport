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

/// The endpoint crate's failures, which this window shows as a notice when the
/// saved devices cannot be read or the choice of one cannot be kept: the
/// client's own storage step through its mapping, and the system's own words
/// for a worker that could not start.
impl Localize for spaceport_endpoint::Error {
    fn localize(&self) -> String {
        match self {
            Self::Client(error) => error.localize(),
            Self::Worker(error) => error.to_string(),
        }
    }
}
