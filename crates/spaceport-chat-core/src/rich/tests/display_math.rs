use super::*;
use crate::rich::{
    math_input::{protect_display, protected_ranges},
    parse::normalize_math,
};

#[test]
fn multiline_display_math_precedes_markdown_and_preserves_the_exact_tex_source() {
    let source = "\n\\hat x_{\\rm norm}\n=\n\\frac{c_{21}}{1.315}\n-\\tanh(p_{10,3}-0.2266).\n";
    for (open, close) in [("$$", "$$"), (r"\[", r"\]")] {
        let message = format!("Before\n\n{open}{source}{close}\n\nAfter");
        let blocks = parsed(&message);
        assert!(
            matches!(&blocks[..], [Block::Paragraph { kind: Kind::Paragraph, .. }, Block::Math { source: actual, .. }, Block::Paragraph { kind: Kind::Paragraph, .. }] if actual.as_ref() == source)
        );
    }
}

#[test]
fn display_protection_keeps_comments_blank_lines_and_alignment_commands() {
    let source = "\nx = 1 % this comment ends here\n+ 2\n\n\\begin{aligned}a &= b \\\\\nc &= d\\end{aligned}\n";
    let blocks = parsed(&format!("$${source}$$"));
    assert!(
        matches!(&blocks[..], [Block::Math { source: actual, .. }] if actual.as_ref() == source)
    );
}

#[test]
fn formulas_do_not_reinterpret_fenced_inline_or_indented_code() {
    let source = "`$$x$$`\n\n```tex\n$$\nx\n=\ny\n$$\n```\n\n    $$x = y$$\n\n$$\nz\n=\n2\n$$";
    let blocks = parsed(source);
    assert_eq!(
        blocks
            .iter()
            .filter(|block| matches!(block, Block::Math { .. }))
            .count(),
        1
    );
    let codes: Vec<_> = blocks
        .iter()
        .filter_map(|block| match block {
            Block::Code { source, .. } => Some(source.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(codes, ["$$\nx\n=\ny\n$$\n", "$$x = y$$\n"]);
}

#[test]
fn streaming_and_literal_markers_do_not_alias_complete_formulas() {
    let first = parsed("$$\nx\n=\ny\n$$\n\n$$HERDRDISPLAY$$\n\n$$\nz");
    let formulas: Vec<_> = first
        .iter()
        .filter_map(|block| match block {
            Block::Math { source, .. } => Some(source.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(formulas, ["\nx\n=\ny\n", "HERDRDISPLAY"]);
    let escaped = r"\$$ literal dollars and $$$ not a pair $$$";
    assert_eq!(protect_display(&normalize_math(escaped)).markdown, escaped);
}

#[test]
fn an_unfinished_formula_cannot_borrow_its_closer_from_code_or_html() {
    for content in [
        "Unfinished \\[\n\n```tex\n\\]\n```",
        "Unfinished \\[\n\n<div>\\]</div>",
        "Unfinished \\[ `\\]`",
    ] {
        assert_eq!(normalize_math(content).text, content);
    }
    let source = "\n\\begin{aligned}\n\n    a &= b \\\\\n    c &= d\n\\end{aligned}\n";
    let blocks = parsed(&format!("\\[{source}\\]"));
    assert!(
        matches!(&blocks[..], [Block::Math { source: actual, .. }] if actual.as_ref() == source)
    );
}

#[test]
fn image_source_never_leaks_display_placeholders_or_normalized_delimiters() {
    for source in [
        "![formula $$\nx^2\n$$](local.png)",
        "![`code` \\[\nx^2\n\\]](local.png)",
    ] {
        let blocks = parsed(source);
        let visible: String = blocks
            .iter()
            .filter_map(|block| match block {
                Block::Paragraph { content, .. } => Some(
                    content
                        .iter()
                        .filter_map(|part| match part {
                            Inline::Text(text, _) => Some(text.as_str()),
                            _ => None,
                        })
                        .collect::<String>(),
                ),
                _ => None,
            })
            .collect();
        assert_eq!(visible, source);
    }
}

#[test]
fn quoted_display_formulas_keep_math_comparisons_but_not_markdown_markers() {
    for prefix in ["> ", "> > "] {
        let text = format!("{prefix}$$\n{prefix}x > 1\n{prefix}=\n{prefix}y\n{prefix}$$");
        let blocks = parsed(&text);
        assert!(
            matches!(&blocks[..], [Block::Math { source, .. }] if source.as_ref() == "\nx > 1\n=\ny\n")
        );
    }
}

#[test]
fn normalizing_math_keeps_code_and_html_where_a_fresh_parse_finds_them() {
    for text in [
        r"Area \( \pi r^2 \) and `\(code\)` then <b>\[x\]</b> \[ y \]",
        "Unfinished \\[\n\n```tex\n\\]\n```\n\\(a\\)",
        r"\(α\) ![\(img\)](x.png) \(β\)",
    ] {
        let normalized = normalize_math(text);
        assert_eq!(
            normalized.protected,
            protected_ranges(&normalized.text),
            "{text}"
        );
    }
}
