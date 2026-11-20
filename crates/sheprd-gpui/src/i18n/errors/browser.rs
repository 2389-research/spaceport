//! Browser tab and saved-layout failures, from `error-browser.ftl`. The JSON
//! parser's own text stays as it is.

use super::Localize;
use crate::i18n::fl;
use sheprd_browser::Error;

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            Self::InvalidBrowserTabs => fl!("error-browser-tabs"),
            Self::InvalidBrowserUrl => fl!("error-browser-url"),
            Self::InvalidCodeFile => fl!("error-browser-code-file"),
            Self::InvalidGroupLayouts => fl!("error-browser-group-layouts"),
            Self::InvalidLocalPage => fl!("error-browser-local-page"),
            Self::InvalidReviewCheckout => fl!("error-browser-review-checkout"),
            Self::Json(error) => error.to_string(),
            Self::StateFile(error) => error.localize(),
        }
    }
}
