use super::*;

fn acks(receiver: &Receiver<ServerMessage>) -> Vec<(CommandId, Option<Rejection>)> {
    drain(receiver)
        .into_iter()
        .filter_map(|message| match message {
            ServerMessage::Ack { id, rejected } => Some((id, rejected)),
            _ => None,
        })
        .collect()
}

#[test]
fn prompts_during_a_turn_queue_and_run_in_order() {
    let mut harness = ready();
    let first = prompt(&mut harness, 0, "one");
    harness.command(
        0,
        Command::Prompt {
            text: "two".to_owned(),
            attachments: Vec::new(),
        },
    );
    harness.command(
        0,
        Command::Prompt {
            text: "three".to_owned(),
            attachments: Vec::new(),
        },
    );
    assert!(harness.sent().is_empty());
    let queued: Vec<_> = harness
        .transcript()
        .queued
        .iter()
        .map(|queued| queued.text.as_str())
        .collect();
    assert_eq!(queued, ["two", "three"]);
    harness.respond(first, json!({"stopReason": "end_turn"}));
    let request = harness.last_request("session/prompt");
    assert_eq!(request["params"]["prompt"][0]["text"], "two");
    assert_eq!(harness.transcript().queued.len(), 1);
    assert_eq!(harness.transcript().status, Status::Working);
}

#[test]
fn prompts_before_the_session_opens_wait_for_it() {
    let mut harness = harness_with(config());
    harness.session.start();
    harness.command(
        0,
        Command::Prompt {
            text: "early".to_owned(),
            attachments: Vec::new(),
        },
    );
    assert_eq!(harness.transcript().queued.len(), 1);
    let id = harness.last_request("initialize")["id"].as_i64().unwrap();
    harness.respond(id, json!({"protocolVersion": 1, "agentCapabilities": {}}));
    let id = harness.last_request("session/new")["id"].as_i64().unwrap();
    harness.respond(id, json!({"sessionId": "s1"}));
    let request = harness.last_request("session/prompt");
    assert_eq!(request["params"]["prompt"][0]["text"], "early");
    assert!(harness.transcript().queued.is_empty());
}

#[test]
fn unqueue_withdraws_a_waiting_prompt() {
    let mut harness = ready();
    prompt(&mut harness, 0, "one");
    harness.command(
        0,
        Command::Prompt {
            text: "two".to_owned(),
            attachments: Vec::new(),
        },
    );
    let queued = harness.transcript().queued[0].id;
    harness.command(0, Command::Unqueue { id: queued });
    assert!(harness.transcript().queued.is_empty());
    harness.command(0, Command::Unqueue { id: queued });
    assert_eq!(
        harness.session.handled.back().unwrap().1,
        Some(Rejection::UnknownQueued)
    );
}

#[test]
fn cancel_ends_the_turn_and_drops_the_queue() {
    let mut harness = ready();
    let id = prompt(&mut harness, 0, "one");
    harness.command(
        0,
        Command::Prompt {
            text: "two".to_owned(),
            attachments: Vec::new(),
        },
    );
    harness.command(0, Command::Cancel);
    let cancel = harness.last_request("session/cancel");
    assert_eq!(cancel["params"]["sessionId"], "s1");
    assert!(cancel.get("id").is_none());
    assert!(harness.transcript().queued.is_empty());
    harness.respond(id, json!({"stopReason": "cancelled"}));
    assert_eq!(harness.transcript().status, Status::Idle);
    assert!(
        harness
            .sent()
            .iter()
            .all(|m| m["method"] != "session/prompt")
    );
    harness.command(0, Command::Cancel);
    assert_eq!(
        harness.session.handled.back().unwrap().1,
        Some(Rejection::NotWorking)
    );
}

#[test]
fn bad_prompts_are_rejected_with_a_reason() {
    let mut harness = ready();
    let client = harness.subscribe(1, false, None);
    let empty = harness.command(
        1,
        Command::Prompt {
            text: "  \n".to_owned(),
            attachments: Vec::new(),
        },
    );
    let long = harness.command(
        1,
        Command::Prompt {
            text: "x".repeat(spaceport_chat_protocol::MAX_PROMPT_SIZE + 1),
            attachments: Vec::new(),
        },
    );
    assert_eq!(
        acks(&client),
        [
            (empty, Some(Rejection::EmptyPrompt)),
            (long, Some(Rejection::PromptTooLong))
        ]
    );
}

