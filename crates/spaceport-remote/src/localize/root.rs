//! ABOUTME: Localizes typed failures from the shared host script routes.
//! ABOUTME: Keeps unsupported targets and development sessions distinct.

use crate::{Error, host_script};
use spaceport_i18n::{Localize, fl};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            Self::HostScript(error) => error.localize(),
            Self::Setup(error) => error.localize(),
        }
    }
}

impl Localize for host_script::Error {
    fn localize(&self) -> String {
        match self {
            Self::UnsupportedHost => fl!("error-host-script-unsupported-host"),
            Self::DevelopmentSession => fl!("error-host-script-development-session"),
        }
    }
}
