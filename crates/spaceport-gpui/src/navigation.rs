//! Where a focus request is aimed, as the palette's ranking crate holds it.
//!
//! A Go To row carries one, which is why it lives below the app; it is
//! expected to move again with the live-state activation program.

pub(crate) use spaceport_palette::{NavigationTarget, OwnedNavigationTarget};
