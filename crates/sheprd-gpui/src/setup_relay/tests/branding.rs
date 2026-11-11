// ABOUTME: Checks product identity in actual setup relay error formatting.
// ABOUTME: Covers English and pseudo-locales without starting a device setup.
use super::*;
use crate::i18n::Localize;
use sheprd_i18n::{Locale, Pseudo};

#[test]
fn closed_setup_names_spaceport() {
    let error = Error::SetupRelay(crate::setup_relay::Error::Closed);
    assert_eq!(
        error.to_string(),
        "Spaceport closed the setup before it finished. Start it again from Add Device."
    );
}

#[test]
fn localized_closed_setup_preserves_spaceport() {
    for locale in [
        Locale::FALLBACK,
        Pseudo::Accented.locale(),
        Pseudo::Bidi.locale(),
    ] {
        crate::i18n::show(&locale);
        let shown = Error::SetupRelay(crate::setup_relay::Error::Closed).localize();
        crate::i18n::show(&Locale::FALLBACK);
        assert!(shown.contains("Spaceport"), "{locale:?}: {shown}");
        assert!(!shown.contains("Sheprd"), "{locale:?}: {shown}");
    }
}
