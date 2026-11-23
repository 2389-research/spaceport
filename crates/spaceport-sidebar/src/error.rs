//! Why the sidebar's local state could not be written.

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error(transparent)]
    DeviceTags(#[from] crate::device_tags::Error),
    // keep-sorted end
}
