//! ABOUTME: Localizes typed failures from the shared host script routes.
//! ABOUTME: Keeps unsupported targets and development sessions distinct.

use super::Localize;
use crate::i18n::fl;
use sheprd_remote::{Error, host_script};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            Self::HostScript(error) => error.localize(),
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
