// ABOUTME: Paints terminal cells, decorations, and images from prepared frame data.
// ABOUTME: Reuses font-dependent glyph shapes while applying current colours during paint.
//! Painting one terminal surface: the cells, their glyphs, the decorations
//! over them, and the images between and above them.
//!
//! Each layer's work is the module named for it; this file wires them together
//! and holds the `TerminalPainter` every paint borrows.

mod area;
mod backgrounds;
mod composition;
mod decorations;
mod diagnostics;
mod frame;
mod glyphs;
mod graphics;
mod grid;
mod highlight;
mod image_paint;
#[cfg(test)]
pub(crate) mod image_probe;
mod images;
mod links;
mod surface;
mod text;

pub(crate) use self::area::{Layer, Part, Span, cell_ranges, clip, covers};
pub(crate) use self::decorations::decoration_spans;
use self::diagnostics::PaintDiagnostics;
use self::glyphs::GlyphCache;
pub(crate) use self::graphics::CellSeparator;
pub(crate) use self::grid::Ground;
pub(crate) use self::highlight::{Highlight, Tint};
pub(crate) use self::image_paint::ImageBatch;
use self::images::ImageCache;
pub(crate) use self::images::{ImageRegion, ImageTarget, PlacedImages};
pub(crate) use self::images::{fit_bgra, texture};

use crate::config::Theme;
use crate::terminal::{CELL_HEIGHT, FONT_SIZE};
use gpui::Font;
#[cfg(test)]
use gpui::{Bounds, Pixels, Rgba};
use std::time::Instant;

/// The painter settings that affect the recorded cell scene.
#[derive(PartialEq)]
pub(crate) struct Appearance {
    font_size: f32,
    cell_height: f32,
    theme: Theme,
}

pub(crate) struct TerminalPainter {
    font_size: f32,
    cell_height: f32,
    theme: Theme,
    config: Option<Font>,
    // Resolved foreground includes reverse, dim and hidden; only bold/italic
    // affect shaping. Decorations remain at exact cell-grid coordinates.
    glyphs: GlyphCache,
    cell_width: Option<f32>,
    diagnostics: PaintDiagnostics,
    images: ImageCache,
    /// Each image painted, with its z, in paint order.
    #[cfg(test)]
    painted_images: Vec<(i32, Bounds<Pixels>)>,
    #[cfg(test)]
    painted_separators: Vec<(Bounds<Pixels>, Rgba)>,
    #[cfg(feature = "integration-test")]
    pub uncached: bool,
}

impl TerminalPainter {
    pub(crate) fn appearance(&self) -> Appearance {
        Appearance {
            font_size: self.font_size,
            cell_height: self.cell_height,
            theme: self.theme.clone(),
        }
    }
}

impl Default for TerminalPainter {
    fn default() -> Self {
        Self {
            font_size: FONT_SIZE,
            cell_height: CELL_HEIGHT,
            theme: Theme::default(),
            config: None,
            glyphs: GlyphCache::default(),
            cell_width: None,
            diagnostics: PaintDiagnostics::new(Instant::now()),
            images: ImageCache::default(),
            #[cfg(test)]
            painted_images: Vec::new(),
            #[cfg(test)]
            painted_separators: Vec::new(),
            #[cfg(feature = "integration-test")]
            uncached: false,
        }
    }
}

#[cfg(test)]
mod tests;
