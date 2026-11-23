use super::*;

/// A code block or a display formula draws itself, and a paragraph or table
/// draws the inline formulas in its text: nothing else counts as a visual.
#[test]
fn a_block_reports_the_visuals_it_draws() {
    let text = |text: &str| Inline::Text(text.to_owned(), Style::default());
    let math = |source: &str| Inline::Math {
        id: VisualId::of(("math", crate::visual::MathMode::Inline, source)),
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
            id: VisualId::of(("math", crate::visual::MathMode::Display, &source)),
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
        VisualId::of(("math", crate::visual::MathMode::Display, &source)),
        VisualId::of(("math", crate::visual::MathMode::Inline, "a")),
        VisualId::of(("math", crate::visual::MathMode::Inline, "b")),
    ];
    assert_eq!(drawn, expected);
}
