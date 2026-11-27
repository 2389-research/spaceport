//! The chat model below the app's views: which existing agent conversations a
//! window may attach to, which daemon's chat programs it may reach and how,
//! the `[chat.env]` table a chat's tab is created with,
//! how the measured entry list follows a transcript, the attach side itself —
//! the transcript a session holds and the worker that reads it — the syntax
//! spans of the code a transcript shows, the identity, bounds and diagram theme
//! of the pictures it shows, how long a step or turn took, which of an agent's
//! answers is a compaction summary, the block model a message's Markdown is
//! parsed into with the parse that asks for the visuals a message draws, the
//! entries a view draws out of that — their parts, the pictures they ask for,
//! and the words written into them, which this crate reads from
//! `spaceport-i18n` — the selectable text of a transcript and
//! the selection over it, and the catalog agent JSON is read into.
//! The transcript's own wire types are [`spaceport_chat_protocol`]'s.
//!
//! Everything here is a pure function of its inputs or hands its blocking work
//! to a worker thread, so it is tested without a window. The app keeps the rest
//! of its chat: the views, the messages they write, the classification of the
//! destinations it may act on, the pictures it draws, and every piece of window
//! state those read.

// 2026-11-27 quality gate: 1 function here is over clippy.toml's 100-line
// ceiling. Split it, then delete this with the crate's entry in
// docs/notes-quality-gate-sweep.md.
#![allow(clippy::too_many_lines)]

mod error;
mod localize;

pub mod discovery;
pub mod env;
pub mod existing;
pub mod highlight;
pub mod link;
pub mod list;
pub mod prepared;
pub mod rich;
pub mod selection;
pub mod session;
pub mod summary;
pub mod text;
pub mod visual;
pub mod widgets;

pub use error::{Error, Result};

use spaceport_chat_protocol::{EndpointId, TOKEN};

/// Chat panes attached at once; more are left to their terminal views.
pub const MAX_CHATS: usize = 16;

/// Commands waiting for their acknowledgement, per pane.
pub const MAX_PENDING: usize = 32;
/// Messages a worker holds for the window before it gives the connection up
/// as too far behind; the program then replays from the cursor.
pub const MAX_EVENTS: usize = 1024;

/// The program a pane's metadata names, when its token holds a valid id.
pub fn endpoint(tokens: &[(String, String)]) -> Option<EndpointId> {
    tokens
        .iter()
        .find(|(name, _)| name == TOKEN)
        .and_then(|(_, value)| value.parse().ok())
}
