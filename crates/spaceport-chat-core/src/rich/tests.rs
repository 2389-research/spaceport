#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

mod blocks;
mod code;
mod display_math;
mod prose;
mod wants;

/// The classification the app hands [`parse`]: a web address, or a local path.
/// Which paths are links is the terminal's rule and is tested where it lives,
/// so these tests act on a web address and leave every path as visible source.
fn destination(url: &str) -> Option<String> {
    spaceport_web_url::WebUrl::try_from(url)
        .ok()
        .map(|url| url.as_str().to_owned())
}

/// The blocks `text` parses into, and the visuals it asks the app to draw.
fn parse_with(text: &str) -> (Option<Vec<Block>>, Vec<Want>) {
    let mut wants = Vec::new();
    let blocks = parse(text, &mut wants, destination);
    (blocks, wants)
}

/// `text`'s blocks, for a test whose fragment stays rich.
fn parsed(text: &str) -> Vec<Block> {
    parse_with(text).0.unwrap()
}

/// A code block or a display formula draws itself, and a paragraph or table
/// draws the inline formulas in its text: nothing else counts as a visual.
#[test]
fn a_block_reports_the_visuals_it_draws() {
    let text = |text: &str| Inline::Text(text.to_owned(), Style::default());
    let math = |source: &str| Inline::Math {
        id: VisualId::of(("math", MathMode::Inline, source)),
        source: source.into(),
    };
    let paragraph = |content: Vec<Inline>| Block::Paragraph {
        kind: Kind::Paragraph,
        indent: 0,
        content,
    };
    let source: Arc<str> = "x".into();
    let blocks = [
        Block::Code {
            id: VisualId::of(("code", "", &source)),
            language: String::new(),
            source: source.clone(),
        },
        Block::Math {
            id: VisualId::of(("math", MathMode::Display, &source)),
            source: source.clone(),
        },
        paragraph(vec![text("before "), math("a"), text(" after")]),
        paragraph(vec![Inline::Break, text("no formula here")]),
        Block::Table {
            align: Vec::new(),
            rows: vec![vec![vec![math("b")]]],
        },
    ];
    let drawn: Vec<VisualId> = blocks.iter().flat_map(Block::visuals).collect();
    let expected = [
        VisualId::of(("code", "", &source)),
        VisualId::of(("math", MathMode::Display, &source)),
        VisualId::of(("math", MathMode::Inline, "a")),
        VisualId::of(("math", MathMode::Inline, "b")),
    ];
    assert_eq!(drawn, expected);
}
