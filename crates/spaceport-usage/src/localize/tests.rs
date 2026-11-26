//! Every variant of this crate's failures words itself for a window.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::{
    Error,
    shell::{self, START},
};
use spaceport_i18n::assertions::assert_localized;
use std::io;

#[test]
fn every_variant_has_a_message() {
    assert_localized(&[Error::Shell(shell::Error::Timeout)]);
}

#[test]
fn every_host_shell_variant_has_a_message() {
    let mut errors = vec![
        shell::Error::Size,
        shell::Error::Timeout,
        shell::Error::Unreachable,
        shell::Error::WslHostUnsupported,
    ];
    errors.extend(
        [START, "read the moon"].map(|operation| shell::Error::Process {
            operation,
            source: io::Error::other("no such file"),
        }),
    );
    assert_localized(&errors);
    assert_localized(&[shell::Error::Client(spaceport_client::Error::Disconnected)]);
}
