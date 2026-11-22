// ABOUTME: Checks product copy in Claude's unsupported-control response.
// ABOUTME: Preserves request correlation and the existing refusal envelope.
use super::*;

#[test]
fn unsupported_controls_name_spaceport_and_preserve_correlation() {
    let output = opened().on_harness(&json!({
        "type": "control_request", "request_id": "branding-39",
        "request": {"subtype": "hook_callback", "callback_id": "hook_0"}
    }));
    assert_eq!(
        claude(&output[0]),
        &json!({
            "type": "control_response",
            "response": {
                "subtype": "error", "request_id": "branding-39",
                "error": "Spaceport does not support this request."
            }
        })
    );
}
