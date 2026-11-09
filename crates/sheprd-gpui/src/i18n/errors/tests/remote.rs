// ABOUTME: Checks localized refusals for the retained host script routes.
// ABOUTME: Ensures the shared root error delegates without exposing migration wording.

use super::*;
use sheprd_remote::{Error, host_script};

#[test]
fn every_host_refusal_has_a_message() {
    assert_localized(&[
        host_script::Error::UnsupportedHost,
        host_script::Error::DevelopmentSession,
    ]);
    assert_localized(&[Error::HostScript(host_script::Error::DevelopmentSession)]);
}
