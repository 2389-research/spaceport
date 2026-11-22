// ABOUTME: Checks the product named by a refused Core handshake.
// ABOUTME: Keeps the runtime name and typed generation mismatch intact.
use super::*;

#[test]
fn a_newer_core_generation_requests_a_spaceport_update() {
    let mut reply = hello_reply("0.1.13", &every_method())["result"].clone();
    reply["generation"] = 2.into();
    let hello: crate::hello::Hello = serde_json::from_value(reply).unwrap();
    let error = hello.check().unwrap_err();
    assert_eq!(error, crate::hello::Mismatch::Newer(2));
    assert_eq!(
        error.to_string(),
        "herdr-core speaks protocol generation 2; update Spaceport"
    );
}
