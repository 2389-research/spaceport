// ABOUTME: Defines typed failures when a window cannot show another host.
// ABOUTME: Gives connection failures readable diagnostics without changing their identity.
//! Why a window could not show another host.

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("No enabled host has the ID {id}")]
    NoHost { id: String },
    #[error("This connection cannot switch terminal screens")]
    SurfaceSwitchingUnsupported,
    // keep-sorted end
}
