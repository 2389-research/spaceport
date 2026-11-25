//! The app's own errors as the person using the app reads them
//! (docs/i18n.md). Every error enum that reaches a window keeps its
//! `thiserror` English `Display` for logs, and [`Localize`] picks its message
//! by variant where it is shown. An enum wrapping another crate's error
//! delegates to that error's own mapping, so each failure has one message, as
//! it has one definition.
//!
//! A crate words its own errors, so each mapping lives in the crate that owns
//! the `Error` and its `error-<name>.ftl` stays in the one i18n tree. What is
//! here words the app's own errors, one file per feature module.
// i18n: migrated

mod backdrop;
mod caffeine;
mod chat;
mod clipboard;
mod daemon;
mod endpoint;
mod file_drop;
#[cfg(windows)]
mod launch_at_login;
mod listening_ports;
mod notification_mutes;
mod overview;
mod pane_move;
mod port_forward;
mod processes;
mod root;
#[cfg(test)]
pub(crate) mod tests;
mod wsl_drop;

pub(crate) use spaceport_i18n::Localize;

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
