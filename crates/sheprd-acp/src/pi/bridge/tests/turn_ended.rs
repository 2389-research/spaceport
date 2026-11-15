//! The exact prompt reply the bridge sends the session: the contract its
//! shared builders must keep.
use super::*;

fn ended(bridge: &mut Bridge, last: Value) -> Vec<Out> {
    bridge.on_harness(&json!({"type": "agent_end", "messages": [
        {"role": "user", "content": "Look"},
        {"role": "assistant", "stopReason": "toolUse", "usage": {"input": 100, "output": 20, "cacheRead": 50, "cacheWrite": 10}},
        last,
    ]}))
}

#[test]
fn a_finished_turn_sends_exactly_this_reply() {
    let mut bridge = opened();
    prompt(&mut bridge, 2);
    let outs = ended(
        &mut bridge,
        json!({"role": "assistant", "stopReason": "stop", "usage": {"input": 30, "output": 5, "cacheRead": 0, "cacheWrite": 0}}),
    );
    let answer = response(&outs[0], 2).as_ref().unwrap();
    // pi reports no reasoning tokens, so the reply names none.
    assert_eq!(
        answer,
        &json!({"stopReason": "end_turn", "usage": {
            "inputTokens": 140,
            "cachedReadTokens": 50,
            "outputTokens": 25,
            "totalTokens": 215,
        }})
    );
    let tokens = crate::project::tokens(answer).unwrap();
    assert_eq!(
        (
            tokens.input,
            tokens.cached,
            tokens.output,
            tokens.thought,
            tokens.total
        ),
        (140, 50, 25, 0, 215)
    );
}

#[test]
fn a_turn_without_usage_still_sends_zero_counts() {
    let mut bridge = opened();
    prompt(&mut bridge, 2);
    let outs = bridge.on_harness(&json!({"type": "agent_end", "messages": []}));
    let answer = response(&outs[0], 2).as_ref().unwrap();
    assert_eq!(
        answer,
        &json!({"stopReason": "end_turn", "usage": {
            "inputTokens": 0,
            "cachedReadTokens": 0,
            "outputTokens": 0,
            "totalTokens": 0,
        }})
    );
}

#[test]
fn a_failed_turn_and_a_second_prompt_are_refused_with_these_errors() {
    let mut bridge = opened();
    prompt(&mut bridge, 2);
    let busy = bridge.on_session(request(
        3,
        "session/prompt",
        json!({"sessionId": SESSION, "prompt": [{"type": "text", "text": "again"}]}),
    ));
    let error = response(&busy[0], 3).as_ref().unwrap_err();
    assert_eq!(
        serde_json::to_value(error).unwrap(),
        json!({"code": -32603, "message": "A turn is already running."})
    );
    let outs = ended(
        &mut bridge,
        json!({"role": "assistant", "stopReason": "error", "errorMessage": "No credit"}),
    );
    let error = response(&outs[0], 2).as_ref().unwrap_err();
    assert_eq!(
        serde_json::to_value(error).unwrap(),
        json!({"code": -32603, "message": "No credit"})
    );
}
