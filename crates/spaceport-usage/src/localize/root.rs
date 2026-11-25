//! Usage observation errors: the host shell's, which keep their own messages.

use crate::Error;
use spaceport_i18n::Localize;

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::Shell(error) => error.localize(),
            // keep-sorted end
        }
    }
}
