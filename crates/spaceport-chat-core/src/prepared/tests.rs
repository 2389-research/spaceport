use super::*;
use crate::labels::Labels;
use spaceport_chat_protocol::{Entry, Image, Tool, ToolKind, ToolStatus};

/// The words this model writes into an entry. A crate below the app has no
/// messages, so a test names them itself.
fn labels() -> Labels {
    Labels {
        answer_cut: "Copy takes all of it".into(),
        diff_after: "After".into(),
        diff_before: "Before".into(),
        diff_new_file: "New file".into(),
        question_answer: "Answer".into(),
        question_number: |number| format!("Question {number}"),
        question_truncated: "More answers were left out".into(),
        terminal: |id| format!("Terminal {id}"),
        tool_ask_user: "Questions".into(),
    }
}

/// No link in these tests names a destination the window may act on.
fn no_destination(_: &str) -> Option<String> {
    None
}

fn agent(id: u64, text: &str) -> Entry {
    Entry {
        id: EntryId(id),
        kind: EntryKind::Agent { text: text.into() },
    }
}

/// The parts of an agent's message, as a test reads them.
fn message_parts(text: &str) -> Vec<Part> {
    message(
        text,
        Author::Agent,
        &mut Vec::new(),
        &labels(),
        no_destination,
    )
}

#[test]
fn diffs_count_lines_added_and_removed() {
    assert_eq!(diff_counts("a\nb\nc\n", "a\nB\nc\nd\n"), (2, 1));
    assert_eq!(diff_counts("", "one\ntwo"), (2, 0));
    assert_eq!(diff_counts("same", "same"), (0, 0));
}

#[test]
fn a_tool_call_is_summarized() {
    let text = (0..10)
        .map(|n| format!("line {n}"))
        .collect::<Vec<_>>()
        .join("\n");
    let summary = summarize(
        &[
            ToolContent::Text { text },
            ToolContent::Diff {
                new: "x\ny\n".into(),
                old: None,
                path: "/src/new.rs".into(),
            },
            ToolContent::Terminal { id: "t1".into() },
        ],
        &mut Vec::new(),
    );
    assert_eq!(summary.lines.len(), 6);
    assert_eq!(summary.more, 4);
    assert_eq!(
        summary.diffs,
        vec![DiffSummary {
            path: "/src/new.rs".into(),
            added: 2,
            removed: 0
        }]
    );
    assert_eq!(summary.terminals, 1);
}

#[test]
fn an_entrys_pictures_are_asked_for_once_each_in_order() {
    let image = Image::new("image/png", "aGk=".into()).unwrap();
    let mut transcript = Transcript::new("Claude");
    transcript.entries = vec![
        agent(
            1,
            "before\n\n```mermaid\nflowchart LR\n  A --> B\n```\n\nafter\n",
        ),
        Entry {
            id: EntryId(2),
            kind: EntryKind::Image {
                author: Author::Agent,
                image,
            },
        },
    ];
    let keys = vec![(EntryId(1), 1), (EntryId(2), 1)];
    let mut prepared = Prepared::default();
    prepared.update(&transcript, &keys, &labels(), no_destination);
    let asked: Vec<_> = prepared
        .take_wants()
        .into_iter()
        .map(|want| want.id())
        .collect();
    let Body::Message(parts) = &*prepared.body(EntryId(1)).unwrap() else {
        panic!("not a message");
    };
    let diagram = parts
        .iter()
        .find_map(|part| match part {
            Part::Diagram(id, _) => Some(*id),
            _ => None,
        })
        .unwrap();
    let Body::Image(image) = &*prepared.body(EntryId(2)).unwrap() else {
        panic!("not an image");
    };
    assert_eq!(asked, vec![diagram, *image]);
    assert_eq!(
        prepared.drawn(),
        asked.iter().copied().collect::<HashSet<_>>()
    );
    // A pass that prepares nothing asks for nothing.
    prepared.update(&transcript, &keys, &labels(), no_destination);
    assert!(prepared.take_wants().is_empty());
}

#[test]
fn bodies_are_prepared_once_per_revision() {
    let mut transcript = Transcript::new("Claude");
    transcript.entries = vec![
        agent(1, "**bold** text"),
        Entry {
            id: EntryId(2),
            kind: EntryKind::Tool(Tool {
                call_id: "c".into(),
                content: Vec::new(),
                kind: ToolKind::Read,
                locations: Vec::new(),
                parent: None,
                started_ms: None,
                status: ToolStatus::Completed,
                title: "Read".into(),
            }),
        },
        Entry {
            id: EntryId(3),
            kind: EntryKind::User {
                text: "hi".into(),
                files: 0,
            },
        },
    ];
    let keys = vec![(EntryId(1), 1), (EntryId(2), 1), (EntryId(3), 1)];
    let mut prepared = Prepared::default();
    prepared.update(&transcript, &keys, &labels(), no_destination);
    let first = prepared.body(EntryId(1)).unwrap();
    assert!(matches!(&*first, Body::Message(parts)
        if matches!(&parts[..], [Part::Rich(blocks)]
            if matches!(&blocks[0], crate::rich::Block::Paragraph { content, .. }
                if matches!(&content[0], crate::rich::Inline::Text(text, style) if text == "bold" && style.bold)))));
    assert!(matches!(
        &*prepared.body(EntryId(2)).unwrap(),
        Body::Tool(_)
    ));
    assert!(matches!(
        &*prepared.body(EntryId(3)).unwrap(),
        Body::Message(_)
    ));
    prepared.update(&transcript, &keys, &labels(), no_destination);
    assert!(Rc::ptr_eq(&first, &prepared.body(EntryId(1)).unwrap()));
    transcript.entries.remove(0);
    prepared.update(&transcript, &keys[1..], &labels(), no_destination);
    assert!(prepared.body(EntryId(1)).is_none());
}

