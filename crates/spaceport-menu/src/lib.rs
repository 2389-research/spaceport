//! The menu model below the app: what a popup is showing, the workspace, tab
//! and pane a row targets, and the typed request each row becomes.
//!
//! The app keeps the window's aggregate menu state, every word a page says and
//! the painting, and words an action by matching its variant (`docs/i18n.md`).
//! Nothing here reads a window, and nothing here says a sentence a person
//! reads: a target is captured from a snapshot, and a row that sends anything
//! names it explicitly.

mod error;
pub mod page;
pub mod pane;
pub mod tab;
pub mod workspace;
mod worktree;

pub use error::{Error, Result};

#[cfg(test)]
mod tests;
