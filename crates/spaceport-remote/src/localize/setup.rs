//! Device menu failures, from `error-devices.ftl`.

use crate::setup::Error;
use spaceport_i18n::{Localize, fl};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            Self::Busy => fl!("error-devices-busy"),
            Self::Client(error) => error.localize(),
            Self::InvalidLabel => fl!("error-devices-setup-label"),
            Self::Io(error) => error.to_string(),
            Self::NoProfile => fl!("error-devices-setup-no-profile"),
            Self::Remove { status, detail } if detail.is_empty() => {
                fl!("error-devices-remove", status = status.to_string())
            }
            Self::Remove { status, detail } => fl!(
                "error-devices-remove-detail",
                status = status.to_string(),
                detail = detail.as_str()
            ),
            Self::Rename { status, detail } if detail.is_empty() => {
                fl!("error-devices-rename", status = status.to_string())
            }
            Self::Rename { status, detail } => fl!(
                "error-devices-rename-detail",
                status = status.to_string(),
                detail = detail.as_str()
            ),
            Self::SetupTimeout => fl!("error-devices-setup-timeout"),
        }
    }
}
