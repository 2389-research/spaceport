use super::*;
use crate::rich::parse::normalize_math;
use pulldown_cmark::Alignment;

#[test]
fn inline_and_display_math_keep_their_place_and_identity_while_streaming() {
    let blocks = parsed("Before $x^2$ after.\n\n$$\\frac{a}{b}$$\n\nMore");
    let Block::Paragraph { content, .. } = &blocks[0] else {
        panic!("paragraph")
    };
    assert!(
        matches!(&content[..], [Inline::Text(a, _), Inline::Math { .. }, Inline::Text(b, _)] if a == "Before " && b == " after.")
    );
    assert!(matches!(&blocks[1], Block::Math { source, .. } if &**source == r"\frac{a}{b}"));
    let first = blocks[0].visuals().next().unwrap();
    let extended = parsed("Before $x^2$ after. More streamed prose");
    assert_eq!(first, extended[0].visuals().next().unwrap());
    let unfinished = parsed("Before $x^");
    assert!(
        matches!(&unfinished[0], Block::Paragraph { content, .. }
        if matches!(&content[..], [Inline::Text(text, _)] if text == "Before $x^")),
        "unfinished math stays literal"
    );
}

#[test]
fn alternate_delimiters_are_math_but_code_is_never_rewritten() {
    let blocks = parsed(
        "Use \\(x_1\\) then \\[x^2\\]. `\\(literal\\)`\n\n```rust\nlet latex = \"$x$\";\n```",
    );
    assert!(
        blocks
            .iter()
            .any(|block| matches!(block, Block::Math { .. }))
    );
    let code = blocks
        .iter()
        .find_map(|block| match block {
            Block::Code { source, .. } => Some(source),
            _ => None,
        })
        .unwrap();
    assert_eq!(&**code, "let latex = \"$x$\";\n");
    assert!(blocks.iter().filter_map(|block| match block { Block::Paragraph { content, .. } => Some(content), _ => None })
        .flatten().any(|part| matches!(part, Inline::Text(text, style) if style.code && text == r"\(literal\)")));
}

#[test]
fn a_table_has_cells_alignment_and_inline_content() {
    let blocks = parsed("| Name | Value |\n| :--- | ---: |\n| **α** | $x^2$ |");
    let Block::Table { align, rows } = &blocks[0] else {
        panic!("table")
    };
    assert_eq!(align, &[Alignment::Left, Alignment::Right]);
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row.len() == 2));
    assert!(matches!(&rows[1][0][0], Inline::Text(text, style) if text == "α" && style.bold));
    assert!(matches!(&rows[1][1][0], Inline::Math { .. }));
}

#[test]
fn excessive_word_and_table_layouts_use_source_fallback() {
    assert!(
        parse_with(&format!("$x$ {}", "word ".repeat(4_097)))
            .0
            .is_none()
    );
    let table = format!("| h |\n|---|\n{}", "| value |\n".repeat(129));
    assert!(parse_with(&table).0.is_none());
}

#[test]
fn a_message_full_of_unmatched_math_openers_stays_literal_with_bounded_work() {
    let text = r"\(\[".repeat(65_536);
    let started = std::time::Instant::now();
    assert_eq!(normalize_math(&text).text, text);
    // A generous regression budget, not a rendering-performance benchmark.
    // The former repeated suffix search scans tens of GiB for this 256 KiB input.
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
    let text = format!("{} `\\)` \\(α\\)", r"\[".repeat(32_768));
    let normalized = normalize_math(&text).text;
    assert!(normalized.ends_with("`\\)` $α$"));
    assert_eq!(normalize_math(r"\(x\\)y\)").text, r"$x\\)y$");
}

#[test]
fn long_prose_around_math_keeps_its_suffix() {
    let prose = format!("{}終わり", "α".repeat(17_000));
    let blocks = parsed(&format!("{prose} $x$ suffix"));
    let Block::Paragraph { content, .. } = &blocks[0] else {
        panic!("paragraph")
    };
    assert!(matches!(&content[0], Inline::Text(text, _) if text == &format!("{prose} ")));
    assert!(matches!(content.last(), Some(Inline::Text(text, _)) if text == " suffix"));
}

#[test]
fn spaced_inline_delimiters_still_become_math() {
    assert_eq!(
        normalize_math(r"Area \( \pi r^2 \).").text,
        r"Area $\pi r^2$."
    );
    assert_eq!(
        normalize_math(r"Empty \(  \) stays").text,
        r"Empty \(  \) stays"
    );
    let blocks = parsed(r"The area is \( \pi r^2 \).");
    let Block::Paragraph { content, .. } = &blocks[0] else {
        panic!("paragraph")
    };
    assert!(
        content
            .iter()
            .any(|part| matches!(part, Inline::Math { .. }))
    );
}
