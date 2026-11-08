use super::super::*;

#[test]
fn phone_control_request_is_rejected() {
    let request = r#"{"method":"remote","caller":{},"op":"status"}"#;
    assert!(serde_json::from_str::<Request>(request).is_err());
}
