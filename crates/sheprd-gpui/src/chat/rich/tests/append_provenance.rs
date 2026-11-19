// ABOUTME: Fences prepared reuse across replacements, snapshots, truncation, and locale resets.
// ABOUTME: Exercises actual Session reducer events and retained immutable message bodies.
use super::{append_work::*, *};
use crate::chat::selection::{Cell, Region};
use sheprd_chat_protocol::{Change, Entry, EntryKind, ServerMessage, Transcript};

fn replacement(text: &str) -> Change {
    Change::Replace {
        entry: Entry {
            id: ID,
            kind: EntryKind::Agent { text: text.into() },
        },
    }
}

#[test]
fn coalesced_replace_and_append_cannot_extend_an_old_body() {
    let (mut session, mut prepared) = seeded("alpha seed ready");
    let previous = session.keys()[0].1;
    let old_key = prepared
        .text()
        .key(ID, Region::Body(0), Cell::block(0))
        .unwrap();
    apply(&mut session, replacement("omega seed ready"));
    apply(
        &mut session,
        Change::Append {
            id: ID,
            text: " next".into(),
        },
    );
    assert!(!session.appended_since(ID, previous));
    prepare(&mut prepared, &session);
    same_as_full(&prepared, &session);
    assert!(prepared.text().segment(old_key).is_none());

    let current = session.keys()[0].1;
    apply(
        &mut session,
        Change::Append {
            id: ID,
            text: " more".into(),
        },
    );
    assert!(session.appended_since(ID, current));
    let (_, counts) = work::measure(|| prepare(&mut prepared, &session));
    assert_eq!(counts.markdown_bytes, 0);
    same_as_full(&prepared, &session);
}

#[test]
fn snapshot_and_append_are_fenced_even_before_the_view_discards_its_old_body() {
    let (mut session, mut prepared) = seeded("alpha seed ready");
    let previous = session.keys()[0].1;
    let mut transcript = Transcript::new("Generated snapshot");
    transcript.entries.push(Entry {
        id: ID,
        kind: EntryKind::Agent {
            text: "omega seed ready".into(),
        },
    });
    session
        .receive(ServerMessage::Snapshot {
            cursor: cursor(20),
            transcript: Box::new(transcript),
        })
        .unwrap();
    apply(
        &mut session,
        Change::Append {
            id: ID,
            text: " next".into(),
        },
    );
    assert!(!session.appended_since(ID, previous));
    prepare(&mut prepared, &session);
    same_as_full(&prepared, &session);
    let current = session.keys()[0].1;
    apply(
        &mut session,
        Change::Append {
            id: ID,
            text: " more".into(),
        },
    );
    assert!(session.appended_since(ID, current));
    prepare(&mut prepared, &session);
    same_as_full(&prepared, &session);
}

#[test]
fn truncation_and_reused_entry_ids_do_not_reuse_old_source_spans() {
    let (mut session, mut prepared) = seeded("alpha seed ready");
    let previous = session.keys()[0].1;
    apply(
        &mut session,
        Change::Truncate {
            before: sheprd_chat_protocol::EntryId(2),
        },
    );
    prepare(&mut prepared, &session);
    assert!(prepared.body(ID).is_none());
    assert!(!session.appended_since(ID, previous));
    apply(
        &mut session,
        Change::Push {
            entry: Entry {
                id: ID,
                kind: EntryKind::Agent {
                    text: "omega seed ready".into(),
                },
            },
        },
    );
    apply(
        &mut session,
        Change::Append {
            id: ID,
            text: " next".into(),
        },
    );
    prepare(&mut prepared, &session);
    same_as_full(&prepared, &session);
}

#[test]
fn forgetting_text_and_clearing_the_view_discard_append_spans() {
    let (mut session, mut prepared) = seeded("$x$ alpha seed ready");
    prepared.forget_text();
    apply(
        &mut session,
        Change::Append {
            id: ID,
            text: " next".into(),
        },
    );
    prepare(&mut prepared, &session);
    same_as_full(&prepared, &session);
    prepared.clear();
    apply(
        &mut session,
        Change::Append {
            id: ID,
            text: " more".into(),
        },
    );
    prepare(&mut prepared, &session);
    same_as_full(&prepared, &session);
}
