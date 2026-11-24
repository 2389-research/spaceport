use super::*;
use crate::endpoint::Error;

#[test]
fn every_endpoint_variant_has_a_message() {
    assert_localized(&[
        Error::NoHost {
            id: "ssh:devbox".into(),
        },
        Error::SurfaceSwitchingUnsupported,
    ]);
}

#[test]
fn every_catalog_variant_has_a_message() {
    assert_localized(&[spaceport_endpoint::Error::Client(
        spaceport_client::Error::Disconnected,
    )]);
    assert_verbatim(&[spaceport_endpoint::Error::Worker(
        std::io::Error::other("no worker").into(),
    )]);
}
