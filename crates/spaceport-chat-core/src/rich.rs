//! The block model a chat's Markdown is parsed into: a paragraph's inline
//! style, a heading, a list item, a quote, a table, a code block and a display
//! formula, each carrying the visual it draws by. The app paints them;
//! nothing here draws.
//!
//! The parse itself lives here too, because it applies this model's own
//! bounds: [`parse`] admits the code blocks and formulas a message may draw
//! and reports each as a [`Want`], so the app can ask for the visuals it
//! keeps without knowing which were admitted. Which destinations are links
//! is not this crate's to decide: the app hands `parse` its classification.
//!
//! The code, fence and quote rules that read the same text live beside it: a
//! code line's drawn and source offsets, whether a fence is closed, and which
//! block quote markers a line carries.

mod code;
mod fence;
mod math_input;
mod parse;
pub mod quote;

pub use code::{CodeText, MAX_CODE_LINE_CHARS, MAX_CODE_LINES, TAB_WIDTH};
pub use fence::Fence;
pub use parse::parse;

use crate::visual::{MathMode, Source, VisualId};
use pulldown_cmark::{Alignment, HeadingLevel};
use std::sync::Arc;

/// Bytes of one formula's TeX source; a longer one stays visible source.
pub const MAX_MATH_BYTES: usize = 4_096;
/// Formulas one message draws; later ones stay visible source.
const MAX_MATH: usize = 64;

/// A visual an admitted code block or formula asks for, with the identity it
/// is drawn by. [`parse`] reports these in document order, once each, after
/// its own bounds; only it knows which fences and formulas were admitted.
#[derive(Clone, Debug)]
pub struct Want {
    id: VisualId,
    source: Source,
}

impl Want {
    fn new(id: VisualId, source: Source) -> Self {
        Self { id, source }
    }

    pub fn id(&self) -> VisualId {
        self.id
    }

    pub fn into_source(self) -> Source {
        self.source
    }
}

/// A formula's identity and the TeX it is drawn from, when its source is
/// within bounds and the message has not drawn its last one.
fn formula(
    source: &str,
    mode: MathMode,
    count: &mut usize,
    wants: &mut Vec<Want>,
) -> Option<(VisualId, Arc<str>)> {
    if source.len() > MAX_MATH_BYTES || source.trim().is_empty() || *count >= MAX_MATH {
        return None;
    }
    *count += 1;
    let source: Arc<str> = source.into();
    let id = VisualId::of(("math", mode, &source));
    wants.push(Want::new(
        id,
        Source::Math {
            source: source.clone(),
            mode,
        },
    ));
    Some((id, source))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub strike: bool,
    pub link: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Inline {
    Text(String, Style),
    Math { id: VisualId, source: Arc<str> },
    Break,
}

/// What a paragraph block is, which sets its typography and marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Paragraph,
    Heading(HeadingLevel),
    Bullet,
    /// An ordered list item, with the number it was written with.
    Numbered(u32),
    Quote,
    /// A thematic break, which has no content.
    Rule,
}

#[derive(Debug)]
pub enum Block {
    Paragraph {
        kind: Kind,
        /// Nested list depth; bounded in the view so deep input stays readable.
        indent: usize,
        content: Vec<Inline>,
    },
    Math {
        id: VisualId,
        source: Arc<str>,
    },
    Code {
        id: VisualId,
        language: String,
        source: Arc<str>,
    },
    Table {
        align: Vec<Alignment>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
}

impl Block {
    /// The visuals this block draws: its own code or display formula, or the
    /// inline formulas in its text.
    pub fn visuals(&self) -> impl Iterator<Item = VisualId> + '_ {
        let (own, prose, rows): (_, &[Inline], &[Vec<Vec<Inline>>]) = match self {
            Self::Code { id, .. } | Self::Math { id, .. } => (Some(*id), &[], &[]),
            Self::Paragraph { content, .. } => (None, content, &[]),
            Self::Table { rows, .. } => (None, &[], rows),
        };
        let inline = prose.iter().chain(rows.iter().flatten().flatten());
        own.into_iter().chain(inline.filter_map(|part| match part {
            Inline::Math { id, .. } => Some(*id),
            _ => None,
        }))
    }
}

#[cfg(test)]
mod tests;
