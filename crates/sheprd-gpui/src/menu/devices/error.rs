//! Why removing or renaming a saved device failed.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("This device is already being removed. Wait for that to finish, then try again.")]
    Busy,
    #[error("Device names are at most 128 bytes, without control characters.")]
    InvalidLabel,
    #[error("This device has no saved profile.")]
    NoProfile,
    #[error("Removing the device failed ({status}){}", if detail.is_empty() { String::new() } else { format!(": {detail}") })]
    Remove {
        status: std::process::ExitStatus,
        detail: String,
    },
    #[error("Renaming the device failed ({status}){}", if detail.is_empty() { String::new() } else { format!(": {detail}") })]
    Rename {
        status: std::process::ExitStatus,
        detail: String,
    },
    #[error("Saving the device timed out")]
    SetupTimeout,
    // keep-sorted end
}
