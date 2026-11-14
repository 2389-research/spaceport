// ABOUTME: Checks citation identity and freshness against actual per-turn evidence.
// ABOUTME: Refuses invented, label-matched, and prior-turn navigation targets.
use super::answers;
use super::reading_text;
use sheprd_chat_protocol::{Entry, EntryId, EntryKind, Tool, ToolContent, Transcript};
use sheprd_control_protocol::protocol::observation::{ExcerptKind, Reply, Source, Target};

#[test]
fn answer_citations_resolve_only_real_evidence_from_the_same_turn() -> anyhow::Result<()> {
    let source = Source {
        target: Target("actual-token".into()),
        window: 1,
        host: "Device".into(),
        workspace: "Workspace".into(),
        label: "Session".into(),
    };
    let evidence = serde_json::to_string(&Reply::Excerpt {
        source: Box::new(source.clone()),
        kind: ExcerptKind::Conversation,
        text: "observed".into(),
        offset: 0,
        next_offset: None,
        truncated: false,
        observed_at_ms: 123,
        snapshot_revision: 1,
    })?;
    let mut transcript = Transcript::new("Pi");
    transcript.entries = vec![
        Entry { id: EntryId(1), kind: EntryKind::User { text: "Question".into(), files: 0 } },
        Entry { id: EntryId(2), kind: EntryKind::Tool(Tool { call_id: "read".into(), content: vec![ToolContent::Text { text: evidence }], kind: Default::default(), locations: Vec::new(), parent: None, started_ms: None, status: Default::default(), title: "Read conversation".into() }) },
        Entry { id: EntryId(3), kind: EntryKind::Agent { text: "[Session](spaceport-source:actual-token) [Invented](spaceport-source:invented-token)".into() } },
        Entry { id: EntryId(4), kind: EntryKind::User { text: "Another question".into(), files: 0 } },
        Entry { id: EntryId(5), kind: EntryKind::Agent { text: "[Old](spaceport-source:actual-token)".into() } },
    ];
    let citations = answers(&transcript);
    assert_eq!(citations.get(&EntryId(3)), Some(&vec![(source, 123)]));
    assert!(!citations.contains_key(&EntryId(5)));
    Ok(())
}

#[test]
fn source_links_read_as_labels_without_exposing_opaque_identifiers() {
    assert_eq!(
        reading_text(
            "Review [Parser](spaceport-source:exact-token), then [Build](spaceport-source:other-token)."
        ),
        "Review Parser, then Build."
    );
    assert_eq!(
        reading_text(
            "Keep [web](https://example.com) and unfinished [text](spaceport-source:token"
        ),
        "Keep [web](https://example.com) and unfinished [text](spaceport-source:token"
    );
}
