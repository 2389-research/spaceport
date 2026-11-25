//! Messages by ID: the ones the locale shown has, and English for text that
//! leaves the window.

use crate::__private::FILES;
use crate::{Catalog, FluentArgs, FluentValue, Locale};
use std::sync::LazyLock;

/// English without isolation marks, for text that leaves the window.
static ENGLISH: LazyLock<Option<Catalog>> = LazyLock::new(|| Catalog::english(FILES).ok());

/// Each language with messages, named in itself, as [`available`] orders them.
static ENDONYMS: LazyLock<Vec<(Locale, String)>> = LazyLock::new(|| Catalog::endonyms(FILES));

/// `id` in the locale shown. Only [`fl!`](crate::fl) names a message by
/// literal, so its ID is checked; [`message`] takes one chosen at run time.
pub fn text(id: &'static str, args: &[(&str, FluentValue<'_>)]) -> String {
    message(id, args)
}

/// A message whose ID is data, such as a catalog command's. Tests check each
/// such table against the English files (see `takes`, in test builds).
pub fn message(id: &str, args: &[(&str, FluentValue<'_>)]) -> String {
    match crate::current::catalog() {
        Some(catalog) => format(&catalog, id, args),
        None => id.to_owned(),
    }
}

/// A message in English, for command line output and searches that match the
/// English name of a translated label.
pub fn english(id: &str, args: &[(&str, FluentValue<'_>)]) -> String {
    match ENGLISH.as_ref() {
        Some(catalog) => format(catalog, id, args),
        None => id.to_owned(),
    }
}

fn format(catalog: &Catalog, id: &str, args: &[(&str, FluentValue<'_>)]) -> String {
    if args.is_empty() {
        return catalog.text(id);
    }
    let mut fluent = FluentArgs::with_capacity(args.len());
    for (name, value) in args {
        fluent.set(*name, value.clone());
    }
    catalog.text_with(id, &fluent)
}

/// The variables English message `id` takes, or `None` when there is no such
/// message, for tests of tables that name messages as data.
#[cfg(any(test, feature = "test-support"))]
pub fn takes(id: &str) -> Option<&'static [&'static str]> {
    crate::__private::MESSAGES
        .iter()
        .find(|(message, _)| *message == id)
        .map(|(_, variables)| *variables)
}

/// The languages with messages; pseudo-locales are not among them.
pub fn available() -> Vec<Locale> {
    Catalog::available(FILES)
}

/// What `locale` calls itself (`English`, `עברית`), or its tag when it has no
/// messages of its own, whatever language the windows show.
pub fn endonym(locale: &Locale) -> String {
    ENDONYMS
        .iter()
        .find(|(candidate, _)| candidate == locale)
        .map_or_else(|| locale.to_string(), |(_, name)| name.clone())
}
