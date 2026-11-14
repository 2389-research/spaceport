//! The exact prompt reply the bridge sends the session: the contract its
//! shared builders must keep.
use super::*;

fn completed(bridge: &mut Bridge, status: &str) -> Vec<Out> {
    notify(
        bridge,
        "turn/completed",
        json!({"threadId": THREAD, "turn": {"id": "turn-1", "status": status}}),
    )
}

#[test]
fn a_finished_turn_sends_exactly_this_reply() {
    let mut bridge = opened();
    prompt(&mut bridge, 2);
    notify(
        &mut bridge,
        "thread/tokenUsage/updated",
        json!({"threadId": THREAD, "tokenUsage": {
            "last": {"totalTokens": 1200, "inputTokens": 1000, "cachedInputTokens": 400, "outputTokens": 200, "reasoningOutputTokens": 50},
            "total": {}, "modelContextWindow": 400_000,
        }}),
    );
    let end = completed(&mut bridge, "completed");
    let answer = response(&end[0], 2).as_ref().unwrap();
    assert_eq!(
        answer,
        &json!({"stopReason": "end_turn", "usage": {
            "inputTokens": 600,
            "cachedReadTokens": 400,
            "outputTokens": 200,
            "thoughtTokens": 50,
            "totalTokens": 1200,
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
        (600, 400, 200, 50, 1200)
    );
}

#[test]
fn a_turn_without_usage_sends_no_usage() {
    let mut bridge = opened();
    prompt(&mut bridge, 2);
    let end = completed(&mut bridge, "interrupted");
    let answer = response(&end[0], 2).as_ref().unwrap();
    assert_eq!(answer, &json!({"stopReason": "cancelled"}));
    assert!(crate::project::tokens(answer).is_none());
}

#[test]
fn a_failed_turn_and_a_second_prompt_are_refused_with_these_errors() {
    let mut bridge = opened();
    prompt(&mut bridge, 2);
    let busy = bridge.on_session(request(
        3,
        "session/prompt",
        json!({"sessionId": THREAD, "prompt": [{"type": "text", "text": "again"}]}),
    ));
    let error = response(&busy[0], 3).as_ref().unwrap_err();
    assert_eq!(
        serde_json::to_value(error).unwrap(),
        json!({"code": -32603, "message": "A turn is already running."})
    );
    let end = completed(&mut bridge, "failed");
    let error = response(&end[0], 2).as_ref().unwrap_err();
    assert_eq!(
        serde_json::to_value(error).unwrap(),
        json!({"code": -32603, "message": "Codex's turn failed."})
    );
}
