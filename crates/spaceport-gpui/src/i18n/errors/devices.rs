//! Device menu failures, from `error-devices.ftl`.

use super::Localize;
use crate::{i18n::fl, menu::devices::Error};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            Self::Busy => fl!("error-devices-busy"),
            Self::InvalidLabel => fl!("error-devices-setup-label"),
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
