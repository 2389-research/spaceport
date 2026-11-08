//! Why the remaining dispatch history could not be read.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("Invalid saved dispatch history")]
    InvalidHistory,
    // keep-sorted end
}
