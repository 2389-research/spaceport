use super::*;

fn at(epoch: u128, seq: u64) -> Cursor {
    Cursor {
        epoch: uuid::Uuid::from_u128(epoch),
        seq,
    }
}

fn append(id: u64, text: &str) -> Change {
    Change::Append {
        id: EntryId(id),
        text: text.to_owned(),
    }
}

fn following(cursor: Cursor) -> Replica {
    let mut transcript = Transcript::new("Claude");
    transcript.entries = vec![agent(1, "a"), agent(2, "b")];
    let mut replica = Replica::default();
    replica.reset(cursor, transcript);
    replica
}

fn texts(replica: &Replica) -> Vec<Option<&str>> {
    replica
        .transcript()
        .entries
        .iter()
        .map(|entry| entry.kind.text())
        .collect()
}

#[test]
fn a_new_replica_needs_a_snapshot() {
    let mut replica = Replica::default();
    assert_eq!(replica.since(), None);
    let error = replica.follow(&append(1, "x"), at(1, 1)).unwrap_err();
    assert!(matches!(
        error,
        Error::StreamGap { after: None, got } if got == at(1, 1)
    ));
}

#[test]
fn the_next_change_applies_and_moves_the_position() {
    let mut replica = following(at(1, 4));
    assert_eq!(
        replica.follow(&append(2, "c"), at(1, 5)).unwrap(),
        Touched::Entry(EntryId(2))
    );
    assert_eq!(replica.since(), Some(at(1, 5)));
    assert_eq!(texts(&replica), [Some("a"), Some("bc")]);
}

#[test]
fn a_change_that_does_not_follow_reports_both_positions() {
    for got in [at(1, 4), at(1, 6), at(2, 5), at(1, 0)] {
        let mut replica = following(at(1, 4));
        let error = replica.follow(&append(2, "c"), got).unwrap_err();
        assert!(matches!(
            error,
            Error::StreamGap { after, got: refused }
                if after == Some(at(1, 4)) && refused == got
        ));
        assert_eq!(replica.since(), None, "{got:?}");
        assert_eq!(texts(&replica), [Some("a"), Some("b")]);
    }
}

#[test]
fn the_last_sequence_number_has_no_successor() {
    let mut replica = following(at(1, u64::MAX));
    let error = replica.follow(&append(2, "c"), at(1, 0)).unwrap_err();
    assert!(matches!(error, Error::StreamGap { .. }));
    assert_eq!(replica.since(), None);
}

#[test]
fn a_change_that_does_not_fit_keeps_the_transcript_and_forgets_the_position() {
    let mut replica = following(at(1, 4));
    let error = replica.follow(&append(9, "c"), at(1, 5)).unwrap_err();
    assert!(matches!(error, Error::UnknownEntry(EntryId(9))));
    assert_eq!(replica.since(), None);
    assert_eq!(texts(&replica), [Some("a"), Some("b")]);
    // The change that would have followed is refused too, until a snapshot.
    assert!(matches!(
        replica.follow(&append(2, "c"), at(1, 5)),
        Err(Error::StreamGap { after: None, .. })
    ));
}

#[test]
fn a_snapshot_restores_following() {
    let mut replica = following(at(1, 4));
    replica.follow(&append(2, "c"), at(1, 9)).unwrap_err();
    let mut transcript = Transcript::new("Claude");
    transcript.entries = vec![agent(7, "new")];
    replica.reset(at(2, 0), transcript);
    assert_eq!(replica.since(), Some(at(2, 0)));
    replica.follow(&append(7, "er"), at(2, 1)).unwrap();
    assert_eq!(texts(&replica), [Some("newer")]);
}

#[test]
fn losing_the_position_keeps_the_transcript() {
    let mut replica = following(at(1, 4));
    replica.lose();
    assert_eq!(replica.since(), None);
    assert_eq!(texts(&replica), [Some("a"), Some("b")]);
}

#[test]
fn an_edit_from_outside_the_stream_keeps_the_position() {
    let mut replica = following(at(1, 4));
    replica.transcript_mut().status = Status::Working;
    assert_eq!(replica.since(), Some(at(1, 4)));
    assert_eq!(replica.transcript().status, Status::Working);
}

#[test]
fn entry_changes_name_what_they_touch() {
    assert_eq!(
        Change::Push {
            entry: agent(3, "x")
        }
        .touched(),
        Touched::Entry(EntryId(3))
    );
    assert_eq!(
        Change::Replace {
            entry: agent(4, "x")
        }
        .touched(),
        Touched::Entry(EntryId(4))
    );
    assert_eq!(append(5, "x").touched(), Touched::Entry(EntryId(5)));
    assert_eq!(
        Change::Truncate { before: EntryId(6) }.touched(),
        Touched::Truncated { before: EntryId(6) }
    );
    assert_eq!(Change::Clients { count: 2 }.touched(), Touched::Other);
}
