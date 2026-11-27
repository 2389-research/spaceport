//! The menu model below the app: what a popup is showing, how its layer comes
//! and goes, the workspace, tab and pane a row targets, the typed request each
//! row becomes, the rows the device picker and its sessions offer, the rows a
//! page paints — and the chrome it all draws in, from the panel a page sits in
//! to the in-window menu bar a Linux or Windows window draws for itself.
//!
//! The app keeps the window's aggregate menu state, every word a page says and
//! every folded form among its pages, and words an action by matching its
//! variant (`docs/i18n.md`). Nothing here reads a window, and nothing here says
//! a sentence a person reads: a target is captured from a snapshot, a row that
//! sends anything names it explicitly, a row that paints carries the words the
//! app built, and the chrome around a page takes the page's own element from
//! the app.

// 2026-11-27 quality gate: 3 functions here are over clippy.toml's 100-line
// ceiling. Split them, then delete this with the crate's entry in
// docs/notes-quality-gate-sweep.md.
#![allow(clippy::too_many_lines)]

pub mod appear;
pub mod bar;
pub mod chrome;
mod error;
pub mod group;
pub mod host;
mod localize;
pub mod page;
pub mod page_title;
pub mod pane;
pub mod picker;
pub mod rows;
pub mod sessions;
pub mod tab;
pub mod workspace;
mod worktree;

pub use error::{Error, Result};

#[cfg(test)]
mod tests;
