//! The command palette's pure model.
//!
//! What a row is — its label, detail, badge, the destination choosing it
//! reaches, and the identity that makes two rows the same place — which rows a
//! filter lists, how a typed query scores and highlights them (nucleo's fzf
//! algorithm over bounded haystacks), Go To's agent-status filter and the
//! heading order it restores, the destination context every request is fenced
//! by, the key a row's frecency is remembered under, and the errors a chosen
//! row answers with.
//!
//! Nothing here builds an entry, paints a row, or says a word a person reads:
//! the app builds every entry from its own catalog and its daemon state, words
//! every label and badge (`docs/i18n.md`), keeps the palette entity, the
//! frecency history, the project scan and the paint. The navigation target a
//! Go To row aims at also lives here for now; its expected home is the
//! live-state activation program.

mod action;
mod agent_filter;
mod entry;
mod error;
mod filter;
mod frecency;
mod navigation;
mod project;
mod rank;
pub mod search;
mod target;

pub use action::{Action, Identity};
pub use agent_filter::AgentFilter;
pub use entry::Entry;
pub use error::{Error, Result};
pub use filter::Filter;
pub use frecency::{Key, MAX_FIELD_BYTES};
pub use navigation::{NavigationTarget, OwnedNavigationTarget};
pub use project::{NewProject, Project};
pub use rank::rank_rows;
pub use target::Target;

#[cfg(test)]
mod tests;
