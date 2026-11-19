// ABOUTME: Checks redundant parse work through actual rich and transcript consumers.
// ABOUTME: Measures synthetic streaming preparation in release mode without a desktop.
use super::*;
use crate::chat::prepared::{Body, Part, Prepared};
use sheprd_chat_protocol::{Change, EntryId};
use std::{hint::black_box, time::Instant};

#[test]
fn ordinary_markdown_does_not_enter_the_math_protection_parser() {
    for text in ["**bold** text", "```rust\nlet x = 1;\n```"] {
        let (blocks, counts) = work::measure(|| parse(text, &mut Visuals::default()));
        assert!(
            blocks.is_some(),
            "the real rich consumer accepts the fixture"
        );
        assert_eq!(
            counts.protection_bytes, 0,
            "ordinary Markdown needs no math protection parse"
        );
        assert_eq!(
            counts.markdown_bytes,
            text.len(),
            "measure the actual final parser"
        );
    }
}

#[test]
#[ignore = "release-only synthetic streaming CPU benchmark; run explicitly with --release --ignored"]
fn synthetic_stream_preparation_cost() {
    assert!(
        !black_box(cfg!(debug_assertions)),
        "run this benchmark with --release"
    );
    let chunk = "plain paragraph ".repeat(8);
    assert_eq!(chunk.len(), 128);
    for (case, prefix) in [
        ("plain", ""),
        ("math", "$x^2$ "),
        ("code", "```text\n"),
        ("rich", "# Heading\n\n**Streaming** "),
    ] {
        for (mode, reuse) in [("full", false), ("append", true)] {
            let mut samples = Vec::new();
            let mut counts = work::Counts::default();
            for round in 0..11 {
                let id = EntryId(1);
                let mut session = append_work::session(prefix);
                let mut prepared = Prepared::default();
                let (elapsed, bytes) = work::measure(|| {
                    let started = Instant::now();
                    for _ in 0..128 {
                        append_work::apply(
                            &mut session,
                            Change::Append {
                                id,
                                text: chunk.clone(),
                            },
                        );
                        prepared.update(session.transcript(), &session.keys(), |id, revision| {
                            reuse && session.appended_since(id, revision)
                        });
                        black_box(prepared.body(id));
                    }
                    started.elapsed().as_nanos()
                });
                assert!(matches!(&*prepared.body(id).unwrap(), Body::Message(parts)
                if matches!(&parts[..], [Part::Rich(_)])));
                if round >= 2 {
                    samples.push(elapsed);
                    counts = bytes;
                }
            }
            samples.sort_unstable();
            let result = serde_json::json!({
                "case": case,
                "mode": mode,
                "chunks": 128,
                "chunk_bytes": chunk.len(),
                "median_ns": samples[samples.len() / 2],
                "protection_bytes": counts.protection_bytes,
                "markdown_bytes": counts.markdown_bytes,
                "samples_ns": samples,
            });
            println!("{result}");
        }
    }
}
