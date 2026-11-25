//! The failures this crate returns, as the person using the app reads them
//! (docs/i18n.md), beside the `Error` it words. Every enum keeps its English
//! `Display` for logs, and an enum wrapping another crate's error calls that
//! error's own `localize()`, so each failure has one message as it has one
//! definition. The messages are `error-endpoint.ftl`.

use crate::Error;
use spaceport_i18n::Localize;

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            Self::Client(error) => error.localize(),
            Self::Worker(error) => error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests;
