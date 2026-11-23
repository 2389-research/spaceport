use super::*;
use pulldown_cmark::HeadingLevel;

#[test]
fn ordinary_markdown_keeps_hierarchy_lists_and_inline_semantics_without_visual_jobs() {
    let (blocks, wants) = parse_with(
        "# Main\n\nParagraph with **bold**, *emphasis*, `code`, and ~~old~~ text.\n\n## Details\n\n- Parent\n  - Nested\n\n3. Third\n4. Fourth\n\n> Quote\n\nLast paragraph.",
    );
    let blocks = blocks.unwrap();
    assert!(blocks.iter().all(|block| block.visuals().next().is_none()));
    assert!(wants.is_empty());
    let paragraphs: Vec<_> = blocks
        .iter()
        .filter_map(|block| match block {
            Block::Paragraph {
                kind,
                indent,
                content,
            } => Some((*kind, *indent, content)),
            _ => None,
        })
        .collect();
    assert_eq!(paragraphs[0].0, Kind::Heading(HeadingLevel::H1));
    assert_eq!(paragraphs[2].0, Kind::Heading(HeadingLevel::H2));
    assert_eq!((paragraphs[3].0, paragraphs[3].1), (Kind::Bullet, 0));
    assert_eq!((paragraphs[4].0, paragraphs[4].1), (Kind::Bullet, 1));
    assert!(
        paragraphs
            .iter()
            .any(|(kind, _, _)| *kind == Kind::Numbered(3))
    );
    assert!(
        paragraphs
            .iter()
            .any(|(kind, _, _)| *kind == Kind::Numbered(4))
    );
    assert!(paragraphs.iter().any(|(kind, _, _)| *kind == Kind::Quote));
    for (expected, field) in [("bold", 0), ("emphasis", 1), ("code", 2), ("old", 3)] {
        assert!(
            paragraphs[1]
                .2
                .iter()
                .any(|part| matches!(part, Inline::Text(text, style)
            if text == expected && [style.bold, style.italic, style.code, style.strike][field]))
        );
    }
}

#[test]
fn inline_images_preserve_their_source_without_loading_or_linking() {
    let source = "See ![**plot**](plots/a.png) and ![remote](https://example.org/a.png).";
    let blocks = parsed(source);
    assert!(blocks.iter().all(|block| block.visuals().next().is_none()));
    let Block::Paragraph { content, .. } = &blocks[0] else {
        panic!("paragraph")
    };
    let mut visible = String::new();
    for part in content {
        if let Inline::Text(text, style) = part {
            visible.push_str(text);
            assert!(style.link.is_none());
            assert!(!style.bold);
        }
    }
    assert_eq!(visible, source);
}
