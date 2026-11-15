use super::*;

fn agent_closed(harness: &mut Harness) {
    harness.input(Input::Agent(Inbound::Closed(Some(
        crate::Error::LineTooLong { limit: 16 },
    ))));
}

#[test]
fn a_failure_during_a_turn_ends_the_turn_and_withdraws_its_questions() {
    let mut harness = ready();
    prompt(&mut harness, 0, "test");
    ask_permission(&mut harness, 77);
    assert_eq!(harness.reports.borrow().last(), Some(&Activity::Blocked));

    agent_closed(&mut harness);

    assert!(matches!(harness.transcript().status, Status::Failed { .. }));
    assert!(harness.transcript().permissions.is_empty());
    assert!(!harness.transcript().is_blocked());
    assert_eq!(harness.transcript().turn, None);
    assert_eq!(harness.reports.borrow().last(), Some(&Activity::Idle));
}

#[test]
fn a_late_prompt_reply_after_a_failure_changes_nothing() {
    let mut harness = ready();
    let id = prompt(&mut harness, 0, "test");
    agent_closed(&mut harness);
    let entries = harness.transcript().entries.len();

    harness.respond(id, json!({"stopReason": "end_turn"}));

    assert_eq!(harness.transcript().entries.len(), entries);
    assert!(matches!(harness.transcript().status, Status::Failed { .. }));
}

#[test]
fn a_late_session_reply_after_a_failure_does_not_reopen_the_session() {
    let mut harness = harness_with(config());
    harness.session.start();
    let id = harness.last_request("initialize")["id"].as_i64().unwrap();
    harness.respond(id, json!({"protocolVersion": 1, "agentCapabilities": {}}));
    let id = harness.last_request("session/new")["id"].as_i64().unwrap();
    agent_closed(&mut harness);

    harness.respond(id, json!({"sessionId": "s1"}));

    assert!(matches!(harness.transcript().status, Status::Failed { .. }));
    harness.command(
        0,
        Command::Prompt {
            text: "hi".to_owned(),
            attachments: Vec::new(),
        },
    );
    assert!(harness.sent().is_empty());
}

#[test]
fn a_setting_change_after_the_agent_exits_is_refused_and_stays_exited() {
    let mut harness = ready();
    harness.input(Input::AgentExited {
        code: Some(0),
        stderr: Vec::new(),
    });
    harness.sent();

    harness.command(
        0,
        Command::SetMode {
            id: "plan".to_owned(),
        },
    );

    assert_eq!(
        harness.session.handled.back().unwrap().1,
        Some(Rejection::NotReady)
    );
    assert!(harness.sent().is_empty());
    assert!(matches!(harness.transcript().status, Status::Exited { .. }));
}

#[test]
fn a_second_failure_adds_no_second_notice() {
    let mut harness = ready();
    agent_closed(&mut harness);
    let notices = harness.notices().len();

    agent_closed(&mut harness);

    assert_eq!(harness.notices().len(), notices);
}