#[test]
fn an_answer_too_long_to_show_ends_with_a_note() {
    let text = (0..3_000)
        .map(|n| format!("line {n}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut transcript = Transcript::new("Claude");
    transcript.entries = vec![agent(1, &text)];
    let mut prepared = Prepared::default();
    prepared.update(&transcript, &[(EntryId(1), 1)], &labels(), no_destination);
    let Body::Message(parts) = &*prepared.body(EntryId(1)).unwrap() else {
        panic!("not a message");
    };
    let Part::Text(lines) = parts.last().unwrap() else {
        panic!("not text");
    };
    let Part::Rich(blocks) = &parts[0] else {
        panic!("not structured prose");
    };
    assert!(
        matches!(&blocks[0], crate::rich::Block::Paragraph { content, .. }
        if content.iter().any(|part| matches!(part, crate::rich::Inline::Text(text, _)
            if text.contains("line 1999")))),
        "chat shows more than release notes do"
    );
    assert!(lines.last().unwrap().text.contains(&labels().answer_cut));
    // A short answer has no note.
    transcript.entries = vec![agent(2, "short")];
    prepared.update(&transcript, &[(EntryId(2), 1)], &labels(), no_destination);
    let Body::Message(parts) = &*prepared.body(EntryId(2)).unwrap() else {
        panic!("not a message");
    };
    assert!(matches!(&parts[..], [Part::Rich(blocks)] if blocks.len() == 1));
}

#[test]
fn an_answer_cut_twice_ends_with_one_note() {
    // Over the line, word, and per-line bounds at once: the plain fallback
    // notes its long line, and the body clip must not add a second note.
    let text = format!("{}\n{}", "x".repeat(5_000), "a b c\n".repeat(2_100));
    let parts = message_parts(&text);
    let Some(Part::Text(lines)) = parts.last() else {
        panic!("plain lines");
    };
    let note = lines.last().unwrap();
    assert_eq!(lines.iter().filter(|line| *line == note).count(), 1);
    assert!(note.text.contains(&labels().answer_cut));
}

const CARD: &str = r#"{"components":[{"type":"text","text":"Example"}]}"#;

#[test]
fn literal_indented_widget_examples_cannot_become_controls() {
    let json = r#"{"components":[{"type":"text","text":"Example"}]}"#;
    for text in [
        format!("    ```herdr-ui\n    {json}\n    ```"),
        format!("\t```herdr-ui\n\t{json}\n\t```"),
        format!("```example`herdr-ui\n{json}\n```"),
    ] {
        assert!(
            !message_parts(&text)
                .iter()
                .any(|part| matches!(part, Part::Widget(_)))
        );
    }
}

#[test]
fn indented_closers_cannot_activate_unfinished_widgets() {
    for marker in ["```", "~~~"] {
        for indentation in ["    ", "\t", "   \t"] {
            let text = format!("{marker}herdr-ui\n{CARD}\n{indentation}{marker}\n");
            assert!(
                !message_parts(&text)
                    .iter()
                    .any(|part| matches!(part, Part::Widget(_))),
                "{text:?}"
            );
        }
    }
}

#[test]
fn indented_outer_closers_keep_nested_widgets_literal() {
    for marker in ["````", "~~~~"] {
        for indentation in ["    ", "\t"] {
            let text = format!(
                "{marker}markdown\n{indentation}{marker}\n```herdr-ui\n{CARD}\n```\n{marker}\n"
            );
            assert!(
                !message_parts(&text)
                    .iter()
                    .any(|part| matches!(part, Part::Widget(_))),
                "{text:?}"
            );
        }
    }
}

#[test]
fn widget_closers_allow_up_to_three_spaces_and_longer_matching_runs() {
    for marker in ["```", "~~~"] {
        for indentation in ["", " ", "  ", "   "] {
            let text = format!(
                "{marker}herdr-ui\r\n{CARD}\r\n{indentation}{marker}{} \t\r\n",
                &marker[..1]
            );
            let parts = message_parts(&text);
            assert!(matches!(&parts[..], [Part::Widget(_)]), "{text:?}");
        }
    }
}
