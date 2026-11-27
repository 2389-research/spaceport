// ABOUTME: Checks pure transcript and pending-command session contracts.
// ABOUTME: Covers acknowledgement, stream fencing, and pending command delivery.
use super::*;
use crate::link::WriteOutcome;
use spaceport_chat_protocol::{ByeReason, Change, Command, Entry, EntryKind, Status, Transcript};

mod delivery;
mod stream;

fn epoch(n: u128) -> uuid::Uuid {
    uuid::Uuid::from_u128(n)
}

fn cursor(n: u128, seq: u64) -> Cursor {
    Cursor {
        epoch: epoch(n),
        seq,
    }
}

fn welcome() -> ServerMessage {
    ServerMessage::Welcome {
        agent: "Claude".into(),
        pane: Some("p1".into()),
        version: VERSION,
    }
}

fn agent(id: u64, text: &str) -> Entry {
    Entry {
        id: EntryId(id),
        kind: EntryKind::Agent { text: text.into() },
    }
}

fn snapshot(at: Cursor, entries: Vec<Entry>) -> ServerMessage {
    let mut transcript = Transcript::new("Claude");
    transcript.entries = entries;
    transcript.status = Status::Idle;
    ServerMessage::Snapshot {
        cursor: at,
        transcript: Box::new(transcript),
    }
}

fn change(change: Change, at: Cursor) -> ServerMessage {
    ServerMessage::Change { change, cursor: at }
}

fn append(id: u64, text: &str) -> Change {
    Change::Append {
        id: EntryId(id),
        text: text.into(),
    }
}

fn live(at: Cursor, entries: Vec<Entry>) -> Session {
    let mut session = Session::default();
    session.begin();
    assert!(session.receive(welcome()).unwrap());
    assert!(session.receive(snapshot(at, entries)).unwrap());
    session
}

#[test]
fn daemon_status_updates_only_observers_and_preserves_the_wire_cursor() {
    let mut session = live(cursor(1, 0), vec![agent(1, "Existing conversation")]);
    assert!(!session.set_observed_status(Status::Working));
    let mut transcript = session.transcript().clone();
    transcript.read_only = true;
    session
        .receive(ServerMessage::Snapshot {
            cursor: cursor(2, 4),
            transcript: Box::new(transcript),
        })
        .unwrap();
    assert!(session.set_observed_status(Status::Working));
    assert_eq!(session.transcript().status, Status::Working);
    assert_eq!(session.since(), Some(cursor(2, 4)));
    assert!(!session.set_observed_status(Status::Working));
    assert!(session.resend().is_empty());
}

#[test]
fn a_read_only_transcript_cannot_queue_commands() {
    let mut session = live(cursor(1, 0), vec![agent(1, "Existing CLI conversation")]);
    let mut transcript = session.transcript().clone();
    transcript.read_only = true;
    session
        .receive(ServerMessage::Snapshot {
            cursor: cursor(2, 0),
            transcript: Box::new(transcript),
        })
        .unwrap();
    for command in [
        Command::Cancel,
        Command::Prompt {
            text: "Continue".into(),
            attachments: Vec::new(),
        },
    ] {
        assert!(matches!(session.submit(command), Err(Error::ReadOnly)));
    }
    assert!(session.resend().is_empty());
}

#[test]
fn changes_apply_in_order_after_a_snapshot() {
    let mut session = live(cursor(1, 4), vec![agent(1, "Hel")]);
    assert!(session.live());
    assert!(session.synced());
    assert_eq!(session.agent(), "Claude");
    session
        .receive(change(append(1, "lo"), cursor(1, 5)))
        .unwrap();
    session
        .receive(change(
            Change::Status {
                status: Status::Working,
            },
            cursor(1, 6),
        ))
        .unwrap();
    assert_eq!(session.transcript().entries[0].kind.text(), Some("Hello"));
    assert_eq!(session.transcript().status, Status::Working);
    assert_eq!(session.since(), Some(cursor(1, 6)));
}

