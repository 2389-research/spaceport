use super::*;
use spaceport_usage::{Error, shell};

#[test]
fn every_variant_has_a_message() {
    assert_localized(&[Error::Shell(shell::Error::Timeout)]);
}
