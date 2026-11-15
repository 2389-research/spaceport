use super::*;
use sheprd_usage::{Error, shell};

#[test]
fn every_variant_has_a_message() {
    assert_localized(&[Error::Shell(shell::Error::Timeout)]);
}
