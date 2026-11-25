//! The keymap's paths for the app. `spaceport-keymap` resolves the keystrokes
//! bound to each catalog command and matches the daemon's custom shortcuts; the
//! re-exports below keep the names the app's windows, menus and CLI have always
//! used (`docs/notes-keymap-extraction.md`). What is still the app's is the
//! keys it reserves for its own menus (`crate::actions`) and how a keystroke
//! reaches a pane (`crate::terminal`), so only those two tests stay here.

pub use spaceport_keymap::daemon;
pub use spaceport_keymap::daemon::DaemonKeys;
#[cfg(any(test, feature = "integration-test"))]
pub use spaceport_keymap::parse_keystroke;
pub use spaceport_keymap::{Error, HOST_OS, Keymap, Reach, available_commands, secondary};
#[cfg(test)]
pub use spaceport_keymap::{identity, secondary_key, typed_matches};
pub use spaceport_settings::keys::{Binding, PaneKeys};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
