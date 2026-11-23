//! What a message asks the app to draw, and the bounds past which it asks for
//! nothing: the counters and caps are the parse's own, so only it can report
//! which fences and formulas were admitted.

use super::*;
use crate::rich::parse::MAX_HIGHLIGHTED;

#[test]
fn a_fence_past_the_highlight_budget_asks_for_nothing_but_keeps_its_identity() {
    let text: String = (0..=MAX_HIGHLIGHTED)
        .map(|n| format!("```rust\nlet x{n} = 1;\n```\n\n"))
        .collect();
    let (blocks, wants) = parse_with(&text);
    let blocks = blocks.unwrap();
    assert_eq!(blocks.len(), MAX_HIGHLIGHTED + 1, "{text}");
    assert_eq!(wants.len(), MAX_HIGHLIGHTED);
    let id = match &blocks[MAX_HIGHLIGHTED] {
        Block::Code { id, .. } => id,
        _ => panic!("code"),
    };
    assert!(!wants.iter().any(|want| want.id() == *id));
}

#[test]
fn a_formula_past_the_source_bound_asks_for_nothing_and_stays_source() {
    let source = "x".repeat(MAX_MATH_BYTES + 1);
    let (blocks, wants) = parse_with(&format!("Before ${source}$ after"));
    assert!(wants.is_empty());
    let Block::Paragraph { content, .. } = &blocks.unwrap()[0] else {
        panic!("paragraph")
    };
    assert!(
        content.iter().any(
            |part| matches!(part, Inline::Text(text, _) if text.contains(&format!("${source}$")))
        ),
        "an unadmitted formula stays visible source"
    );
}
