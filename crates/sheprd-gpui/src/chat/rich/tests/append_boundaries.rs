// ABOUTME: Compares streaming reuse with full Markdown preparation at syntax boundaries.
// ABOUTME: Preserves limits, math, links, embedded content, and selectable coordinates.
use super::{append_work::*, *};
use sheprd_chat_protocol::Change;

#[test]
fn literal_appends_keep_styles_visuals_spaces_and_source_coordinates() {
    for prefix in [
        "plain tail",
        "# Heading\n\n**Bold** tail",
        "$x^2$ tail",
        "```rust\nlet x = 1;\n```\n\nplain tail",
        "first paragraph\n\nsecond tail",
        "- item tail",
        "1. item tail",
        "> quote tail",
        "```rust\nliteral tail",
        "    literal tail",
        "> ```rust\n> literal tail",
        "A | B\n--- | ---\nleft | tail",
    ] {
        let (mut session, mut prepared) = seeded(prefix);
        let old = prepared.body(ID).unwrap();
        let before = format!("{old:?}");
        for text in [
            " ",
            " ",
            "joined",
            "123",
            "  ",
            "words",
            " 終わり",
            " العربية",
            "",
        ] {
            apply(
                &mut session,
                Change::Append {
                    id: ID,
                    text: text.into(),
                },
            );
            let (_, counts) = work::measure(|| prepare(&mut prepared, &session));
            assert_eq!(
                counts.markdown_bytes, 0,
                "literal append must reuse prepared content: prefix={prefix:?} append={text:?} old={old:?}"
            );
            same_as_full(&prepared, &session);
            assert!(
                format!("{old:?}") == before,
                "prior immutable body must stay unchanged"
            );
        }
    }
}

#[test]
fn markdown_changes_use_full_preparation_and_can_change_earlier_content() {
    let cases: &[(&str, &[&str])] = &[
        ("tail", &[" **bold**", " text", " more"]),
        ("tail", &["\n\n# Heading", "\n\nnext", " words"]),
        ("Use &amp", &[";", " more"]),
        ("Use &not", &["in", "; ", "word"]),
        ("See http", &["://example.com", " more"]),
        ("See /tmp/fi", &["le.rs", " more"]),
        ("**bold**", &["tail", " more"]),
        ("partial `code", &["`", " tail"]),
        (
            "[word][id]\n\ntext",
            &["\n\n[id]: https://example.com", "\n\nmore"],
        ),
        ("$$x", &["$$", " prose"]),
        ("```rust\nlet x", &[" = 1;", "\n```", "\n\nnext", " words"]),
        ("intro", &["\n\n```html\n<div>Hello</div>\n```"]),
        ("![picture](https://example.com/p.png)", &[" text"]),
        ("first | second", &["\n--- | ---\nvalue | other"]),
        (
            "Unicode e",
            &[
                "\u{301}",
                "\u{a0}word",
                " 🦖",
                "\0",
                "\u{061c}",
                "\u{202e}",
                "\u{2066}",
            ],
        ),
    ];
    for &(prefix, additions) in cases {
        let (mut session, mut prepared) = seeded(prefix);
        for (index, &text) in additions.iter().enumerate() {
            apply(
                &mut session,
                Change::Append {
                    id: ID,
                    text: text.into(),
                },
            );
            let (_, counts) = work::measure(|| prepare(&mut prepared, &session));
            if index == 0 {
                assert!(
                    counts.markdown_bytes > 0,
                    "unsafe boundary must use the actual full parser: {prefix:?}"
                );
            }
            same_as_full(&prepared, &session);
        }
    }
}

#[test]
fn summary_transitions_and_message_limits_keep_the_full_preparation_contract() {
    let (mut session, mut prepared) =
        seeded("# Objective\nReview\n# Work State\nPending\n# Next Mov");
    apply(
        &mut session,
        Change::Append {
            id: ID,
            text: "e\nContinue".into(),
        },
    );
    prepare(&mut prepared, &session);
    assert!(matches!(
        &*prepared.body(ID).unwrap(),
        crate::chat::prepared::Body::Summary(_)
    ));
    same_as_full(&prepared, &session);

    let words = format!("{}tail", "word ".repeat(parse::MAX_WORDS - 1));
    let bytes = "é".repeat(crate::chat::prepared::MESSAGE.body / 2 - 2);
    let lines = "line\n\n".repeat(crate::chat::prepared::MESSAGE.lines);
    for prefix in [words, bytes, lines] {
        let (mut session, mut prepared) = seeded(&prefix);
        apply(
            &mut session,
            Change::Append {
                id: ID,
                text: " extra".into(),
            },
        );
        prepare(&mut prepared, &session);
        same_as_full(&prepared, &session);
    }
}

#[test]
fn appends_past_the_message_cut_do_not_reparse_the_unchanged_visible_prefix() {
    let limit = crate::chat::prepared::MESSAGE.body;
    let prefixes = [
        "x".repeat(limit + 16),
        format!("```rust\n{}", "x".repeat(limit + 16)),
        "line\n\n".repeat(crate::chat::prepared::MESSAGE.lines + 1),
    ];
    for prefix in prefixes {
        let (mut session, mut prepared) = seeded(&prefix);
        for text in [" more", "\n\n[earlier]: https://example.com", "\n```"] {
            apply(
                &mut session,
                Change::Append {
                    id: ID,
                    text: text.into(),
                },
            );
            let (_, counts) = work::measure(|| prepare(&mut prepared, &session));
            assert_eq!(
                counts.markdown_bytes, 0,
                "a cut message's visible source cannot change after an append"
            );
            same_as_full(&prepared, &session);
        }
    }
}

#[test]
fn raw_html_literal_tails_keep_the_current_parser_mapping() {
    for prefix in [
        "<div>\nplain tail",
        "<script>one tail",
        "<!-- comment\nplain tail",
        "<?instruction\nplain tail",
        "<!DOCTYPE\nplain tail",
    ] {
        let (mut session, mut prepared) = seeded(prefix);
        for text in [" more", "word", "  ", " joined"] {
            apply(
                &mut session,
                Change::Append {
                    id: ID,
                    text: text.into(),
                },
            );
            prepare(&mut prepared, &session);
            same_as_full(&prepared, &session);
        }
    }
}

#[test]
fn trailing_reference_definitions_do_not_own_the_previous_rendered_text_tail() {
    for prefix in [
        "foo\n\n[x]: foo",
        "foo [x] foo\n\n[x]: foo",
        "A | B\n--- | ---\nleft | foo\n\n[x]: foo",
        "A | B\n--- | ---\nleft | foo | foo",
        "```text\nfoo\n```\n\n[x]: foo",
    ] {
        let (mut session, mut prepared) = seeded(prefix);
        apply(
            &mut session,
            Change::Append {
                id: ID,
                text: "bar".into(),
            },
        );
        let (_, counts) = work::measure(|| prepare(&mut prepared, &session));
        same_as_full(&prepared, &session);
        assert!(
            counts.markdown_bytes > 0,
            "hidden source may change earlier reference resolution"
        );
    }
}