#[test]
fn a_resent_command_is_acknowledged_without_running_twice() {
    let mut harness = ready();
    let client = harness.subscribe(1, false, None);
    let id = CommandId::random();
    for _ in 0..2 {
        harness.input(Input::Command {
            command: Command::Prompt {
                text: "once".to_owned(),
                attachments: Vec::new(),
            },
            conn: ConnId(1),
            id,
        });
    }
    let prompts = harness
        .sent()
        .into_iter()
        .filter(|message| message["method"] == "session/prompt")
        .count();
    assert_eq!(prompts, 1);
    assert_eq!(acks(&client), [(id, None), (id, None)]);
}

#[test]
fn a_command_resent_on_a_new_connection_is_acknowledged_without_running_twice() {
    let mut harness = ready();
    let first = harness.subscribe(1, false, None);
    let id = CommandId::random();
    let resent = Command::Prompt {
        text: "once".to_owned(),
        attachments: Vec::new(),
    };
    harness.input(Input::Command {
        command: resent.clone(),
        conn: ConnId(1),
        id,
    });
    // An ID still in the server cache keeps its outcome across connections.
    harness.input(Input::Detach(ConnId(1)));
    let second = harness.subscribe(2, false, None);
    harness.input(Input::Command {
        command: resent,
        conn: ConnId(2),
        id,
    });
    let prompts = harness
        .sent()
        .into_iter()
        .filter(|message| message["method"] == "session/prompt")
        .count();
    assert_eq!(prompts, 1, "one command id asks the agent once");
    assert!(
        harness.transcript().queued.is_empty(),
        "the repeat is not a second prompt waiting behind the first"
    );
    assert_eq!(acks(&first), [(id, None)]);
    assert_eq!(acks(&second), [(id, None)]);
}

#[test]
fn set_mode_asks_the_agent_then_records_the_mode() {
    let mut harness = ready();
    harness.command(
        0,
        Command::SetMode {
            id: "nope".to_owned(),
        },
    );
    assert_eq!(
        harness.session.handled.back().unwrap().1,
        Some(Rejection::UnknownMode)
    );
    harness.command(
        0,
        Command::SetMode {
            id: "plan".to_owned(),
        },
    );
    let request = harness.last_request("session/set_mode");
    assert_eq!(request["params"]["modeId"], "plan");
    assert_eq!(
        harness.transcript().modes.as_ref().unwrap().current,
        "default"
    );
    harness.respond(request["id"].as_i64().unwrap(), json!({}));
    assert_eq!(harness.transcript().modes.as_ref().unwrap().current, "plan");
}

#[test]
fn the_queue_is_bounded() {
    let mut harness = ready();
    prompt(&mut harness, 0, "first");
    for n in 0..40 {
        harness.command(
            0,
            Command::Prompt {
                text: format!("queued {n}"),
                attachments: Vec::new(),
            },
        );
    }
    assert_eq!(harness.transcript().queued.len(), 32);
    assert_eq!(
        harness.session.handled.back().unwrap().1,
        Some(Rejection::QueueFull)
    );
}

#[test]
fn a_lost_ack_does_not_repeat_a_prompt_after_another_client_evicts_its_id() {
    use spaceport_chat_core::{
        link::WriteOutcome,
        session::{Refusal, Session},
    };
    use spaceport_chat_protocol::ClientMessage;

    let mut host = ready();
    let mut client = Session::default();
    let first = host.subscribe(1, false, None);
    client.begin();
    for message in drain(&first) {
        client.receive(message).unwrap();
    }
    client
        .submit(Command::Prompt {
            text: "once".into(),
            attachments: Vec::new(),
        })
        .unwrap();
    let ClientMessage::Command { command, id } = client.resend().pop().unwrap() else {
        panic!("prompt")
    };
    host.input(Input::Command {
        command,
        conn: ConnId(1),
        id,
    });
    client.delivered(WriteOutcome::Written(id));
    host.input(Input::Detach(ConnId(1)));
    client.lost(&spaceport_chat_core::Error::Closed);
    let second = host.subscribe(2, false, None);
    for _ in 0..HANDLED {
        let other = host.command(2, Command::SetAutoAccept { on: true });
        assert_eq!(acks(&second), [(other, None)]);
    }
    assert!(!host.session.handled.iter().any(|(entry, _)| *entry == id));
    let next = host.subscribe(3, false, None);
    client.begin();
    for message in drain(&next) {
        client.receive(message).unwrap();
    }
    for message in client.resend() {
        if let ClientMessage::Command { command, id } = message {
            host.input(Input::Command {
                command,
                conn: ConnId(3),
                id,
            });
        }
    }
    assert_eq!(client.rejection(), Some(Refusal::DeliveryUnconfirmed));
    assert_eq!(
        host.sent()
            .iter()
            .filter(|m| m["method"] == "session/prompt")
            .count(),
        1
    );
    assert!(host.transcript().queued.is_empty());
}
