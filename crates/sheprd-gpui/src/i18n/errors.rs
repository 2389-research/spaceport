//! Errors as the person using the app reads them (docs/i18n.md). Every error
//! enum that reaches a window keeps its `thiserror` English `Display` for
//! logs, and [`Localize`] picks its message by variant where it is shown.
//! An enum wrapping another crate's error delegates to that error's own
//! mapping, so each failure has one message, as it has one definition.
//!
//! The mappings live here, one file and one `error-<name>.ftl` per crate or
//! feature, rather than beside each enum: crates below the app have no
//! messages, and the app's feature modules are migrated on their own.
// i18n: migrated

mod backdrop;
mod caffeine;
mod chat;
mod chat_protocol;
mod client;
mod clipboard;
mod config;
mod control;
mod credentials;
mod daemon;
mod devices;
mod file_drop;
mod github;
mod keymap;
#[cfg(windows)]
mod launch_at_login;
mod listening_ports;
mod notification_mutes;
mod overview;
mod palette;
mod pane_move;
mod port_forward;
mod processes;
mod protocol;
mod remote;
mod root;
#[cfg(unix)]
mod setup_relay;
mod sound;
mod system_load;
#[cfg(test)]
pub(crate) mod tests;
mod updater;
mod usage;
mod worktree_identity;
mod worktree_scripts;
mod wsl_drop;

use std::sync::Arc;

/// An error's message in the current locale, for a window, a notification,
/// or an `$error` placeable. Logs keep `Display`.
pub(crate) trait Localize {
    fn localize(&self) -> String;
}

impl<T: Localize + ?Sized> Localize for &T {
    fn localize(&self) -> String {
        (**self).localize()
    }
}

impl<T: Localize + ?Sized> Localize for Box<T> {
    fn localize(&self) -> String {
        (**self).localize()
    }
}

impl<T: Localize + ?Sized> Localize for Arc<T> {
    fn localize(&self) -> String {
        (**self).localize()
    }
}

/// `error`'s message followed by each cause's, for a line that names what
/// went wrong on a host as well as what failed. A cause from the app is
/// localized; any other (the system's, a parser's) keeps its own words.
pub(crate) fn with_causes(error: &crate::Error) -> String {
    let mut text = error.localize();
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        text.push(' ');
        // A boxed cause is the box itself, as `#[source]` hands it over.
        let ours = cause
            .downcast_ref::<crate::Error>()
            .or_else(|| cause.downcast_ref::<Box<crate::Error>>().map(AsRef::as_ref));
        text.push_str(&ours.map_or_else(|| cause.to_string(), Localize::localize));
        source = cause.source();
    }
    text
}
