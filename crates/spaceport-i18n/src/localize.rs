//! An error's message in the locale shown (docs/i18n.md): the trait each crate
//! implements beside its own `Error`, and the checks its tests run.
//!
//! A crate words itself: an error enum keeps its `thiserror` English `Display`
//! for logs, and [`Localize`] picks a message by variant where a window shows
//! it. An enum wrapping another crate's error delegates to that error's own
//! implementation, so each failure has one message as it has one definition.

use std::sync::Arc;

/// An error's message in the current locale, for a window, a notification,
/// or an `$error` placeable. Logs keep `Display`.
pub trait Localize {
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

#[cfg(any(test, feature = "test-support"))]
pub mod assertions {
    //! What a crate's tests check every error mapping for: the English message
    //! is the `Display` without isolation marks, and under `en-XA` it is a
    //! whole translated message.

    use super::Localize;
    use crate::{Locale, Pseudo, set_locale};
    use std::fmt::Display;

    /// Fluent's isolation marks around each placeable, which `Display` lacks.
    pub const ISOLATION: [char; 2] = ['\u{2068}', '\u{2069}'];

    /// Each error's English message is its `Display` without isolation marks,
    /// so the window says what the log does, and under `en-XA` it is a whole
    /// translated message. One call per enum, with every variant.
    pub fn assert_localized<E: Localize + Display>(errors: &[E]) {
        for error in errors {
            set_locale(&Locale::FALLBACK);
            let english = error.localize().replace(ISOLATION, "");
            assert_eq!(english, error.to_string(), "{error}");
            set_locale(&Pseudo::Accented.locale());
            let shown = error.localize();
            set_locale(&Locale::FALLBACK);
            assert!(crate::pseudo::is_marked(&shown), "{error}: {shown}");
        }
    }

    /// The wire's own names for what crosses the connection: a surface is a
    /// pane's screen, a frame one message, a boot one run of the daemon, an
    /// endpoint the connection to a host, a bridge the SSH connection, a codec
    /// how a screen is encoded. A notice is read by a person, so the message
    /// says what happened instead; the typed variant and the log keep the
    /// wire's word. See `docs/notes-error-jargon.md`.
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
    pub fn assert_no_wire_words<E: Localize + Display>(errors: &[E]) {
        set_locale(&Locale::FALLBACK);
        for error in errors {
            let shown = error.localize().to_lowercase();
            for word in WIRE_WORDS {
                assert!(!shown.contains(word), "{word} in {error}: {shown}");
            }
        }
    }

    /// Errors whose text is the system's or a parser's, shown as it is in
    /// every language.
    pub fn assert_verbatim<E: Localize + Display>(errors: &[E]) {
        for error in errors {
            set_locale(&Pseudo::Accented.locale());
            let shown = error.localize();
            set_locale(&Locale::FALLBACK);
            assert_eq!(shown, error.to_string());
        }
    }
}
