//! How a session follows the program's change stream, and what it keeps
//! when a change does not follow.

use super::*;

fn live(at: Cursor, entries: Vec<Entry>) -> Session {
    let mut session = Session::default();
    session.begin();
    session.receive(welcome()).unwrap();
    session.receive(snapshot(at, entries)).unwrap();
    session
}

#[test]
fn a_gap_reports_both_positions_and_keeps_the_transcript() {
    let mut session = live(cursor(1, 4), vec![agent(1, "a")]);
    let error = session
        .receive(change(append(1, "b"), cursor(1, 6)))
        .unwrap_err();
    assert!(matches!(
        error,
        Error::Stream { after, got } if after == Some(cursor(1, 4)) && got == cursor(1, 6)
    ));
    assert!(error.desynced());
    assert_eq!(session.transcript().entries, vec![agent(1, "a")]);
}

#[test]
fn another_epoch_reports_both_positions_and_keeps_the_transcript() {
    let mut session = live(cursor(1, 4), vec![agent(1, "a")]);
    let error = session
        .receive(change(append(1, "b"), cursor(2, 5)))
        .unwrap_err();
    assert!(matches!(
        error,
        Error::Stream { after, got } if after == Some(cursor(1, 4)) && got == cursor(2, 5)
    ));
    assert_eq!(session.transcript().entries, vec![agent(1, "a")]);
    session.lost(&error);
    assert_eq!(session.since(), None);
}

#[test]
fn a_change_that_does_not_fit_carries_the_reducer_error() {
    let mut session = live(cursor(1, 0), vec![agent(1, "a")]);
    let error = session
        .receive(change(append(9, "b"), cursor(1, 1)))
        .unwrap_err();
    assert!(matches!(
        &error,
        Error::Apply(spaceport_chat_protocol::Error::UnknownEntry(EntryId(9)))
    ));
    assert!(error.desynced());
    assert_eq!(session.transcript().entries, vec![agent(1, "a")]);
}

#[test]
fn a_change_before_any_snapshot_does_not_follow() {
    let mut session = Session::default();
    session.begin();
    session.receive(welcome()).unwrap();
    let error = session
        .receive(change(append(1, "b"), cursor(1, 1)))
        .unwrap_err();
    assert!(matches!(
        error,
        Error::Stream { after: None, got } if got == cursor(1, 1)
    ));
    assert!(!session.synced());
}

#[test]
fn a_snapshot_after_a_desync_restores_following() {
    let mut session = live(cursor(1, 4), vec![agent(1, "a")]);
    let error = session
        .receive(change(append(1, "b"), cursor(1, 9)))
        .unwrap_err();
    session.lost(&error);
    assert!(!session.live());
    session.begin();
    session.receive(welcome()).unwrap();
    assert!(!session.live());
    session
        .receive(snapshot(cursor(1, 9), vec![agent(1, "ab")]))
        .unwrap();
    assert!(session.live());
    session
        .receive(change(append(1, "c"), cursor(1, 10)))
        .unwrap();
    assert_eq!(session.since(), Some(cursor(1, 10)));
    assert_eq!(session.transcript().entries, vec![agent(1, "abc")]);
    assert_eq!(session.snapshots(), 2);
}

#[test]
fn the_last_sequence_number_has_no_successor() {
    let mut session = live(cursor(1, u64::MAX), vec![agent(1, "a")]);
    let error = session
        .receive(change(append(1, "b"), cursor(1, 0)))
        .unwrap_err();
    assert!(matches!(
        error,
        Error::Stream { after, got } if after == Some(cursor(1, u64::MAX)) && got == cursor(1, 0)
    ));
    assert_eq!(session.transcript().entries, vec![agent(1, "a")]);
}
