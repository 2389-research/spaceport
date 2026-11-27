//! The app's text fields: the single-line field a search box is built from
//! ([`search_input`]), the editing model every field shares — a grapheme- and
//! bidi-aware caret and selection ([`caret`]), the UTF-16 mapping the
//! platform's input API speaks and the text newlines are normalized in
//! ([`text_editing`]), and which edit a key asks for ([`text_editing::keys`]) —
//! the platform input handler GPUI is handed ([`input`]), the app's notice that
//! the locale changed ([`locale`]), and the edit actions a field registers
//! ([`actions`]).
//!
//! No window, no message and no words: the caller owns the placeholder text,
//! the label a screen reader reads, the frame around the field, and what a key
//! that is not an edit means to it. The edit actions are declared once, here,
//! because GPUI dispatches an action by its type; the app re-exports them for
//! its Edit menu.

// 2026-11-27 quality gate: 1 function here is over clippy.toml's 100-line
// ceiling. Split it, then delete this with the crate's entry in
// docs/notes-quality-gate-sweep.md.
#![allow(clippy::too_many_lines)]

pub mod actions;
pub mod caret;
pub mod input;
pub mod locale;
pub mod search_input;
pub mod text_editing;
