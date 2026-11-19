// ABOUTME: Measures final Markdown parser work through real streaming preparation.
// ABOUTME: Checks complete generated output and nested measurement ownership.
use super::*;
use crate::chat::prepared::{Body, Part, Prepared};
use sheprd_chat_protocol::{Change, Entry, EntryId, EntryKind, Transcript};

#[test]
fn appended_transcript_reports_final_parser_work_without_losing_text() {
    let id = EntryId(1);
    let chunk = "plain paragraph ".repeat(8);
    let mut transcript = Transcript::new("Generated parser workload");
    transcript
        .apply(&Change::Push {
            entry: Entry {
                id,
                kind: EntryKind::Agent {
                    text: "**Streaming** ".into(),
                },
            },
        })
        .unwrap();
    let mut prepared = Prepared::default();
    let (_, counts) = work::measure(|| {
        for revision in 1..=128 {
            transcript
                .apply(&Change::Append {
                    id,
                    text: chunk.clone(),
                })
                .unwrap();
            prepared.update(&transcript, &[(id, revision)]);
        }
    });
    let body = prepared.body(id).unwrap();
    let Body::Message(parts) = &*body else {
        panic!("rich message")
    };
    let [Part::Rich(blocks)] = &parts[..] else {
        panic!("rich fragment")
    };
    let [Block::Paragraph { content, .. }] = &blocks[..] else {
        panic!("paragraph")
    };
    let text: String = content
        .iter()
        .map(|inline| match inline {
            Inline::Text(text, _) => text.as_str(),
            _ => panic!("generated prose stays text"),
        })
        .collect();
    let expected = format!("Streaming {}", chunk.repeat(128));
    assert!(
        text == expected.trim_end_matches(' '),
        "complete displayed prose must match the generated Markdown, excluding trailing source spaces"
    );
    assert_eq!(counts.protection_bytes, 0);
    assert!(
        counts.markdown_bytes > 0,
        "the real final parser must be observed"
    );
    println!(
        "STREAMING_WORK final_source_bytes={} markdown_bytes={} protection_bytes={}",
        transcript.entries[0].kind.text().unwrap().len(),
        counts.markdown_bytes,
        counts.protection_bytes
    );
}

#[test]
fn nested_parser_measurement_keeps_its_own_work() {
    let before = "**before**";
    let after = "**after**";
    let (_, outer) = work::measure(|| {
        assert!(parse(before, &mut Visuals::default()).is_some());
        let (_, inner) = work::measure(|| {
            assert!(parse("$x$", &mut Visuals::default()).is_some());
        });
        assert!(inner.protection_bytes > 0);
        assert!(inner.markdown_bytes > 0);
        assert!(parse(after, &mut Visuals::default()).is_some());
    });
    assert_eq!(outer.protection_bytes, 0);
    assert_eq!(outer.markdown_bytes, before.len() + after.len());
}
