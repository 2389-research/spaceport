//! Structured Markdown kept separately from layout: the app parses a message
//! into the block model [`spaceport_chat_core::rich`] defines, asking for the
//! visuals its formulas and code blocks need, and splits a message into its
//! diagrams and pages by the fences that model reads. Math rasterization and
//! grammar parsing use Visuals' workers.
//!
//! The values parsed here — a paragraph's inline style, a heading, a list item,
//! a table, a code block, a display formula, a code line's drawn offsets, a
//! fence, a block quote marker — are the crate's; this module re-exports them
//! and keeps what builds them.

mod math;
mod math_input;
mod parse;
mod remote;

// The rich model below the app's views, at the paths the rest of the chat
// reads it at.
pub(super) use math::draw as draw_math;
pub(super) use parse::parse;
pub(super) use remote::remote_content;
pub(crate) use spaceport_chat_core::rich::{Block, Inline, Kind, Style};
pub(super) use spaceport_chat_core::rich::{
    CodeText, Fence, MAX_CODE_LINE_CHARS, MAX_CODE_LINES, TAB_WIDTH, quote,
};
pub(crate) use spaceport_chat_core::visual::MathMode;

use super::visual::{Source, VisualId, Visuals};
use std::sync::Arc;

pub(super) const MAX_MATH_BYTES: usize = 4_096;
pub(super) const MAX_MATH: usize = 64;
pub(super) const MAX_CODE_BYTES: usize = 32 * 1024;

fn formula(
    source: &str,
    mode: MathMode,
    count: &mut usize,
    visuals: &mut Visuals,
) -> Option<(VisualId, Arc<str>)> {
    if source.len() > MAX_MATH_BYTES || source.trim().is_empty() || *count >= MAX_MATH {
        return None;
    }
    *count += 1;
    let source: Arc<str> = source.into();
    let id = VisualId::of(("math", mode, &source));
    visuals.want(
        id,
        Source::Math {
            source: source.clone(),
            mode,
        },
    );
    Some((id, source))
}

/// Classify bounded code text with the shared chat grammar cache.
pub(super) fn highlight(language: &str, source: &str) -> Vec<Vec<super::highlight::Span>> {
    let extension = match language {
        "rust" => "rs",
        "python" | "py" => "py",
        "javascript" | "js" | "jsx" => "js",
        "typescript" | "ts" | "tsx" => "ts",
        "bash" | "sh" | "shell" | "zsh" => "sh",
        "c++" | "cpp" => "cpp",
        "c#" | "csharp" => "cs",
        "ruby" => "rb",
        "yaml" | "yml" => "yaml",
        "markdown" | "md" => "md",
        other => other,
    };
    super::highlight::colour_file(&format!("chat.{extension}"), source, 512)
}

#[cfg(test)]
mod tests;
