// ABOUTME: Localizes saved-tab recovery refusals at the display boundary.
// ABOUTME: Shares the repair guidance between tab actions and terminal creation.
use super::Localize;
use crate::{browser::Error, i18n::fl};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            Self::Capacity => fl!("browser-too-many-tabs"),
            Self::RestorationBlocked => fl!("browser-state-needs-repair"),
        }
    }
}
