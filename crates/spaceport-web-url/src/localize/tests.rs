//! The address failure words itself for a window.

use crate::Error;
use spaceport_i18n::assertions::assert_localized;

#[test]
fn the_invalid_address_message_is_complete() {
    assert_localized(&[Error::InvalidUrl]);
}
