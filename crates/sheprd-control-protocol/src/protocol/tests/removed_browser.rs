use super::*;

#[test]
fn browser_reload_wire_shapes_are_rejected() {
    assert!(
        serde_json::from_str::<Request>(r#"{"method":"browser.reload","pane_id":"p1"}"#).is_err()
    );
    assert!(serde_json::from_str::<Response>(r#"{"status":"reloaded","tabs":1}"#).is_err());
}
