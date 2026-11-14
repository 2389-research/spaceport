// ABOUTME: Classifies invalid observation inputs before reading app state.
// ABOUTME: Keeps request validation errors typed at the shared protocol boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Observation bounds must be positive and the offset must be bounded")]
    InvalidBounds,
    #[error("Observation target must be an app-issued identifier")]
    InvalidTarget,
}
