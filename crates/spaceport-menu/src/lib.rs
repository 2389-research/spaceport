//! The menu model below the app: what a popup is showing, how its layer comes
//! and goes, the workspace, tab and pane a row targets, the typed request each
//! row becomes, and the rows the device picker and its sessions offer.
//!
//! The app keeps the window's aggregate menu state, every word a page says and
//! the painting, and words an action by matching its variant (`docs/i18n.md`).
//! Nothing here reads a window, and nothing here says a sentence a person
//! reads: a target is captured from a snapshot, a row that sends anything names
//! it explicitly, and a row that paints carries the words the app built.

pub mod appear;
mod error;
pub mod page;
pub mod pane;
pub mod picker;
pub mod sessions;
pub mod tab;
pub mod workspace;
mod worktree;

pub use error::{Error, Result};

#[cfg(test)]
mod tests;
