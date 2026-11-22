use super::*;

#[test]
fn fan_out_request_is_rejected() {
    let request = r#"{"method":"work","caller":{"pane_id":"w_1:p1","workspace_id":"w_1","daemon_socket":null},"op":{"op":"fan_out","prompt":"fix it","agents":[["claude",2]]}}"#;
    assert!(serde_json::from_str::<Request>(request).is_err());
}
