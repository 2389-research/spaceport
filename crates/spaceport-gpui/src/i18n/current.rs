//! Loading a catalog, and the one thing the chrome does with a locale besides
//! the words: `spaceport-widgets` mirrors every element by the direction it
//! runs in, and this is where the direction of the locale shown is published
//! to it. The catalog itself is `spaceport-i18n`'s.

use spaceport_i18n::Locale;

/// Shows `locale`'s messages, and lays the chrome out in its direction. A
/// catalog that cannot load keeps the one shown so far, and its direction.
pub(super) fn replace(locale: &Locale) {
    if spaceport_i18n::set_locale(locale) {
        spaceport_widgets::direction::set(locale.direction());
    }
}
