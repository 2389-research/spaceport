//! Why a window could not show another host.

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("No enabled host has the ID {id}")]
    NoHost { id: String },
    #[error("Current endpoint does not support surface switching")]
    SurfaceSwitchingUnsupported,
    // keep-sorted end
}
