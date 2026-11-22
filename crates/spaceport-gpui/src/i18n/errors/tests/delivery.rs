// ABOUTME: Checks local delivery refusals independently from wire rejections.
// ABOUTME: Preserves uncertainty and conversation-change text through localization.
use super::*;
use crate::chat::Refusal;

#[test]
fn every_delivery_refusal_has_an_honest_localized_message() {
    assert_localized(&[
        Refusal::DeliveryUnconfirmed,
        Refusal::Remote(spaceport_chat_protocol::Rejection::Unsupported),
        Refusal::SessionChanged,
    ]);
}
