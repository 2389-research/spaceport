// ABOUTME: Where a focus request is aimed, at the path the window names it by.
// ABOUTME: The closed set of targets lives in spaceport-live, whose activation fence uses it.
//! Where a focus request is aimed: a closed set of targets, so navigation is
//! never a boolean-plus-ID pair or a string tag. The live state's activation
//! fence carries one, which is what brings it to `spaceport-live`; the app
//! reaches it here.
pub(crate) use spaceport_live::{NavigationTarget, OwnedNavigationTarget};
