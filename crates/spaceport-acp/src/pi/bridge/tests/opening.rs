// ABOUTME: Exercises Pi opening failures through real translator messages.
// ABOUTME: Rejects failed handshake data and missing session identifiers.
use super::*;

fn refused(command: &Value) -> Value {
    json!({"type": "response", "id": command["id"], "success": false, "error": "offline"})
}

#[test]
fn failed_state_or_models_answers_pending_and_later_openings() {
    for failed in [0, 1] {
        for early in [true, false] {
            let mut bridge = Bridge::default();
            let start = bridge.start();
            if !early {
                bridge.on_session(request(1, "session/new", json!({})));
            }
            let outs = bridge.on_harness(&refused(harness(&start[failed])));
            if early {
                assert!(outs.is_empty());
            } else {
                assert_eq!(outs.len(), 1, "failed handshake must answer the opening");
                assert_eq!(
                    response(&outs[0], 1).as_ref().unwrap_err().message,
                    "offline"
                );
            }
            let other = 1 - failed;
            assert!(
                bridge
                    .on_harness(&done(
                        harness(&start[other]),
                        json!({"sessionId": SESSION, "models": []})
                    ))
                    .is_empty()
            );
            let outs = bridge.on_session(request(2, "session/new", json!({})));
            assert_eq!(outs.len(), 1);
            assert_eq!(
                response(&outs[0], 2).as_ref().unwrap_err().message,
                "offline"
            );
        }
    }
}

#[test]
fn missing_or_empty_session_ids_fail_opening() {
    for state in [json!({}), json!({"sessionId": ""})] {
        let mut bridge = Bridge::default();
        let start = bridge.start();
        bridge.on_session(request(1, "session/new", json!({})));
        bridge.on_harness(&done(harness(&start[0]), state));
        let outs = bridge.on_harness(&done(harness(&start[1]), json!({"models": []})));
        assert!(
            response(&outs[0], 1).is_err(),
            "a session needs a nonempty id"
        );
    }
}

#[test]
fn opening_works_when_handshake_finishes_before_the_request() {
    let mut bridge = Bridge::default();
    let start = bridge.start();
    bridge.on_harness(&done(harness(&start[1]), json!({"models": []})));
    bridge.on_harness(&done(harness(&start[0]), json!({"sessionId": SESSION})));
    let outs = bridge.on_session(request(1, "session/new", json!({})));
    assert_eq!(
        response(&outs[0], 1).as_ref().unwrap()["sessionId"],
        SESSION
    );
}

#[test]
fn overlapping_opening_does_not_replace_the_requested_session() {
    let mut bridge = Bridge::default();
    let start = bridge.start();
    bridge.on_session(request(1, "session/load", json!({"sessionId": SESSION})));
    let outs = bridge.on_session(request(2, "session/load", json!({"sessionId": "other"})));
    assert_eq!(outs.len(), 1);
    assert!(response(&outs[0], 2).is_err());
    bridge.on_harness(&done(harness(&start[0]), json!({"sessionId": "full-id"})));
    let outs = bridge.on_harness(&done(harness(&start[1]), json!({"models": []})));
    assert_eq!(
        response(&outs[0], 1).as_ref().unwrap()["sessionId"],
        SESSION
    );
}

#[test]
fn a_finished_opening_can_be_followed_by_another_session() {
    let mut bridge = opened();
    let outs = bridge.on_session(request(
        2,
        "session/load",
        json!({"sessionId": "next-session"}),
    ));
    assert_eq!(
        response(&outs[0], 2).as_ref().unwrap()["sessionId"],
        "next-session"
    );
}

#[test]
fn a_later_opening_keeps_the_current_model_setting() {
    let mut bridge = opened();
    let calls = bridge.on_session(request(
        2,
        "session/set_config_option",
        json!({"sessionId": SESSION, "configId": "model", "value": "openai/gpt-6"}),
    ));
    bridge.on_harness(&done(
        harness(&calls[0]),
        json!({"provider": "openai", "id": "gpt-6"}),
    ));
    let outs = bridge.on_session(request(3, "session/new", json!({})));
    let opened = response(&outs[0], 3).as_ref().unwrap();
    assert_eq!(opened["configOptions"][0]["currentValue"], "openai/gpt-6");
}
