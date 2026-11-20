// ABOUTME: Shares localization assertions across typed error families.
// ABOUTME: Checks English diagnostics and complete pseudo-locale messages.
//! Shared checks for every error mapping, and a topic file per mapping.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::Localize;
use sheprd_i18n::{Locale, Pseudo};
use std::fmt::Display;

mod activation_guidance;
mod backdrop;
mod caffeine;
mod chat;
mod chat_protocol;
mod client;
mod clipboard;
mod config;
mod control;
mod daemon;
mod delivery;
mod devices;
mod endpoint;
mod file_drop;
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
mod shell;
mod sound;
mod terminal_attach;
mod usage;
mod version_guidance;
mod wsl_drop;

/// Fluent's isolation marks around each placeable, which `Display` lacks.
const ISOLATION: [char; 2] = ['\u{2068}', '\u{2069}'];

/// Each error's English message is its `Display` without isolation marks,
/// so the window says what the log does, and under `en-XA` it is a whole
/// translated message. One call per enum, with every variant.
pub(crate) fn assert_localized<E: Localize + Display>(errors: &[E]) {
    for error in errors {
        crate::i18n::show(&Locale::FALLBACK);
        let english = error.localize().replace(ISOLATION, "");
        assert_eq!(english, error.to_string(), "{error}");
        crate::i18n::show(&Pseudo::Accented.locale());
        let shown = error.localize();
        crate::i18n::show(&Locale::FALLBACK);
        assert!(sheprd_i18n::pseudo::is_marked(&shown), "{error}: {shown}");
    }
}

/// The wire's own names for what crosses the connection: a surface is a pane's
/// screen, a frame one message, a boot one run of the daemon, an endpoint the
/// connection to a host, a bridge the SSH connection, a codec how a screen is
/// encoded. A notice is read by a person, so the message says what happened
/// instead; the typed variant and the log keep the wire's word. See
/// `docs/notes-error-jargon.md`.
const WIRE_WORDS: [&str; 7] = [
    "boot",
    "bridge",
    "codec",
    "endpoint",
    "frame",
    "herdr server",
    "surface",
];

/// No message in `errors` says one of those words to the person reading it.
pub(crate) fn assert_no_wire_words<E: Localize + Display>(errors: &[E]) {
    crate::i18n::show(&Locale::FALLBACK);
    for error in errors {
        let shown = error.localize().to_lowercase();
        for word in WIRE_WORDS {
            assert!(!shown.contains(word), "{word} in {error}: {shown}");
        }
    }
}

/// Errors whose text is the system's or a parser's, shown as it is in
/// every language.
pub(crate) fn assert_verbatim<E: Localize + Display>(errors: &[E]) {
    for error in errors {
        crate::i18n::show(&Pseudo::Accented.locale());
        let shown = error.localize();
        crate::i18n::show(&Locale::FALLBACK);
        assert_eq!(shown, error.to_string());
    }
}
