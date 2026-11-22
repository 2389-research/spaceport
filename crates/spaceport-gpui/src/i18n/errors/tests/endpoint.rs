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
