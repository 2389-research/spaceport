// ABOUTME: Proves the last GUI detaching leaves a headless agent session alive.
// ABOUTME: Reattachment uses the same session without ACP lifecycle commands.
use super::*;

#[test]
fn the_last_gui_detaches_without_ending_or_restarting_the_headless_session() {
    let mut harness = harness_with(Config {
        exit_with_agent: true,
        ..config()
    });
    harness.session.start();
    let initialize = harness.last_request("initialize");
    harness.respond(
        initialize["id"].as_i64().unwrap(),
        json!({"protocolVersion": 1, "agentCapabilities": {}}),
    );
    let new = harness.last_request("session/new");
    harness.respond(new["id"].as_i64().unwrap(), json!({"sessionId": "kept"}));
    let gui = harness.subscribe(1, false, None);
    assert_eq!(harness.transcript().clients, 1);
    let session = harness.transcript().session.clone();
    harness.sent();

    assert_eq!(harness.input(Input::Detach(ConnId(1))), Flow::Continue);
    assert_eq!(harness.transcript().clients, 0);
    assert_eq!(harness.transcript().session, session);
    assert_eq!(harness.transcript().status, Status::Idle);
    assert!(harness.sent().is_empty(), "detach sends no ACP command");
    assert!(
        !drain(&gui)
            .iter()
            .any(|message| matches!(message, ServerMessage::Bye { .. }))
    );

    let _reattached = harness.subscribe(2, false, None);
    assert_eq!(harness.transcript().clients, 1);
    assert!(harness.sent().is_empty(), "reattach sends no ACP command");
    harness.command(
        2,
        Command::Prompt {
            text: "still here".into(),
            attachments: Vec::new(),
        },
    );
    let sent = harness.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0]["method"], "session/prompt");
    assert_eq!(sent[0]["params"]["sessionId"], "kept");
    assert_eq!(sent[0]["params"]["prompt"][0]["text"], "still here");
    harness.respond(
        sent[0]["id"].as_i64().unwrap(),
        json!({"stopReason": "end_turn"}),
    );
    assert_eq!(harness.transcript().status, Status::Idle);
    assert_eq!(harness.transcript().session, session);
}
