//! Teleport's typed errors enter through the remote crate's root error.

use super::Error;

impl From<sheprd_remote::teleport::Error> for Error {
    fn from(error: sheprd_remote::teleport::Error) -> Self {
        sheprd_remote::Error::from(error).into()
    }
}
