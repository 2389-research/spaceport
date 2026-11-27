// ABOUTME: Words saved tab, layout, and reference failures for the window.
// ABOUTME: Gives repair guidance while preserving parser and file diagnostics.
//! Browser tab and saved-layout failures as the person using the app reads
//! them, from `error-browser.ftl`. The JSON parser's own text stays as it is,
//! and so does a wrapped state file's, which words itself.
// i18n: migrated

use crate::Error;
use spaceport_i18n::{Localize, fl};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            Self::InvalidBrowserTabs => fl!("error-browser-tabs"),
            Self::InvalidGroupLayouts => fl!("error-browser-group-layouts"),
            Self::Json(error) => error.to_string(),
            Self::RestorationBlocked => fl!("error-browser-restoration-blocked"),
            Self::StateFile(error) => error.localize(),
            Self::TabIdsExhausted => fl!("error-browser-tab-ids-exhausted"),
        }
    }
}

#[cfg(test)]
mod tests;