#[test]
fn nothing_counts_before_the_greeting() {
    let mut session = Session::default();
    session.begin();
    assert!(matches!(
        session.receive(snapshot(cursor(1, 0), Vec::new())),
        Err(Error::Handshake)
    ));
    assert!(!session.synced());
}

#[test]
fn another_protocol_version_is_permanent() {
    let mut session = Session::default();
    session.begin();
    let error = session
        .receive(ServerMessage::Welcome {
            agent: "x".into(),
            pane: None,
            version: VERSION + 1,
        })
        .unwrap_err();
    assert!(matches!(error, Error::Version(_)));
    assert!(error.permanent());
    assert!(Error::Bye(ByeReason::Version { supported: 9 }).permanent());
    assert!(!Error::Bye(ByeReason::Exiting).permanent());
}

#[test]
fn a_gap_in_the_stream_asks_for_a_snapshot_next() {
    let mut session = live(cursor(1, 4), vec![agent(1, "a")]);
    let error = session
        .receive(change(append(1, "b"), cursor(1, 6)))
        .unwrap_err();
    assert!(matches!(error, Error::Stream { .. }));
    assert_eq!(session.transcript().entries[0].kind.text(), Some("a"));
    session.lost(&error);
    assert_eq!(session.since(), None);
    assert!(!session.live());
    // The old transcript stays on screen until the snapshot arrives.
    assert!(session.synced());
}

#[test]
fn a_change_from_another_epoch_does_not_apply() {
    let mut session = live(cursor(1, 4), vec![agent(1, "a")]);
    let error = session
        .receive(change(append(1, "b"), cursor(2, 5)))
        .unwrap_err();
    assert!(matches!(error, Error::Stream { .. }));
    assert!(error.desynced());
}

#[test]
fn a_change_that_does_not_fit_leaves_the_transcript() {
    let mut session = live(cursor(1, 0), vec![agent(1, "a")]);
    let error = session
        .receive(change(append(9, "b"), cursor(1, 1)))
        .unwrap_err();
    assert!(matches!(error, Error::Apply(_)));
    assert!(std::error::Error::source(&error).is_some());
    assert_eq!(session.transcript().entries, vec![agent(1, "a")]);
    // The position is gone at once, not only when the connection ends.
    assert_eq!(session.since(), None);
    session.lost(&error);
    assert_eq!(session.since(), None);
}

#[test]
fn a_dropped_connection_resumes_from_its_cursor() {
    let mut session = live(cursor(1, 3), Vec::new());
    session.lost(&Error::Closed);
    assert_eq!(session.since(), Some(cursor(1, 3)));
    session.begin();
    session.receive(welcome()).unwrap();
    assert!(!session.live(), "the greeting has not confirmed the epoch");
    session
        .receive(change(
            Change::Status {
                status: Status::Idle,
            },
            cursor(1, 4),
        ))
        .unwrap();
    assert!(session.live());
}

#[test]
fn a_new_snapshot_replaces_the_transcript() {
    let mut session = live(cursor(1, 3), vec![agent(1, "old")]);
    session
        .receive(snapshot(cursor(2, 0), vec![agent(5, "new")]))
        .unwrap();
    assert_eq!(session.transcript().entries.len(), 1);
    assert_eq!(session.transcript().entries[0].id, EntryId(5));
    assert_eq!(session.snapshots(), 2);
}

#[test]
fn commands_wait_for_their_acknowledgement() {
    let mut session = live(cursor(1, 0), Vec::new());
    let ClientMessage::Command { id, .. } = session
        .submit(Command::Prompt {
            text: "hi".into(),
            attachments: Vec::new(),
        })
        .unwrap()
    else {
        panic!("a command");
    };
    session.submit(Command::Cancel).unwrap();
    let resent = session.resend();
    assert_eq!(resent.len(), 2);
    assert!(matches!(&resent[0], ClientMessage::Command { id: first, .. } if *first == id));
    assert!(
        !session
            .receive(ServerMessage::Ack { id, rejected: None })
            .unwrap()
    );
    assert_eq!(session.rejection(), None);
}

