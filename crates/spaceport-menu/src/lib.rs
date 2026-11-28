//! The menu model below the app: what a popup is showing, how its layer comes
//! and goes, the workspace, tab and pane a row targets, the typed request each
//! row becomes, the rows the device picker and its sessions offer, the rows a
//! page paints — the Keyboard Shortcuts page's sections among them — and the
//! chrome it all draws in, from the panel a page sits in to the in-window menu
//! bar a Linux or Windows window draws for itself.
//!
//! The app keeps the window's aggregate menu state, every word a page says and
//! every folded form among its pages, and words an action by matching its
//! variant (`docs/i18n.md`). Nothing here reads a window, and nothing here says
//! a sentence a person reads: a target is captured from a snapshot, a row that
//! sends anything names it explicitly, a row that paints carries the words the
//! app built, and the chrome around a page takes the page's own element from
//! the app.

pub mod appear;
pub mod bar;
pub mod chrome;
mod error;
pub mod form;
pub mod group;
pub mod host;
pub mod keybinds;
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
