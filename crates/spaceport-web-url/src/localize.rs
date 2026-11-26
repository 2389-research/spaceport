//! The address failure as the person using the app reads it, from
//! `error-browser.ftl`, which this crate's errors share with the browser's.

use crate::Error;
use spaceport_i18n::{Localize, fl};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            Self::InvalidUrl => fl!("error-browser-url"),
        }
    }
}

#[cfg(test)]
mod tests;
