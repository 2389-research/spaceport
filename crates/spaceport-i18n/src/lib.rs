//! The languages Spaceport can show (docs/i18n.md): locales and their text
//! direction, the `language` setting and its negotiation against the system's
//! languages with an `en-US` fallback, the Fluent catalog of the embedded
//! messages, the `en-XA`/`en-XB` pseudo-locales, which regional Han glyphs a
//! Chinese, Japanese, or Korean language wants, and the lookup every crate
//! names its words through: [`fl!`], [`text`], [`message`], [`english`].
//!
//! The published, translated words are here (`locales/<tag>/<feature>.ftl`),
//! and a crate that shows text calls [`fl!`] itself, naming its own keys. No
//! GPUI: which locale a window shows is the app's, and this crate only
//! reports and replaces the catalog shown. A crate that words text adds this
//! crate's `test-support` feature to its dev-dependencies, so a test that
//! switches locale switches only its own thread's catalog.
// i18n: migrated

mod catalog;
mod check;
mod cjk;
mod current;
mod error;
mod locale;
mod lookup;
mod negotiate;
pub mod pseudo;

pub use catalog::{Catalog, ENDONYM, File};
pub use cjk::CjkScript;
pub use error::{Error, Result};
pub use fluent::{FluentArgs, FluentValue};
pub use locale::{Direction, Language, Locale, Pseudo};
#[cfg(any(test, feature = "test-support"))]
pub use lookup::takes;
pub use lookup::{available, endonym, english, message, text};
pub use negotiate::{negotiate, system_languages};
pub use unic_langid::LanguageIdentifier;

/// What [`fl!`] reaches in the crate that calls it, which `$crate` names.
/// Nothing here is for calling by hand.
#[doc(hidden)]
pub mod __private {
    pub use crate::check::check;

    include!(concat!(env!("OUT_DIR"), "/messages.rs"));
}

/// A message in the locale shown. `fl!("id")`, or with named arguments
/// `fl!("id", name = value)`; a value is anything Fluent takes, such as a
/// string or a number.
///
/// A call naming no English message, an argument the message does not take,
/// or one it leaves out stops the build at that call.
#[macro_export]
macro_rules! fl {
    ($id:literal $(,)?) => {{
        const _: () = $crate::__private::check($id, &[]);
        $crate::text($id, &[])
    }};
    ($id:literal, $($name:ident = $value:expr),+ $(,)?) => {{
        const _: () = $crate::__private::check($id, &[$(stringify!($name)),+]);
        $crate::text(
            $id,
            &[$((stringify!($name), $crate::FluentValue::from($value))),+],
        )
    }};
}

/// The locale the messages shown are in, for a cache of text written in it to
/// notice a switch. Read where there is no app context.
pub fn shown_locale() -> Option<Locale> {
    current::catalog().map(|catalog| catalog.locale().clone())
}

/// Shows `locale`'s messages from now on, keeping the ones shown so far when
/// they cannot load. Returns whether they loaded, so a caller that mirrors
/// the chrome by the locale's direction knows whether to publish it.
pub fn set_locale(locale: &Locale) -> bool {
    current::replace(locale)
}

#[cfg(test)]
mod tests;
