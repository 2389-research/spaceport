//! Host selection errors enter through the remote crate's root error.

use super::Error;

impl From<sheprd_remote::host_script::Error> for Error {
    fn from(error: sheprd_remote::host_script::Error) -> Self {
        sheprd_remote::Error::from(error).into()
    }
}
