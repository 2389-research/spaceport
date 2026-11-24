//! The chrome's shared widget kit: the logical start and end sides every styled
//! element is given ([`direction`]), the selection marks dialogs, menus and
//! settings draw ([`toggles`]), and the AccessKit helpers GPUI's own setters
//! cannot express ([`accessibility`]).
//!
//! No window, no message and no words: a caller owns the row, its label, its
//! click handler and every sentence it shows. The direction is the one thing
//! kept here, because an element is mirrored where it is built, without an app
//! context; the app publishes the shown locale's direction with
//! [`direction::set`] as it switches the catalog.

pub mod accessibility;
pub mod direction;
pub mod toggles;
