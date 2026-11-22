// ABOUTME: Pins Codex turn start, cancellation, and completion ordering.
// ABOUTME: Verifies prompt ownership through delayed provider messages.
use super::*;

fn cancel(bridge: &mut Bridge) -> Vec<Out> {
    bridge.on_session(Inbound::Notification {
        method: "session/cancel".to_owned(),
        params: json!({}),
    })
}

fn start(bridge: &mut Bridge, id: i64) -> Value {
    let outs = bridge.on_session(request(
        id,
        "session/prompt",
        json!({
            "sessionId": THREAD, "prompt": [{"type": "text", "text": "go"}],
        }),
    ));
    harness(&outs[0]).clone()
}

#[test]
fn a_natural_completion_after_cancel_keeps_codexs_result() {
    let mut bridge = opened();
    prompt(&mut bridge, 12);
    cancel(&mut bridge);
    let outs = notify(
        &mut bridge,
        "turn/completed",
        json!({
            "threadId": THREAD, "turn": {"id": "turn-1", "status": "completed"},
        }),
    );
    assert_eq!(
        response(&outs[0], 12).as_ref().unwrap()["stopReason"],
        "end_turn"
    );
    assert!(cancel(&mut bridge).is_empty());
}

#[test]
fn a_failed_start_after_cancel_releases_the_prompt() {
    let mut bridge = opened();
    let started = start(&mut bridge, 12);
    assert!(cancel(&mut bridge).is_empty());
    let outs = bridge.on_harness(&json!({"id": started["id"], "error": {"message": "No model"}}));
    assert_eq!(
        response(&outs[0], 12).as_ref().unwrap_err().message,
        "No model"
    );
    assert!(cancel(&mut bridge).is_empty());
    let next = start(&mut bridge, 13);
    assert!(
        bridge
            .on_harness(&answer(&next["id"], json!({"turn": {"id": "turn-2"}})))
            .is_empty()
    );
}

#[test]
fn repeated_cancel_and_started_messages_send_one_interrupt() {
    let mut bridge = opened();
    let started = start(&mut bridge, 12);
    assert!(cancel(&mut bridge).is_empty());
    let event = json!({"threadId": THREAD, "turn": {"id": "turn-1"}});
    let outs = notify(&mut bridge, "turn/started", event.clone());
    assert_eq!(harness(&outs[0])["method"], "turn/interrupt");
    assert!(cancel(&mut bridge).is_empty());
    assert!(
        bridge
            .on_harness(&answer(&started["id"], json!({"turn": {"id": "turn-1"}})))
            .is_empty()
    );
    assert!(notify(&mut bridge, "turn/started", event).is_empty());
}

#[test]
fn late_start_failure_and_old_completion_do_not_end_the_next_prompt() {
    let mut bridge = opened();
    let first = start(&mut bridge, 12);
    let outs = notify(
        &mut bridge,
        "turn/completed",
        json!({
            "threadId": THREAD, "turn": {"id": "turn-1", "status": "completed"},
        }),
    );
    assert!(response(&outs[0], 12).is_ok());
    // ACP may reuse an id after its prior prompt has completed.
    let second = start(&mut bridge, 12);
    assert!(
        bridge
            .on_harness(&json!({"id": first["id"], "error": {"message": "late"}}))
            .is_empty()
    );
    bridge.on_harness(&answer(&second["id"], json!({"turn": {"id": "turn-2"}})));
    assert!(
        notify(
            &mut bridge,
            "turn/completed",
            json!({
                "threadId": THREAD, "turn": {"id": "turn-1", "status": "completed"},
            })
        )
        .is_empty()
    );
    let ended = notify(
        &mut bridge,
        "turn/completed",
        json!({
            "threadId": THREAD, "turn": {"id": "turn-2", "status": "completed"},
        }),
    );
    assert!(response(&ended[0], 12).is_ok());
}
