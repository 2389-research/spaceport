#![allow(clippy::unwrap_used, clippy::expect_used)]
// ABOUTME: Checks saved-state admission diagnostics through the localization boundary.
// ABOUTME: Keeps English errors consistent and pseudo-locale messages complete.
use super::*;
use spaceport_i18n::assertions::{assert_localized, assert_no_wire_words};

#[test]
fn saved_state_admission_errors_are_complete_localized_messages() {
    assert_localized(&[
        Error::InvalidBrowserTabs,
        Error::InvalidGroupLayouts,
        Error::RestorationBlocked,
        Error::StateFile(spaceport_state_file::Error::MissingRoot("/tmp".into())),
        Error::TabIdsExhausted,
    ]);
    assert_no_wire_words(&[Error::RestorationBlocked, Error::TabIdsExhausted]);
}
