use super::*;
use sheprd_usage::shell::{Error, START};
use std::io;

#[test]
fn every_variant_has_a_message() {
    let mut errors = vec![
        Error::Size,
        Error::Timeout,
        Error::Unreachable,
        Error::WslHostUnsupported,
    ];
    errors.extend([START, "read the moon"].map(|operation| Error::Process {
        operation,
        source: io::Error::other("no such file"),
    }));
    assert_localized(&errors);
    assert_localized(&[Error::Client(sheprd_client::Error::Disconnected)]);
}
