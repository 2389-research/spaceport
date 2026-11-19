//! An uncommitted IME composition, painted over the cells at the input cursor.

use super::TerminalPainter;
use super::text::paint_glyphs;
use gpui::{Bounds, ContentMask, Font, Pixels, Point, Window, fill, point, px, rgb, size};

/// Where an IME composition sits: at the input cursor, shifted left only as
/// far as it takes to end inside the grid. Text wider than the grid loses its
/// start rather than its end, where the IME is editing.
pub(super) fn composition_origin(
    cursor: Point<Pixels>,
    width: Pixels,
    grid: Bounds<Pixels>,
) -> Point<Pixels> {
    point(cursor.x.min(grid.right() - width), cursor.y)
}

/// The byte index of a UTF-16 offset, as the platform input handler counts.
pub(super) fn byte_index(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    text.char_indices()
        .find(|(_, c)| {
            let found = units >= utf16;
            units += c.len_utf16();
            found
        })
        .map_or(text.len(), |(index, _)| index)
}

impl TerminalPainter {
    /// Paints an uncommitted IME composition over the cells at the input
    /// cursor. It is shaped as one line rather than per cell, so it can span
    /// wide glyphs, and it bypasses the glyph cache since it changes with
    /// every keystroke. Nothing reaches the pane until the IME commits.
    pub fn paint_composition(
        &self,
        text: &str,
        cursor: Point<Pixels>,
        grid: Bounds<Pixels>,
        font: &Font,
        window: &mut Window,
    ) {
        if text.is_empty() {
            return;
        }
        let line = self.shape(font, 0, text, window);
        let origin = composition_origin(cursor, line.width, grid);
        let bounds = Bounds::new(origin, size(line.width, px(self.cell_height)));
        let color = rgb(self.theme.foreground);
        // A layer of its own, painted after the frame, keeps it above the
        // cells and the cursor it covers. It clips to the input area, so text
        // wider than a popup never paints over the pane beneath it.
        window.with_content_mask(Some(ContentMask { bounds: grid }), |window| {
            window.paint_layer(bounds, |window| {
                window.paint_quad(fill(bounds, rgb(self.theme.background)));
                window.paint_quad(fill(
                    Bounds::new(
                        origin + point(px(0.), px(self.cell_height - 2.)),
                        size(line.width, px(1.)),
                    ),
                    color,
                ));
                if paint_glyphs(&line, origin, px(self.cell_height), color, window).is_err() {
                    tracing::warn!(category = "glyph_paint", "IME composition paint failed");
                }
            });
        });
    }

    /// The bounds of `range` (UTF-16) within a composition painted by
    /// `paint_composition`, so the IME can place its candidate window under
    /// the clause being converted.
    pub fn composition_bounds(
        &self,
        text: &str,
        range: std::ops::Range<usize>,
        cursor: Bounds<Pixels>,
        grid: Bounds<Pixels>,
        font: &Font,
        window: &Window,
    ) -> Bounds<Pixels> {
        if text.is_empty() {
            return cursor;
        }
        let line = self.shape(font, 0, text, window);
        let origin = composition_origin(cursor.origin, line.width, grid);
        let start = line.x_for_index(byte_index(text, range.start));
        let end = line.x_for_index(byte_index(text, range.end));
        let width = (end - start).max(cursor.size.width);
        // A caret after the text, or a clause scrolled off the left, still
        // anchors the candidate window inside the grid.
        let x = (origin.x + start)
            .min(grid.right() - width)
            .max(grid.left());
        Bounds::new(point(x, origin.y), size(width, cursor.size.height))
    }
}
