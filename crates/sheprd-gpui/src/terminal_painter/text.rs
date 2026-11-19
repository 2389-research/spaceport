//! Cell text: the shaping cache's glyphs, and the separators drawn as paths
//! instead.

use super::TerminalPainter;
use super::area::{Span, cell_ranges};
use super::glyphs;
use super::graphics::{CellSeparator, Graphic};
use super::grid::grid_corners;
use crate::terminal::cell_colors;
use anyhow::Result;
use gpui::{Font, Pixels, Point, Rgba, ShapedLine, Window, point, px, rgb};
use sheprd_client::protocol::{CellData, FrameData};

/// `ShapedLine::paint` without its per-call layer, whose BoundsTree insert
/// would otherwise run once per cell. Glyph placement matches GPUI's.
pub(super) fn paint_glyphs(
    line: &ShapedLine,
    origin: Point<Pixels>,
    line_height: Pixels,
    color: Rgba,
    window: &mut Window,
) -> Result<()> {
    let baseline = origin
        + point(
            px(0.),
            (line_height - line.ascent - line.descent) / 2. + line.ascent,
        );
    for run in &line.runs {
        for glyph in &run.glyphs {
            let position = baseline + point(glyph.position.x, px(0.));
            if glyph.is_emoji {
                window.paint_emoji(position, run.font_id, glyph.id, line.font_size)?;
            } else {
                window.paint_glyph(
                    position,
                    run.font_id,
                    glyph.id,
                    line.font_size,
                    color.into(),
                )?;
            }
        }
    }
    Ok(())
}

impl TerminalPainter {
    /// Shapes and paints every visible glyph `area` covers, returning failures.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_text(
        &mut self,
        frame: &FrameData,
        area: &[Span],
        origin: Point<Pixels>,
        cell_width: f32,
        font: &Font,
        cached: bool,
        in_bar: &impl Fn(usize) -> bool,
        shows_through: &impl Fn(&CellData) -> bool,
        window: &mut Window,
        #[cfg(feature = "integration-test")] counts: &mut crate::performance::Counts,
    ) -> u64 {
        let mut paint_errors = 0;
        for index in cell_ranges(frame, area).flatten() {
            let cell = &frame.cells[index];
            if cell.skip
                || cell.symbol.is_empty()
                || cell.symbol == " "
                || shows_through(cell)
                || in_bar(index)
                || Graphic::from_symbol(&cell.symbol).is_some()
            {
                continue;
            }
            if let Some(separator) = CellSeparator::from_symbol(&cell.symbol) {
                let (column, row) = (
                    index % usize::from(frame.width),
                    index / usize::from(frame.width),
                );
                // A separator the daemon measures as wide spans its continuation cell,
                // as the font glyph would.
                let wide = column + 1 < usize::from(frame.width)
                    && frame.cells.get(index + 1).is_some_and(|next| next.skip);
                let cells = if wide { 2 } else { 1 };
                let (near, far) = grid_corners(
                    window,
                    origin,
                    point(
                        px(column as f32 * cell_width),
                        px(row as f32 * self.cell_height),
                    ),
                    point(
                        px((column + cells) as f32 * cell_width),
                        px((row + 1) as f32 * self.cell_height),
                    ),
                );
                // A degenerate cell whose path cannot tessellate keeps the font glyph.
                if let Ok(path) = separator.path(near, far) {
                    let color = rgb(cell_colors(cell, &self.theme).0);
                    #[cfg(test)]
                    self.painted_separators.push((path.bounds, color));
                    window.paint_path(path, color);
                    #[cfg(feature = "integration-test")]
                    {
                        counts.paths += 1;
                    }
                    continue;
                }
            }
            let style = glyphs::style(cell.modifier);
            let newly_shaped;
            let shaped = match cached
                .then(|| self.glyphs.get(style, &cell.symbol))
                .flatten()
            {
                Some(line) => line,
                None => {
                    #[cfg(feature = "integration-test")]
                    {
                        counts.shapes += 1;
                    }
                    let line = self.shape(font, style, &cell.symbol, window);
                    if cached && self.glyphs.has_room() {
                        self.glyphs.insert(style, &cell.symbol, line)
                    } else {
                        newly_shaped = line;
                        &newly_shaped
                    }
                }
            };
            let position = origin
                + point(
                    px((index % usize::from(frame.width)) as f32 * cell_width),
                    px((index / usize::from(frame.width)) as f32 * self.cell_height),
                );
            let color = rgb(cell_colors(cell, &self.theme).0);
            let result = paint_glyphs(shaped, position, px(self.cell_height), color, window);
            paint_errors += u64::from(result.is_err());
            #[cfg(feature = "integration-test")]
            {
                counts.glyphs += shaped.runs.iter().map(|r| r.glyphs.len()).sum::<usize>();
                counts.paint_errors += usize::from(result.is_err());
            }
        }
        paint_errors
    }
}
