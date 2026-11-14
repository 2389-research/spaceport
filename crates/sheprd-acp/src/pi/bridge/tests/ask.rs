// ABOUTME: Exercises the restricted Pi translation without provider calls.
// ABOUTME: Covers literal input, actual acceptance, and queue-safe cancellation.
use super::*;

#[test]
fn restricted_input_is_literal_and_acceptance_is_an_observed_receipt() {
    let mut bridge = opened();
    bridge.restrict();
    let outs = bridge.on_session(request(
        2,
        "session/prompt",
        json!({"sessionId":SESSION,"prompt":[{"type":"text","text":"/bash dangerous"}]}),
    ));
    let command = harness(&outs[0]).clone();
    assert!(!command["message"].as_str().unwrap().starts_with('/'));
    assert!(
        command["message"]
            .as_str()
            .unwrap()
            .contains("/bash dangerous")
    );
    let accepted = bridge.on_harness(&done(&command, json!({"disposition":"started"})));
    assert!(
        matches!(&accepted[0], Out::Session(Inbound::Notification{method,..}) if method=="spaceport/ask_accepted")
    );
}

#[test]
fn restricted_cancel_clears_pi_queue_before_abort() {
    let mut bridge = opened();
    prompt(&mut bridge, 2);
    bridge.restrict();
    let outs = bridge.on_session(Inbound::Notification {
        method: "session/cancel".into(),
        params: json!({}),
    });
    assert_eq!(harness(&outs[0])["type"], "clear_queue");
    assert_eq!(harness(&outs[1])["type"], "abort");
}

#[test]
fn restricted_bridge_refuses_attachments_and_settings() {
    let mut bridge = opened();
    bridge.restrict();
    let outs = bridge.on_session(request(2,"session/prompt",json!({"sessionId":SESSION,"prompt":[{"type":"image","mimeType":"image/png","data":"image"}]})));
    assert!(response(&outs[0], 2).is_err());
    let outs = bridge.on_session(request(
        3,
        "session/set_config_option",
        json!({"sessionId":SESSION,"configId":"model","value":"openai/gpt-6"}),
    ));
    assert!(response(&outs[0], 3).is_err());
}

#[test]
fn restricted_queue_or_handled_receipts_never_claim_an_active_turn() {
    for disposition in ["queued", "handled", "unknown"] {
        let mut bridge = opened();
        bridge.restrict();
        let outs = bridge.on_session(request(
            2,
            "session/prompt",
            json!({"sessionId":SESSION,"prompt":[{"type":"text","text":"question"}]}),
        ));
        let command = harness(&outs[0]).clone();
        let outs = bridge.on_harness(&done(&command, json!({"disposition":disposition})));
        assert!(
            outs.iter()
                .any(|out| matches!(out, Out::Session(Inbound::Response { result: Err(_), .. })))
        );
        assert!(!outs.iter().any(|out| matches!(out,Out::Session(Inbound::Notification{method,..}) if method=="spaceport/ask_accepted")));
    }
}
