//! A connection's live state, below the window that drives it.
//!
//! What one connection to a Herdr daemon last said — its snapshot, its
//! surface, its notices, the mailboxes the window drains — and the rules over
//! it: which ordered client events change what, when a snapshot and a surface
//! belong to each other, and the activation fence that keeps navigation from
//! acting on a projection the daemon has not acknowledged. The window's own
//! copy of the selected endpoint's state stays the window's; this crate only
//! holds the value and the rules, and the app decides which copy a view reads.
//!
//! Nothing here paints, opens a socket, or names its words elsewhere: a notice
//! words itself through `spaceport-i18n`, and the two things it cannot — an
//! agent notice's task and place, and a client error's wording — arrive as the
//! `fn` items in [`words`]. Every effect — the pasteboard, a bell, a sound, a
//! toast — is the app's.

// 2026-11-27 quality gate: 2 functions here are over clippy.toml's 100-line
// ceiling and 1 over its cognitive-complexity ceiling. Split them, then delete
// this with the crate's entry in docs/notes-quality-gate-sweep.md.
#![allow(clippy::cognitive_complexity, clippy::too_many_lines)]

mod agent_recency;
pub mod notice;
pub mod notifications;
mod projection;
mod state;
pub mod words;

pub use agent_recency::AgentRecency;
pub use notice::{Cause, HeldNotice, Notices};
pub use notifications::{Notice, PENDING_LIMIT, Toasts, VISIBLE_LIMIT};
pub use spaceport_activation as activation;
pub use spaceport_activation::{Failure, Fence, SurfaceActivation, TIMEOUT, Transition};
pub use spaceport_activation::{NavigationTarget, OwnedNavigationTarget};
pub use state::{
    AnnouncementDismissal, Answer, ConnectionStatus, DialogResponse, LiveState,
    MAX_DAEMON_VERSION_CHARS, MAX_WINDOW_TITLE_CHARS, RenameResult,
};
