// ABOUTME: Exercises command delivery accounting across lost chat connections.
// ABOUTME: Keeps unsent prompts separate from writes with unknown outcomes.
use super::*;
use spaceport_chat_protocol::{Command, CommandId, Rejection};

fn current(session: &mut Session, at: Cursor) {
    session.receive(welcome()).unwrap();
    session.receive(snapshot(at, Vec::new())).unwrap();
}

fn submit(session: &mut Session) -> CommandId {
    let ClientMessage::Command { id, .. } = session.submit(Command::Cancel).unwrap() else {
        panic!("command")
    };
    id
}

#[test]
fn dispatch_waits_for_the_epoch_and_never_queues_a_command_twice() {
    let mut session = Session::default();
    session.begin();
    let id = submit(&mut session);
    session.receive(welcome()).unwrap();
    assert!(session.resend().is_empty());
    session.receive(snapshot(cursor(1, 0), Vec::new())).unwrap();
    assert!(
        matches!(&session.resend()[..], [ClientMessage::Command { id: sent, .. }] if *sent == id)
    );
    assert!(session.resend().is_empty());
    session.delivered(WriteOutcome::Written(id));
    assert!(session.resend().is_empty());
}

#[test]
fn a_written_command_stays_uncertain_after_same_epoch_reconnect() {
    let mut session = Session::default();
    current(&mut session, cursor(1, 0));
    let id = submit(&mut session);
    assert_eq!(session.resend().len(), 1);
    session.delivered(WriteOutcome::Written(id));
    session.lost(&Error::Closed);
    session.begin();
    assert!(session.resend().is_empty());
    current(&mut session, cursor(1, 4));
    assert!(session.resend().is_empty());
    assert_eq!(session.rejection(), Some(Refusal::DeliveryUnconfirmed));
}

#[test]
fn a_failed_write_is_unknown_and_is_never_replayed() {
    let mut session = Session::default();
    current(&mut session, cursor(1, 0));
    let id = submit(&mut session);
    assert_eq!(session.resend().len(), 1);
    assert!(session.delivered(WriteOutcome::Uncertain(id)));
    session.lost(&Error::Closed);
    session.begin();
    current(&mut session, cursor(1, 0));
    assert!(session.resend().is_empty());
    assert_eq!(session.rejection(), Some(Refusal::DeliveryUnconfirmed));
}

#[test]
fn neither_written_nor_unsent_commands_cross_a_conversation_epoch() {
    for written in [false, true] {
        let mut session = Session::default();
        current(&mut session, cursor(1, 0));
        let id = submit(&mut session);
        assert_eq!(session.resend().len(), 1);
        if written {
            session.delivered(WriteOutcome::Written(id));
        }
        session.lost(&Error::Closed);
        session.begin();
        current(&mut session, cursor(2, 0));
        assert!(session.resend().is_empty());
        assert_eq!(session.rejection(), Some(Refusal::SessionChanged));
        assert!(!session.delivered(WriteOutcome::Written(id)));
        assert_eq!(session.rejection(), Some(Refusal::SessionChanged));
    }
}

#[test]
fn a_desynced_transcript_keeps_new_commands_in_the_composer() {
    let mut session = Session::default();
    current(&mut session, cursor(1, 0));
    let error = session
        .receive(change(append(5, "gap"), cursor(1, 2)))
        .unwrap_err();
    session.lost(&error);
    assert!(matches!(
        session.submit(Command::Cancel),
        Err(Error::NotAttached)
    ));
}

#[test]
fn acknowledgements_correlate_and_can_arrive_before_the_writer_receipt() {
    let mut session = Session::default();
    current(&mut session, cursor(1, 0));
    let id = submit(&mut session);
    assert_eq!(session.resend().len(), 1);
    assert!(
        !session
            .receive(ServerMessage::Ack {
                id: CommandId::random(),
                rejected: Some(Rejection::NotReady),
            })
            .unwrap()
    );
    assert_eq!(session.rejection(), None);
    assert!(
        !session
            .receive(ServerMessage::Ack { id, rejected: None })
            .unwrap()
    );
    assert!(!session.delivered(WriteOutcome::Uncertain(id)));
    session.lost(&Error::Closed);
    session.begin();
    current(&mut session, cursor(1, 0));
    assert!(session.resend().is_empty());
    assert_eq!(session.rejection(), None);
}

#[test]
fn greeted_connections_without_writes_never_drop_a_prompt() {
    let mut session = Session::default();
    session.begin();
    session.receive(welcome()).unwrap();
    session.receive(snapshot(cursor(1, 0), Vec::new())).unwrap();
    let prompt = session
        .submit(Command::Prompt {
            text: "Keep this unsent prompt".into(),
            attachments: Vec::new(),
        })
        .unwrap();
    for _ in 0..4 {
        session.lost(&Error::Closed);
        session.begin();
        session.receive(welcome()).unwrap();
        assert!(
            session.resend().is_empty(),
            "a greeting does not prove the epoch"
        );
        session.receive(snapshot(cursor(1, 0), Vec::new())).unwrap();
        assert_eq!(session.resend(), vec![prompt.clone()]);
    }
    assert_eq!(session.rejection(), None);
}

#[test]
fn losing_unacknowledged_writes_frees_the_offline_queue() {
    let mut session = Session::default();
    current(&mut session, cursor(1, 0));
    let ids: Vec<_> = (0..MAX_PENDING).map(|_| submit(&mut session)).collect();
    assert_eq!(session.resend().len(), ids.len());
    for id in ids {
        session.delivered(WriteOutcome::Written(id));
    }
    session.lost(&Error::Closed);
    assert_eq!(session.rejection(), Some(Refusal::DeliveryUnconfirmed));
    submit(&mut session);
}
