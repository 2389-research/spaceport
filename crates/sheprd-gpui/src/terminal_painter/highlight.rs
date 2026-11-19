//! What a paint tints: the cells a selection, a search match, or the copy-mode
//! cursor covers.
//!
//! `Tint::color` is the only part that needs `gpui`, so a future `sheprd-cells`
//! crate takes the rest as it stands.

use crate::config::Theme;
use gpui::{Rgba, rgba};

/// The selection tints the cells it covers instead of replacing their colors:
/// a terminal's own background is meaningful, and the glyphs above it stay
/// readable on every theme.
const SELECTION_ALPHA: u32 = 0x59;
/// Search matches tint like the selection, the current one strongly enough to
/// find at a glance among the others.
const MATCH_ALPHA: u32 = 0x4d;
const CURRENT_MATCH_ALPHA: u32 = 0xa6;

/// Why a span of cells is tinted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tint {
    Selection,
    Match,
    CurrentMatch,
    /// The copy-mode cursor, drawn as a block over its cell.
    CopyCursor,
}

impl Tint {
    pub(super) fn color(self, theme: &Theme) -> Rgba {
        match self {
            Self::Selection => rgba((theme.primary() << 8) | SELECTION_ALPHA),
            Self::Match => rgba((theme.palette[3] << 8) | MATCH_ALPHA),
            Self::CurrentMatch => rgba((theme.palette[3] << 8) | CURRENT_MATCH_ALPHA),
            Self::CopyCursor => rgba((theme.cursor << 8) | CURRENT_MATCH_ALPHA),
        }
    }
}

/// Tinted cells of one row, in the grid of the frame that paints them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Highlight {
    pub(crate) row: u16,
    pub(crate) columns: std::ops::Range<u16>,
    pub(crate) tint: Tint,
}
