//! Where a focus request is aimed: the shared closed set of targets,
//! re-exported at the path this crate, the palette and the app name it by.
//!
//! The set and the rules over a snapshot live in `spaceport-client`, which the
//! phone reads too; the activation fence carries one, which is what brought it
//! below the window.

pub use spaceport_client::navigation::{NavigationTarget, OwnedNavigationTarget};
