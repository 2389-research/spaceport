// ABOUTME: Pins Pi turn cancellation and prompt response ordering.
// ABOUTME: Verifies provider stop reasons remain authoritative.
use super::*;

fn cancel(bridge: &mut Bridge) -> Vec<Out> {
    bridge.on_session(Inbound::Notification {
        method: "session/cancel".to_owned(),
        params: json!({}),
    })
}

#[test]
fn a_natural_completion_after_cancel_keeps_pis_result() {
    let mut bridge = opened();
    prompt(&mut bridge, 12);
    cancel(&mut bridge);
    let outs = bridge.on_harness(&json!({"type": "agent_end", "messages": [
        {"role": "assistant", "stopReason": "stop"},
    ]}));
    assert_eq!(
        response(&outs[0], 12).as_ref().unwrap()["stopReason"],
        "end_turn"
    );
    assert!(cancel(&mut bridge).is_empty());
    assert!(
        bridge
            .on_harness(&json!({"type": "agent_end", "messages": []}))
            .is_empty()
    );
}

#[test]
fn a_prompt_can_complete_before_its_acceptance() {
    let mut bridge = opened();
    let outs = bridge.on_session(request(
        12,
        "session/prompt",
        json!({
            "sessionId": SESSION, "prompt": [{"type": "text", "text": "go"}],
        }),
    ));
    let command = harness(&outs[0]).clone();
    let ended = bridge.on_harness(&json!({"type": "agent_end", "messages": []}));
    assert!(response(&ended[0], 12).is_ok());
    assert!(bridge.on_harness(&done(&command, Value::Null)).is_empty());
    prompt(&mut bridge, 13);
}

#[test]
fn repeated_cancel_sends_one_abort() {
    let mut bridge = opened();
    prompt(&mut bridge, 12);
    assert_eq!(harness(&cancel(&mut bridge)[0])["type"], "abort");
    assert!(cancel(&mut bridge).is_empty());
}

#[test]
fn a_late_prompt_refusal_does_not_end_the_next_prompt() {
    let mut bridge = opened();
    let outs = bridge.on_session(request(
        12,
        "session/prompt",
        json!({
            "sessionId": SESSION, "prompt": [{"type": "text", "text": "go"}],
        }),
    ));
    let first = harness(&outs[0]).clone();
    bridge.on_harness(&json!({"type": "agent_end", "messages": []}));
    // ACP may reuse an id after its prior prompt has completed.
    prompt(&mut bridge, 12);
    assert!(
        bridge
            .on_harness(&json!({
                "type": "response", "id": first["id"], "success": false, "error": "late",
            }))
            .is_empty()
    );
    let ended = bridge.on_harness(&json!({"type": "agent_end", "messages": []}));
    assert!(response(&ended[0], 12).is_ok());
}
