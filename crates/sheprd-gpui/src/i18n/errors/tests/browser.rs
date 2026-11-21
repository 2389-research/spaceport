// ABOUTME: Checks saved-state admission diagnostics through the localization boundary.
// ABOUTME: Keeps English errors consistent and pseudo-locale messages complete.
use super::*;
use sheprd_browser::Error;

#[test]
fn saved_state_admission_errors_are_complete_localized_messages() {
    assert_localized(&[Error::RestorationBlocked, Error::TabIdsExhausted]);
}
