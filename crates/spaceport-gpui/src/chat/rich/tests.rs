#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::chat::visual::{Error, Visuals};

mod display_math;

#[test]
fn code_is_copied_verbatim_and_only_complete_fences_queue_grammars() {
    let mut visuals = Visuals::default();
    let open = parse("```rust\nfn main() {\n    let α = 2;", &mut visuals).unwrap();
    let Block::Code { id, source, .. } = &open[0] else {
        panic!("code")
    };
    assert_eq!(&**source, "fn main() {\n    let α = 2;");
    assert!(!visuals.wants(*id));
    let closed = parse("```rust\nfn main() {\n    let α = 2;\n}\n```", &mut visuals).unwrap();
    let Block::Code {
        id,
        source,
        language,
    } = &closed[0]
    else {
        panic!("code")
    };
    assert!(visuals.wants(*id));
    let spans = highlight(language, source);
    assert!(
        spans
            .iter()
            .flatten()
            .any(|span| span.token == crate::chat::highlight::Token::Keyword)
    );
    for (line, spans) in source.lines().zip(spans) {
        assert!(
            spans
                .iter()
                .all(|span| line.is_char_boundary(span.start) && line.is_char_boundary(span.end))
        );
    }
}

#[test]
fn math_limits_and_errors_preserve_a_readable_source_path() {
    let nested_groups = format!("{}x{}", "{".repeat(65), "}".repeat(65));
    let nested_fractions = format!("{}x{}", r"\frac{1}{".repeat(65), "}".repeat(65));
    let nested_roots = format!("{}x", r"\sqrt".repeat(65));
    for source in [nested_groups, nested_fractions, nested_roots] {
        assert!(matches!(
            draw_math(&source, MathMode::Inline, 16., 0xffffff),
            Err(Error::MathLimit)
        ));
    }
    assert!(matches!(
        draw_math(
            &"x".repeat(MAX_MATH_BYTES + 1),
            MathMode::Inline,
            16.,
            0xffffff
        ),
        Err(Error::MathLimit)
    ));
    assert!(matches!(
        draw_math(r"\notARealCommand{", MathMode::Inline, 16., 0xffffff),
        Err(Error::MathParse(_))
    ));
    assert!(matches!(
        draw_math(
            r"\rule{100000em}{100000em}",
            MathMode::Inline,
            16.,
            0xffffff
        ),
        Err(Error::MathLimit) | Err(Error::Math(_))
    ));
    let inline = draw_math(r"\frac{a}{b}", MathMode::Inline, 16., 0xaabbcc).unwrap();
    let display = draw_math(r"\frac{a}{b}", MathMode::Display, 16., 0xaabbcc).unwrap();
    assert!(
        display.height > inline.height,
        "display fractions use display style"
    );
    assert!(
        inline
            .baseline
            .is_some_and(|baseline| baseline > 0. && baseline < inline.height)
    );
    let pixels = inline.texture.as_bytes(0).unwrap();
    assert!(pixels.as_chunks::<4>().0.iter().any(|p| p[3] == 0));
    assert!(
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[3] > 0 && p[..3] == [0xcc, 0xbb, 0xaa])
    );
}

#[test]
fn rich_prose_links_explicit_files_but_never_code_or_bare_file_urls() {
    let text = "See [docs](https://herdr.dev/a?b=1) or https://example.com/x). \
                [local](/tmp/file.rs:12) file:///tmp/file.rs \
                `https://in.code` and $x$.";
    let blocks = parse(text, &mut Visuals::default()).unwrap();
    let urls: Vec<_> = blocks
        .iter()
        .filter_map(|block| match block {
            Block::Paragraph { content, .. } => Some(content),
            _ => None,
        })
        .flatten()
        .filter_map(|part| match part {
            Inline::Text(_, style) => style.link.as_deref(),
            _ => None,
        })
        .collect();
    assert_eq!(
        urls,
        [
            "https://herdr.dev/a?b=1",
            "https://example.com/x",
            "/tmp/file.rs:12"
        ]
    );
    let plain = parse("See https://example.com/x", &mut Visuals::default()).unwrap();
    assert!(matches!(&plain[0], Block::Paragraph { content, .. }
        if content.iter().any(|part| matches!(part, Inline::Text(_, style)
            if style.link.as_deref() == Some("https://example.com/x")))));
}

#[test]
fn prose_links_accept_web_and_explicit_file_targets_but_leave_unsafe_schemes_literal() {
    let blocks = parse(
        "[docs](https://example.org/docs) https://example.org/raw. `https://example.org/code` [bad](javascript:alert) [file](/tmp/x)",
        &mut Visuals::default(),
    ).unwrap();
    let Block::Paragraph { content, .. } = &blocks[0] else {
        panic!("paragraph")
    };
    let links: Vec<_> = content
        .iter()
        .filter_map(|part| match part {
            Inline::Text(_, style) => style.link.as_deref(),
            _ => None,
        })
        .collect();
    assert_eq!(
        links,
        [
            "https://example.org/docs",
            "https://example.org/raw",
            "/tmp/x"
        ]
    );
    let visible = content
        .iter()
        .filter_map(|part| match part {
            Inline::Text(text, _) => Some(text.as_str()),
            _ => None,
        })
        .collect::<String>();
    assert!(visible.contains("file"));
    assert!(!visible.contains("[file](/tmp/x)"));
    assert!(visible.contains("[bad](javascript:alert)"));
}
