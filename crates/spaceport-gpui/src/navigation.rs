// ABOUTME: Where a focus request is aimed, at the path the window names it by.
// ABOUTME: The closed set of targets and its snapshot rules live in spaceport-client.
//! Where a focus request is aimed: a closed set of targets, so navigation is
//! never a boolean-plus-ID pair or a string tag. The set and the rules over a
//! snapshot live in `spaceport-client`, which the phone reads too; the live
//! state's activation fence and the app reach it here.
pub(crate) use spaceport_live::{NavigationTarget, OwnedNavigationTarget};
