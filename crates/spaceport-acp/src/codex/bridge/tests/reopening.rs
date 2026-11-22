// ABOUTME: Preserves the current Codex thread while another opening is unresolved.
// ABOUTME: Covers prompts and active-turn routing after pending or refused reopenings.
use super::*;

fn reopen(bridge: &mut Bridge) -> Value {
    let outs = bridge.on_session(request(
        10,
        "session/load",
        json!({"sessionId": "unavailable", "cwd": "/work"}),
    ));
    harness(&outs[0]).clone()
}

fn refuse(bridge: &mut Bridge, opening: &Value) {
    let outs = bridge.on_harness(&json!({
        "id": opening["id"],
        "error": {"code": -1, "message": "No such thread"},
    }));
    assert_eq!(
        response(&outs[0], 10).as_ref().unwrap_err().message,
        "No such thread"
    );
}

#[test]
fn a_pending_reopening_keeps_prompting_the_current_thread() {
    let mut bridge = opened();
    reopen(&mut bridge);
    let call = prompt(&mut bridge, 2);
    assert_eq!(call["params"]["threadId"], THREAD);
}

#[test]
fn a_failed_reopening_keeps_prompting_the_current_thread() {
    for invalid in [false, true] {
        let mut bridge = opened();
        let opening = reopen(&mut bridge);
        if invalid {
            let outs = bridge.on_harness(&answer(&opening["id"], json!({"thread": {"id": ""}})));
            assert!(response(&outs[0], 10).is_err());
        } else {
            refuse(&mut bridge, &opening);
        }
        let call = prompt(&mut bridge, 2);
        assert_eq!(call["params"]["threadId"], THREAD);
    }
}

#[test]
fn reopening_keeps_cancellation_routed_to_the_existing_turn() {
    for failed in [false, true] {
        let mut bridge = opened();
        prompt(&mut bridge, 2);
        let opening = reopen(&mut bridge);
        if failed {
            refuse(&mut bridge, &opening);
        }
        let outs = bridge.on_session(Inbound::Notification {
            method: "session/cancel".to_owned(),
            params: json!({}),
        });
        assert_eq!(outs.len(), 1, "the existing turn must still be interrupted");
        assert_eq!(harness(&outs[0])["method"], "turn/interrupt");
        assert_eq!(
            harness(&outs[0])["params"],
            json!({"threadId": THREAD, "turnId": "turn-1"})
        );
    }
}

#[test]
fn reopening_keeps_existing_turn_updates_and_completion_routed() {
    for failed in [false, true] {
        let mut bridge = opened();
        prompt(&mut bridge, 2);
        let opening = reopen(&mut bridge);
        if failed {
            refuse(&mut bridge, &opening);
        }
        let chunks = notify(
            &mut bridge,
            "item/agentMessage/delta",
            json!({"threadId": THREAD, "itemId": "reply", "delta": "Still here"}),
        );
        assert_eq!(
            chunks.len(),
            1,
            "the current thread cannot become a foreign thread"
        );
        assert_eq!(update(&chunks[0])["content"]["text"], "Still here");
        let outs = notify(
            &mut bridge,
            "turn/completed",
            json!({"threadId": THREAD, "turn": {"id": "turn-1", "status": "completed"}}),
        );
        assert_eq!(outs.len(), 1);
        assert_eq!(
            response(&outs[0], 2).as_ref().unwrap()["stopReason"],
            "end_turn"
        );
    }
}
