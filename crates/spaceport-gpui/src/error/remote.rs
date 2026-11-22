//! Host selection errors enter through the remote crate's root error.

use super::Error;

impl From<spaceport_remote::host_script::Error> for Error {
    fn from(error: spaceport_remote::host_script::Error) -> Self {
        spaceport_remote::Error::from(error).into()
    }
}
