//! The chat model below the app's views: which existing agent conversations a
//! window may attach to, the `[chat.env]` table a chat's tab is created with,
//! and how the measured entry list follows a transcript. The transcript's own
//! wire types are [`spaceport_chat_protocol`]'s.
//!
//! Everything here is a pure function of its inputs or hands its blocking work
//! to a worker thread, so it is tested without a window. The app keeps the rest
//! of its chat: the views, the messages they write, the connection worker and
//! the session state it applies to, and every piece of window state those read.

mod error;

pub mod env;
pub mod existing;
pub mod list;

pub use error::{Error, Result};

use spaceport_chat_protocol::{EndpointId, TOKEN};

/// Chat panes attached at once; more are left to their terminal views.
pub const MAX_CHATS: usize = 16;

/// The program a pane's metadata names, when its token holds a valid id.
pub fn endpoint(tokens: &[(String, String)]) -> Option<EndpointId> {
    tokens
        .iter()
        .find(|(name, _)| name == TOKEN)
        .and_then(|(_, value)| value.parse().ok())
}
