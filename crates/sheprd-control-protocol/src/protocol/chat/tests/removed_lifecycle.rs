// ABOUTME: Rejects retired chat lifecycle requests and replies at decoding.
// ABOUTME: A stale caller cannot trigger a desktop-owned agent conversion.
use super::*;

#[test]
fn removed_chat_lifecycle_wire_requests_are_rejected() {
    for op in [r#"{"op":"convert"}"#, r#"{"op":"convert","pane":"w1:p1"}"#] {
        assert!(serde_json::from_str::<ChatOp>(op).is_err(), "{op}");
        let request = format!(r#"{{"method":"chat","caller":{{}},"op":{op}}}"#);
        assert!(serde_json::from_str::<Request>(&request).is_err());
    }
}

#[test]
fn removed_chat_lifecycle_wire_replies_are_rejected() {
    let reply = r#"{"kind":"converting","agent":"claude","pane_id":"w1:p1"}"#;
    assert!(serde_json::from_str::<Reply>(reply).is_err());
    let response = format!(r#"{{"status":"chat","reply":{reply}}}"#);
    assert!(serde_json::from_str::<Response>(&response).is_err());
}
