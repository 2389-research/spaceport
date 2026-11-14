// ABOUTME: Verifies standalone transcript revisions preserve reading and selection boundaries.
// ABOUTME: Uses isolated headless entities without helpers or connected sessions.
use super::View;
use crate::chat::prepared::{Body, Part, Prepared};
use gpui::{AppContext, ListOffset, TestAppContext, px};
use sheprd_chat_protocol::{Entry, EntryId, EntryKind, Status, Transcript};
use std::rc::Rc;
mod autoscroll;
mod colors;
mod composer;

#[test]
fn reading_cache_retains_unchanged_entries_and_updates_changed_revisions() {
    let mut transcript = Transcript {
        status: Status::Idle,
        entries: vec![Entry {
            id: EntryId(1),
            kind: EntryKind::Agent {
                text: "Original".into(),
            },
        }],
        ..Transcript::new("Pi")
    };
    let mut prepared = Prepared::default();
    prepared.update_reading_lines(&transcript, &[(EntryId(1), 1)]);
    let original = prepared.body(EntryId(1));
    prepared.update_reading_lines(&transcript, &[(EntryId(1), 1)]);
    let unchanged = prepared.body(EntryId(1));
    assert!(
        original
            .as_ref()
            .zip(unchanged.as_ref())
            .is_some_and(|(old, new)| Rc::ptr_eq(old, new))
    );
    transcript.entries[0].kind = EntryKind::Agent {
        text: "Changed".into(),
    };
    prepared.update_reading_lines(&transcript, &[(EntryId(1), 2)]);
    let updated = prepared.body(EntryId(1));
    assert!(
        matches!(updated.as_deref(), Some(Body::Message(parts)) if matches!(parts.first(), Some(Part::Text(lines)) if lines.first().is_some_and(|line|line.text=="Changed")))
    );
}

#[gpui::test]
fn refreshed_transcript_preserves_the_exact_entry_being_read(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let view = cx.new(|cx| View::new(Default::default(), Default::default(), cx));
        view.update(cx, |view, cx| {
            let mut transcript = Transcript::new("Pi");
            transcript.status = Status::Idle;
            transcript.entries = (1..=6)
                .map(|id| Entry {
                    id: EntryId(id),
                    kind: EntryKind::Agent {
                        text: format!("Answer {id}"),
                    },
                })
                .collect();
            view.owner
                .update(cx, |owner, _| owner.set_transcript(transcript.clone()));
            view.prepare(cx);
            view.list.scroll_to(ListOffset {
                item_ix: 2,
                offset_in_item: px(9.),
            });
            view.glide.user_scrolled();
            transcript.entries.push(Entry {
                id: EntryId(7),
                kind: EntryKind::Agent {
                    text: "Later answer".into(),
                },
            });
            view.owner
                .update(cx, |owner, _| owner.set_transcript(transcript));
            view.prepare(cx);
            let top = view.list.logical_scroll_top();
            assert_eq!(view.list_keys[top.item_ix].0, EntryId(3));
            assert_eq!(top.offset_in_item, px(9.));
        });
    });
}
