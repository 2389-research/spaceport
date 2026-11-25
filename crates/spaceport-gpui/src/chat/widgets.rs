//! A small, declarative native UI catalog. Agent JSON is data: it cannot name
//! code, URLs, tools, or commands. Only an explicit action click prepares a
//! normal chat prompt containing the user's current form values.
//!
//! The catalog read from that JSON is [`spaceport_chat_core::widgets`]'s, and
//! this module re-exports it at the paths the rest of the chat reads it at.
//! What stays here is the state a user's edits live in: the retained form
//! values per entry and card, and the native inputs holding them.

mod state;

pub(crate) use spaceport_chat_core::widgets::{
    ActionMode, Component, Document, Key, MAX_INPUT, Point, Value, Values, slider_value, snap,
};
pub(crate) use state::{Form, Input, State, bounded_input};
// The catalog's own failures word themselves; only the parse tests name the type.
#[cfg(test)]
pub(crate) use spaceport_chat_core::widgets::Error;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
