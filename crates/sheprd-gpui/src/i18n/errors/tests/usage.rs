use super::*;
use sheprd_usage::Error;
use std::io;

#[test]
fn every_variant_has_a_message() {
    let mut errors = vec![
        Error::Connect,
        Error::NoPlan,
        Error::RateLimited,
        Error::Size,
        Error::Status(429),
        Error::Timeout,
        Error::Unavailable,
        Error::Unreachable,
        Error::WslHostUnsupported,
    ];
    errors.extend(
        ["start the remote usage shell", "read the moon"].map(|operation| Error::Process {
            operation,
            source: io::Error::other("no such file"),
        }),
    );
    assert_localized(&errors);
    assert_localized(&[Error::Client(sheprd_client::Error::Disconnected)]);
}
