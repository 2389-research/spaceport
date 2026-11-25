//! The keymap below the app: which command a keystroke runs, and what else the
//! app keeps for itself.
//!
//! [`keymap`] resolves the one answer the app's keys and labels follow — the
//! catalog's defaults, the daemon's `[keys]` table, the GUI's `[keybindings]`
//! and the `[pane_keys]` a pane takes — and [`custom`] matches the daemon's
//! `[[keys.command]]` shortcuts against it when they are typed. [`daemon`]
//! reads Herdr's `[keys]` table, [`reserved`] holds the keystrokes the app
//! itself binds, and [`pane`] says whether a keystroke can reach a pane at all.
//! [`parse_keystroke`], [`unparse`](gpui::Keystroke::unparse)'s caller and
//! [`secondary`] name the OS they were built for.
//!
//! Nothing here holds a window, an element, an entity, a message or a word: it
//! names GPUI's three keystroke values and its parse failure, and none of GPUI's
//! runtime. If a keystroke value type ever lands below GPUI — the terminal's
//! input encoder is the only candidate — this crate can drop its `gpui` edge
//! without changing a consumer, because it only ever names `Keystroke`.

mod custom;
pub mod daemon;
mod error;
mod host;
mod keymap;
mod pane;
mod reserved;

pub use custom::{CustomBinding, Reach, available_commands};
pub use error::{Error, Result};
#[cfg(any(test, feature = "test-support"))]
pub use host::secondary_key;
pub use host::{HOST_OS, parse_keystroke, secondary};
pub use keymap::{Keymap, PaneKey};
#[cfg(any(test, feature = "test-support"))]
pub use keymap::{identity, typed_matches};
pub use pane::{key_code, reaches_pane};
#[cfg(any(target_os = "macos", feature = "test-support"))]
pub use reserved::free_system_keys;
pub use reserved::{EDIT_KEYS, SYSTEM_KEYS, reserved_keystrokes};
