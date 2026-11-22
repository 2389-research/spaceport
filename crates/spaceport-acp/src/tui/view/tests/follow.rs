//! How the view follows the session's change stream, and what it keeps when
//! a change does not follow.

use super::*;

fn shown(view: &View) -> &Transcript {
    view.transcript()
}

fn agent(id: u64, text: &str) -> Entry {
    Entry {
        id: EntryId(id),
        kind: EntryKind::Agent {
            text: text.to_owned(),
        },
    }
}

fn append(id: u64, text: &str) -> Change {
    Change::Append {
        id: EntryId(id),
        text: text.to_owned(),
    }
}

fn following(entries: Vec<Entry>, at: Cursor) -> View {
    let mut view = View::default();
    let transcript = Transcript {
        entries,
        ..Transcript::new("Claude")
    };
    view.receive(ServerMessage::Snapshot {
        cursor: at,
        transcript: Box::new(transcript),
    });
    view
}

fn at(seq: u64) -> Cursor {
    Cursor {
        epoch: Uuid::from_u128(1),
        seq,
    }
}

fn change(view: &mut View, change: Change, cursor: Cursor) -> Received {
    view.receive(ServerMessage::Change { change, cursor })
}

#[test]
fn a_change_before_any_snapshot_asks_for_one() {
    let mut view = View::default();
    assert_eq!(
        change(
            &mut view,
            Change::Push {
                entry: agent(1, "a")
            },
            at(1)
        ),
        Received::Resync
    );
    assert!(shown(&view).entries.is_empty());
}

#[test]
fn a_gap_keeps_the_stale_transcript_on_screen() {
    let mut view = following(vec![agent(1, "a")], at(4));
    assert_eq!(change(&mut view, append(1, "b"), at(6)), Received::Resync);
    assert_eq!(view.cursor(), None);
    assert_eq!(shown(&view).entries, vec![agent(1, "a")]);
    // Without a position, even the change that would have followed is refused.
    assert_eq!(change(&mut view, append(1, "b"), at(5)), Received::Resync);
    assert_eq!(shown(&view).entries, vec![agent(1, "a")]);
}

#[test]
fn a_change_from_another_epoch_asks_for_a_snapshot() {
    let mut view = following(vec![agent(1, "a")], at(4));
    let other = Cursor {
        epoch: Uuid::from_u128(2),
        seq: 5,
    };
    assert_eq!(change(&mut view, append(1, "b"), other), Received::Resync);
    assert_eq!(view.cursor(), None);
    assert_eq!(shown(&view).entries, vec![agent(1, "a")]);
}

#[test]
fn a_change_that_does_not_fit_leaves_the_transcript_and_forgets_the_position() {
    let mut view = following(vec![agent(1, "a")], at(4));
    assert_eq!(change(&mut view, append(9, "b"), at(5)), Received::Resync);
    assert_eq!(view.cursor(), None);
    assert_eq!(shown(&view).entries, vec![agent(1, "a")]);
}

#[test]
fn a_snapshot_after_a_desync_restores_following() {
    let mut view = following(vec![agent(1, "a")], at(4));
    assert_eq!(change(&mut view, append(1, "b"), at(9)), Received::Resync);
    view.receive(ServerMessage::Snapshot {
        cursor: at(9),
        transcript: Box::new(Transcript {
            entries: vec![agent(1, "ab")],
            ..Transcript::new("Claude")
        }),
    });
    assert_eq!(view.cursor(), Some(at(9)));
    assert_eq!(change(&mut view, append(1, "c"), at(10)), Received::Applied);
    assert_eq!(view.cursor(), Some(at(10)));
    assert_eq!(shown(&view).entries, vec![agent(1, "abc")]);
}

#[test]
fn the_last_sequence_number_has_no_successor() {
    let mut view = following(vec![agent(1, "a")], at(u64::MAX));
    // Neither a wrapped count nor a repeat follows the last position.
    assert_eq!(change(&mut view, append(1, "b"), at(0)), Received::Resync);
    assert_eq!(view.cursor(), None);
    assert_eq!(shown(&view).entries, vec![agent(1, "a")]);
}
