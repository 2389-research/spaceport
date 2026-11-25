//! The catalog the locale shown has its messages from.
//!
//! Tests keep one per thread: each test runs on its own, and one switching to
//! a pseudo-locale must not change the text another reads. That is this
//! crate's `test-support` feature as well as `cfg(test)`: a dependency's own
//! `cfg(test)` is false inside its consumers' test builds, so a crate whose
//! tests switch the locale enables the feature in its dev-dependencies.

use crate::__private::FILES;
use crate::{Catalog, Locale};
use std::sync::{Arc, LazyLock};

/// English, until the app picks the setting's locale.
static DEFAULT: LazyLock<Option<Arc<Catalog>>> =
    LazyLock::new(|| load(&Locale::FALLBACK).map(Arc::new));

fn load(locale: &Locale) -> Option<Catalog> {
    Catalog::new(FILES, locale.clone())
        .inspect_err(|error| tracing::warn!(%error, %locale, "Could not load messages"))
        .ok()
}

#[cfg(not(any(test, feature = "test-support")))]
static CURRENT: std::sync::RwLock<Option<Arc<Catalog>>> = std::sync::RwLock::new(None);

#[cfg(not(any(test, feature = "test-support")))]
fn stored() -> Option<Arc<Catalog>> {
    CURRENT.read().ok().and_then(|current| current.clone())
}

#[cfg(not(any(test, feature = "test-support")))]
fn store(catalog: Option<Arc<Catalog>>) {
    if let Ok(mut current) = CURRENT.write() {
        *current = catalog;
    }
}

#[cfg(any(test, feature = "test-support"))]
thread_local! {
    static CURRENT: std::cell::RefCell<Option<Arc<Catalog>>> = const { std::cell::RefCell::new(None) };
}

#[cfg(any(test, feature = "test-support"))]
fn stored() -> Option<Arc<Catalog>> {
    CURRENT.with_borrow(Clone::clone)
}

#[cfg(any(test, feature = "test-support"))]
fn store(catalog: Option<Arc<Catalog>>) {
    CURRENT.set(catalog);
}

/// The catalog shown now, or English when no other has loaded.
pub(crate) fn catalog() -> Option<Arc<Catalog>> {
    stored().or_else(|| DEFAULT.clone())
}

/// Shows `locale`'s messages from now on. Returns whether they loaded: a
/// catalog that cannot is not worth replacing the one shown with.
pub(crate) fn replace(locale: &Locale) -> bool {
    let Some(catalog) = load(locale) else {
        return false;
    };
    store(Some(Arc::new(catalog)));
    true
}
