// ABOUTME: Keeps restricted Pi requests active until authoritative run settlement.
// ABOUTME: Covers retries, cancellation during backoff, stale events, and bounded final diagnostics.
use super::*;

fn restricted_prompt() -> Bridge {
    let mut bridge = opened();
    bridge.restrict();
    send(&mut bridge, 2);
    bridge
}

fn send(bridge: &mut Bridge, id: i64) {
    let outs = bridge.on_session(request(
        id,
        "session/prompt",
        json!({"sessionId":SESSION,"prompt":[{"type":"text","text":"question"}]}),
    ));
    let command = harness(&outs[0]).clone();
    bridge.on_harness(&done(&command, json!({"disposition":"started"})));
    bridge.on_harness(&json!({"type":"agent_start"}));
}

fn ended(bridge: &mut Bridge, reason: &str) -> Vec<Out> {
    bridge.on_harness(&json!({"type":"agent_end","willRetry":reason=="error","messages":[{"role":"assistant","stopReason":reason,"errorMessage":"opaque provider failure"}]}))
}

#[test]
fn restricted_retry_stays_active_until_the_recovered_run_settles() {
    let mut bridge = restricted_prompt();
    assert!(
        ended(&mut bridge, "error").is_empty(),
        "an attempt ending is not run settlement"
    );
    bridge.on_harness(&json!({"type":"auto_retry_start","errorMessage":"retry diagnostic"}));
    let rejected = bridge.on_session(request(
        3,
        "session/prompt",
        json!({"sessionId":SESSION,"prompt":[{"type":"text","text":"second question"}]}),
    ));
    assert!(response(&rejected[0], 3).is_err());
    assert!(ended(&mut bridge, "stop").is_empty());
    let settled = bridge.on_harness(&json!({"type":"agent_settled","aborted":false}));
    assert_eq!(
        response(&settled[0], 2).as_ref().unwrap()["stopReason"],
        "end_turn"
    );
    assert!(
        bridge
            .on_harness(&json!({"type":"agent_settled","aborted":false}))
            .is_empty()
    );
}

#[test]
fn restricted_backoff_cancellation_uses_the_settled_aborted_flag() {
    let mut bridge = restricted_prompt();
    assert!(ended(&mut bridge, "error").is_empty());
    bridge.on_harness(&json!({"type":"auto_retry_start","errorMessage":"opaque retry failure"}));
    let cancel = bridge.on_session(Inbound::Notification {
        method: "session/cancel".into(),
        params: json!({}),
    });
    assert_eq!(harness(&cancel[0])["type"], "clear_queue");
    assert_eq!(harness(&cancel[1])["type"], "abort");
    let settled = bridge.on_harness(&json!({"type":"agent_settled","aborted":true}));
    assert_eq!(
        response(&settled[0], 2).as_ref().unwrap()["stopReason"],
        "cancelled"
    );
}

#[test]
fn restricted_final_error_waits_for_settlement_and_keeps_only_a_bounded_diagnostic() {
    let mut bridge = restricted_prompt();
    let attempt=bridge.on_harness(&json!({"type":"agent_end","willRetry":false,"messages":[{"role":"assistant","stopReason":"error","errorMessage":"λ".repeat(10000),"content":[{"type":"text","text":"x".repeat(100000)}]}]}));
    assert!(attempt.is_empty());
    let settled = bridge.on_harness(&json!({"type":"agent_settled","aborted":false}));
    let error = response(&settled[0], 2).as_ref().unwrap_err();
    assert!(!error.message.is_empty());
    assert!(
        error.message.len() <= 4096,
        "bounded result must not retain provider content or an unbounded error"
    );
}

#[test]
fn restricted_settlement_cannot_reuse_the_previous_turn_or_an_untyped_abort_flag() {
    let mut bridge = restricted_prompt();
    assert!(
        bridge
            .on_harness(&json!({"type":"agent_settled","aborted":false}))
            .is_empty()
    );
    ended(&mut bridge, "stop");
    assert!(
        bridge
            .on_harness(&json!({"type":"agent_settled","aborted":"false"}))
            .is_empty()
    );
    let settled = bridge.on_harness(&json!({"type":"agent_settled","aborted":false}));
    assert!(response(&settled[0], 2).is_ok());
    send(&mut bridge, 3);
    assert!(
        bridge
            .on_harness(&json!({"type":"agent_settled","aborted":false}))
            .is_empty()
    );
    ended(&mut bridge, "length");
    let settled = bridge.on_harness(&json!({"type":"agent_settled","aborted":false}));
    assert_eq!(
        response(&settled[0], 3).as_ref().unwrap()["stopReason"],
        "max_tokens"
    );
}

#[test]
fn restricted_settlement_preserves_an_aborted_attempt_without_a_run_abort() {
    let mut bridge = restricted_prompt();
    ended(&mut bridge, "aborted");
    let settled = bridge.on_harness(&json!({"type":"agent_settled","aborted":false}));
    assert_eq!(
        response(&settled[0], 2).as_ref().unwrap()["stopReason"],
        "cancelled"
    );
}