#[test]
fn only_unwritten_commands_survive_a_lost_link() {
    let mut session = live(cursor(1, 0), Vec::new());
    let ClientMessage::Command { id, .. } =
        session.submit(Command::SetAutoAccept { on: true }).unwrap()
    else {
        panic!("command")
    };
    assert_eq!(session.resend().len(), 1);
    session.delivered(WriteOutcome::Written(id));
    let queued = session.submit(Command::Cancel).unwrap();
    session.lost(&Error::Closed);
    session.begin();
    session.receive(welcome()).unwrap();
    session.receive(snapshot(cursor(1, 0), Vec::new())).unwrap();
    assert_eq!(session.resend(), vec![queued]);
    assert_eq!(session.rejection(), Some(Refusal::DeliveryUnconfirmed));
}

#[test]
fn an_outage_alone_never_drops_a_prompt() {
    let mut session = live(cursor(1, 0), Vec::new());
    session.lost(&Error::Closed);
    session
        .submit(Command::Prompt {
            text: "while away".into(),
            attachments: Vec::new(),
        })
        .unwrap();
    // Connections that never greeted do not count against it.
    for _ in 0..10 {
        session.begin();
        assert!(
            session.resend().is_empty(),
            "wait for an epoch-bearing message"
        );
        session.lost(&Error::Closed);
    }
    session.begin();
    session.receive(welcome()).unwrap();
    session.receive(snapshot(cursor(1, 0), Vec::new())).unwrap();
    assert_eq!(session.resend().len(), 1);
    assert_eq!(session.rejection(), None);
}

#[test]
fn a_refusal_shows_but_a_lost_race_does_not() {
    let mut session = live(cursor(1, 0), Vec::new());
    let ack = |id, rejected| ServerMessage::Ack {
        id,
        rejected: Some(rejected),
    };
    let ClientMessage::Command { id, .. } = session.submit(Command::Cancel).unwrap() else {
        panic!("command")
    };
    assert!(
        !session
            .receive(ack(id, Rejection::UnknownPermission))
            .unwrap()
    );
    assert_eq!(session.rejection(), None);
    let ClientMessage::Command { id, .. } = session.submit(Command::Cancel).unwrap() else {
        panic!("command")
    };
    assert!(session.receive(ack(id, Rejection::NotReady)).unwrap());
    assert_eq!(
        session.rejection(),
        Some(Refusal::Remote(Rejection::NotReady))
    );
    session.submit(Command::Cancel).unwrap();
    assert_eq!(session.rejection(), None);
}

#[test]
fn waiting_commands_are_bounded() {
    let mut session = Session::default();
    for _ in 0..MAX_PENDING {
        session.submit(Command::Cancel).unwrap();
    }
    assert!(matches!(session.submit(Command::Cancel), Err(Error::Busy)));
}

#[test]
fn only_the_changed_entry_moves_its_revision() {
    let mut session = live(cursor(1, 0), vec![agent(1, "a"), agent(2, "b")]);
    let before = session.keys();
    session
        .receive(change(append(2, "c"), cursor(1, 1)))
        .unwrap();
    let after = session.keys();
    assert_eq!(before[0], after[0]);
    assert_ne!(before[1], after[1]);
    session
        .receive(change(
            Change::Truncate { before: EntryId(2) },
            cursor(1, 2),
        ))
        .unwrap();
    assert_eq!(session.keys(), vec![after[1]]);
}

#[test]
fn goodbye_ends_the_connection() {
    let mut session = live(cursor(1, 0), Vec::new());
    assert!(matches!(
        session.receive(ServerMessage::Bye {
            reason: ByeReason::Exiting
        }),
        Err(Error::Bye(ByeReason::Exiting))
    ));
}
