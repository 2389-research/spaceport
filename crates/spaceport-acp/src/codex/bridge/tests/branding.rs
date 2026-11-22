// ABOUTME: Checks Spaceport's display identity in real Codex bridge messages.
// ABOUTME: Keeps client wire names, request IDs, and refusal codes unchanged.
use super::*;

#[test]
fn initialize_names_spaceport_without_renaming_the_wire_client() {
    let output = Bridge::default().start();
    let bytes = serde_json::to_vec(harness(&output[0])).unwrap();
    let message: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(message["method"], "initialize");
    assert_eq!(message["params"]["clientInfo"]["title"], "Spaceport");
    assert_eq!(message["params"]["clientInfo"]["name"], "herdr-acp");
    assert_eq!(
        message["params"]["clientInfo"]["version"],
        env!("CARGO_PKG_VERSION")
    );
}

#[test]
fn unsupported_requests_name_spaceport_and_preserve_correlation() {
    let output =
        opened().on_harness(&json!({"id": 39, "method": "unsupported/request", "params": {}}));
    assert_eq!(
        harness(&output[0]),
        &json!({
            "id": 39,
            "error": {"code": -32601, "message": "Spaceport does not support this request."}
        })
    );
}
