//! The keymap below the app: Herdr's `[keys]` table as this client reads it,
//! and the host OS a keystroke's text is spelled for.
//!
//! [`daemon`] reads the daemon's `[keys]` table — its prefixes, the actions it
//! binds, the `[keys.indexed]` combos over the digits, and the keys that move
//! the workspace picker — into the commands and keystrokes this client runs,
//! and turns Herdr's `+`-separated spelling into GPUI's. [`parse_keystroke`],
//! [`unparse`](gpui::Keystroke::unparse)'s caller and [`secondary`] name the OS
//! it was built for.
//!
//! Nothing here holds a window, an element, an entity, a message or a word: it
//! names GPUI's three keystroke values and its parse failure, and none of GPUI's
//! runtime. If a keystroke value type ever lands below GPUI — the terminal's
//! input encoder is the only candidate — this crate can drop its `gpui` edge
//! without changing a consumer, because it only ever names `Keystroke`.
//!
//! The rest of the keymap (the catalog's defaults, the GUI's `[keybindings]`
//! layered over them, and the `[[keys.command]]` reach) is still the app's.

pub mod daemon;
mod error;
mod host;

pub use error::{Error, Result};
#[cfg(any(test, feature = "test-support"))]
pub use host::secondary_key;
pub use host::{HOST_OS, parse_keystroke, secondary};
