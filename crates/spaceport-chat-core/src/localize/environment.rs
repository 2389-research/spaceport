//! The `[chat.env]` table a chat's tab is created with, from `error-chat.ftl`.

use crate::env;
use spaceport_i18n::{Localize, fl};

impl Localize for env::Error {
    fn localize(&self) -> String {
        match self {
            Self::Name { name } => fl!("error-chat-environment-name", name = name.as_str()),
            Self::Size => fl!(
                "error-chat-environment-size",
                count = env::MAX_ENV,
                bytes = env::MAX_ENV_BYTES
            ),
        }
    }
}
