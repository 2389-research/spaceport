//! Every variant of this crate's failures words itself for a window.

use crate::Error;
use spaceport_i18n::assertions::{assert_localized, assert_verbatim};

#[test]
fn every_catalog_variant_has_a_message() {
    assert_localized(&[Error::Client(spaceport_client::Error::Disconnected)]);
    assert_verbatim(&[Error::Worker(std::io::Error::other("no worker"))]);
}
