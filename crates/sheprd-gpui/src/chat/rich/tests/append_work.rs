// ABOUTME: Bounds streaming parse work through actual Session append events.
// ABOUTME: Compares prepared messages and selectable text with fresh full preparation.
use super::*;
use crate::chat::{
    prepared::{Body, Prepared},
    selection::Region,
    session::Session,
};
use sheprd_chat_protocol::{
    Change, Cursor, Entry, EntryId, EntryKind, ServerMessage, Transcript, VERSION,
};

pub(super) const ID: EntryId = EntryId(1);

pub(super) fn cursor(seq: u64) -> Cursor {
    Cursor {
        epoch: uuid::Uuid::from_u128(7),
        seq,
    }
}

pub(super) fn session(text: &str) -> Session {
    let mut session = Session::default();
    session.begin();
    session
        .receive(ServerMessage::Welcome {
            agent: "Generated".into(),
            pane: None,
            version: VERSION,
        })
        .unwrap();
    let mut transcript = Transcript::new("Generated");
    transcript.entries.push(Entry {
        id: ID,
        kind: EntryKind::Agent { text: text.into() },
    });
    session
        .receive(ServerMessage::Snapshot {
            cursor: cursor(0),
            transcript: Box::new(transcript),
        })
        .unwrap();
    session
}

pub(super) fn apply(session: &mut Session, change: Change) {
    let seq = session.since().unwrap().seq + 1;
    session
        .receive(ServerMessage::Change {
            cursor: cursor(seq),
            change,
        })
        .unwrap();
}

pub(super) fn prepare(prepared: &mut Prepared, session: &Session) {
    prepared.update(session.transcript(), &session.keys(), |id, revision| {
        session.appended_since(id, revision)
    });
}

pub(super) fn seeded(text: &str) -> (Session, Prepared) {
    let mut session = session(text);
    apply(
        &mut session,
        Change::Replace {
            entry: Entry {
                id: ID,
                kind: EntryKind::Agent { text: text.into() },
            },
        },
    );
    let mut prepared = Prepared::default();
    prepare(&mut prepared, &session);
    (session, prepared)
}

pub(super) fn same_as_full(prepared: &Prepared, session: &Session) {
    let mut full = Prepared::default();
    // Reference parsing is outside the measured consumer, including nested comparisons.
    work::measure(|| prepare(&mut full, session));
    assert!(
        format!("{:?}", prepared.body(ID)) == format!("{:?}", full.body(ID)),
        "streamed message must equal fresh full preparation"
    );
    let text = |prepared: &Prepared| {
        prepared
            .text()
            .entry(ID)
            .unwrap()
            .texts()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert!(
        text(prepared) == text(&full),
        "streamed selection must equal fresh full preparation"
    );
    let body = prepared.body(ID).unwrap();
    let parts = match &*body {
        Body::Message(parts) => parts.len(),
        _ => 1,
    };
    for part in 0..parts {
        let region = Region::Body(part);
        let cells: Vec<_> = prepared.text().cells(ID, region).collect();
        assert_eq!(cells, full.text().cells(ID, region).collect::<Vec<_>>());
        for cell in cells {
            let actual_key = prepared.text().key(ID, region, cell).unwrap();
            let expected_key = full.text().key(ID, region, cell).unwrap();
            let actual = prepared.text().segment(actual_key).unwrap();
            let expected = full.text().segment(expected_key).unwrap();
            assert_eq!(actual.atomic(), expected.atomic());
            for offset in 0..=actual.text().len() {
                assert_eq!(actual.source_offset(offset), expected.source_offset(offset));
            }
        }
    }
}

#[test]
fn literal_appends_do_not_reparse_the_prepared_markdown_prefix() {
    let mut session = session("# Heading\n\n**Streaming** ");
    let mut prepared = Prepared::default();
    let chunk = "plain paragraph ".repeat(8);
    let (_, counts) = work::measure(|| {
        for _ in 0..128 {
            apply(
                &mut session,
                Change::Append {
                    id: ID,
                    text: chunk.clone(),
                },
            );
            prepare(&mut prepared, &session);
            same_as_full(&prepared, &session);
        }
    });
    let bytes = session.transcript().entries[0].kind.text().unwrap().len();
    println!(
        "APPEND_WORK source_bytes={bytes} markdown_bytes={} protection_bytes={}",
        counts.markdown_bytes, counts.protection_bytes
    );
    assert!(
        counts.markdown_bytes <= bytes * 4,
        "literal append work must stay bounded by the emitted source: {} parser bytes for {bytes} source bytes",
        counts.markdown_bytes
    );
}
