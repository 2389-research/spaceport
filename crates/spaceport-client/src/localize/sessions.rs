//! `spaceport_client::sessions::Error`: listing or deleting a machine's named
//! sessions, from `error-client-sessions.ftl`.

use crate::sessions::Error;
use spaceport_i18n::{Localize, fl};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::DeleteDefault => fl!("error-client-sessions-delete-default"),
            Self::DeleteFailed(status) => fl!(
                "error-client-sessions-delete-failed",
                status = status.to_string()
            ),
            Self::DeleteTimeout => fl!("error-client-sessions-delete-timeout"),
            Self::Limit => fl!("error-client-sessions-limit"),
            Self::NotRunning => fl!("error-client-sessions-not-running"),
            // keep-sorted end
        }
    }
}
