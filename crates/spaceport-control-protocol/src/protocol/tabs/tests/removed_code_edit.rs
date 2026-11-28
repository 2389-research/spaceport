use super::*;

#[test]
fn code_edit_request_is_rejected_on_the_wire() {
    assert!(serde_json::from_str::<TabOp>(r#"{"op":"edit","path":"/a.rs"}"#).is_err());
    assert!(
        serde_json::from_str::<Request>(
            r#"{"method":"tab","caller":{"workspace_id":"w_1"},"op":{"op":"edit","path":"/a.rs"}}"#
        )
        .is_err()
    );
    round_trip(TabOp::Devices);
}
