use super::*;

#[test]
fn code_preserves_tabs_in_copy_and_keeps_inline_words_separated() {
    let source = "\tlet α = 2;\n\t\tprintln!(\"{α}\");\n";
    for closing in ["", "```"] {
        let blocks = parsed(&format!("```rust\n{source}{closing}"));
        assert!(matches!(&blocks[0], Block::Code { source: code, .. } if &**code == source));
    }
    let blocks = parsed("Use `left\tright` together.");
    let Block::Paragraph { content, .. } = &blocks[0] else {
        panic!("paragraph")
    };
    assert!(content.iter().any(
        |part| matches!(part, Inline::Text(text, style) if style.code && text == "left    right")
    ));
}

#[test]
fn a_closed_fence_inside_a_quote_is_highlighted() {
    for text in [
        "> ```rust\n> fn main() {}\n> ```",
        "> > ```rust\n> > fn main() {}\n> > ```",
        "> - ```rust\n>   fn main() {}\n>   ```",
    ] {
        let (blocks, wants) = parse_with(text);
        let id = blocks.into_iter().flatten().find_map(|block| match block {
            Block::Code { id, .. } => Some(id),
            _ => None,
        });
        assert!(
            id.is_some_and(|id| wants.iter().any(|want| want.id() == id)),
            "{text}"
        );
    }
    let (open, wants) = parse_with("> ```rust\n> fn main() {}");
    let Some(Block::Code { id, .. }) = open.and_then(|blocks| blocks.into_iter().next()) else {
        panic!("code")
    };
    assert!(!wants.iter().any(|want| want.id() == id));
}
