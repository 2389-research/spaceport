//! The app's text fields: the single-line field a search box is built from
//! ([`search_input`]), the single-line draft a dialog is typed into
//! ([`dialog_input`]), the multi-line composer that keeps the newlines
//! ([`multiline_input`]), the editing model every field shares — a grapheme-
//! and bidi-aware caret and selection ([`caret`]), the UTF-16 mapping the
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

pub mod actions;
pub mod caret;
pub mod dialog_input;
pub mod input;
pub mod locale;
pub mod multiline_input;
pub mod search_input;
pub mod text_editing;
