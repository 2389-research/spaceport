use super::*;
use spaceport_chat_protocol::{Entry, EntryKind};

fn diagram(id: u64, source: &str) -> Entry {
    Entry {
        id: EntryId(id),
        kind: EntryKind::Agent {
            text: format!("```mermaid\n{source}```\n"),
        },
    }
}

/// The picture the entry `id` draws.
fn visual_of(prepared: &Prepared, id: EntryId) -> VisualId {
    let body = prepared.body(id).expect("a prepared body");
    let Body::Message(parts) = body.as_ref() else {
        panic!("not a message");
    };
    parts
        .iter()
        .find_map(|part| match part {
            Part::Diagram(visual, _) => Some(*visual),
            _ => None,
        })
        .expect("one diagram")
}

/// The shell hands every picture the model asks for to its [`Visuals`] and
/// keeps those the bodies still draw, so a pass that replaces one entry's
/// picture leaves the other entry's in place.
#[test]
fn a_partial_update_keeps_the_pictures_the_other_entries_draw() {
    let mut transcript = Transcript::new("Claude");
    transcript.entries = vec![
        diagram(1, "flowchart LR\n  A --> B\n"),
        diagram(2, "flowchart LR\n  C --> D\n"),
    ];
    let mut prepared = Prepared::default();
    prepared.update(
        &transcript,
        &[(EntryId(1), 1), (EntryId(2), 1)],
        &crate::chat::labels::now(),
    );
    let kept = visual_of(&prepared, EntryId(1));
    let replaced = visual_of(&prepared, EntryId(2));
    assert!(prepared.visuals.wants(kept));
    assert!(prepared.visuals.wants(replaced));

    transcript.entries[1] = diagram(2, "flowchart LR\n  C --> E\n");
    prepared.update(
        &transcript,
        &[(EntryId(1), 1), (EntryId(2), 2)],
        &crate::chat::labels::now(),
    );

    let current = visual_of(&prepared, EntryId(2));
    assert_ne!(current, replaced, "the fence changed");
    assert!(
        prepared.visuals.wants(kept),
        "an entry whose revision did not move keeps its picture"
    );
    assert!(prepared.visuals.wants(current));
    assert!(
        !prepared.visuals.wants(replaced),
        "no entry draws the replaced picture any more"
    );
}
