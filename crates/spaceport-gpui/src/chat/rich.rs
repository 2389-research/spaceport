//! Structured Markdown kept separately from layout: a message is parsed into
//! the block model [`spaceport_chat_core::rich`] defines and split into its
//! diagrams and pages by the fences that model reads. Math rasterization and
//! grammar parsing use the window's `Visuals` workers.
//!
//! The values parsed for — a paragraph's inline style, a heading, a list
//! item, a table, a code block, a display formula, a code line's drawn
//! offsets, a fence, a block quote marker — are the crate's, as is the parse;
//! this module re-exports them, names the destinations this client may act on,
//! and keeps what builds a visual's texture.

mod math;

// What this module keeps, and the crate's rich model at the paths the rest of
// the chat reads it at.
pub(super) use math::draw as draw_math;
pub(crate) use spaceport_chat_core::rich::{Block, Inline, Kind, Style};
pub(super) use spaceport_chat_core::rich::{
    CodeText, MAX_CODE_LINE_CHARS, MAX_CODE_LINES, MAX_MATH_BYTES,
};
pub(crate) use spaceport_chat_core::visual::MathMode;

use crate::{browser::WebUrl, terminal::file_target};
use std::borrow::Cow;

/// The destination this client may act on: a web address, or a path its
/// terminal's link rules read. Anything else stays visible source.
pub(super) fn destination(url: &str) -> Option<String> {
    WebUrl::try_from(url)
        .ok()
        .map(|url| url.as_str().to_owned())
        .or_else(|| file_target(url).map(|_| url.to_owned()))
}

/// Whether a destination names a file on the machine this client runs on.
fn is_local_path(url: &str) -> bool {
    file_target(url).is_some()
}

/// `content` with the links that name files elsewhere made inert, each with
/// its destination written out after the label.
pub(super) fn remote_content(content: &[Inline]) -> Cow<'_, [Inline]> {
    spaceport_chat_core::rich::remote_content(content, is_local_path)
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
