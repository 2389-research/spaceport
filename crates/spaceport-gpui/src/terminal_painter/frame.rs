//! One frame's cells, layer by layer: backgrounds and highlights, text,
//! decorations, and the images between and above them.

use super::TerminalPainter;
use super::area::{Layer, Part, cell_ranges, clip, covers, whole};
use super::backgrounds::background_spans;
use super::decoration_spans;
use super::graphics::Graphic;
use super::grid::{Ground, background_extent, grid_rect, last_row_has_separator};
use super::highlight::Highlight;
use super::images::{ImageGeometry, PlacedImages, below_text};
use crate::terminal::{HIDDEN, Scrollbar, cell_colors, in_rect, on_default_background};
use gpui::{
    App, Bounds, Corners, Font, Pixels, Point, RenderImage, Size, Window, fill, point, px, rgb,
    rgba, size,
};
use spaceport_client::protocol::{CellData, FrameData, PaneSurfacePane, SurfaceRect};
use std::{sync::Arc, time::Instant};

/// The theme's scrollbar thumb: inset from the daemon's cell rect, and drawn
/// at this alpha over what it covers.
const SCROLLBAR_INSET: f32 = 1.;
const SCROLLBAR_ALPHA: u32 = 0xc0;

impl TerminalPainter {
    /// Paints one layer of the cells of one frame that `part` covers, or
    /// every layer of all of them, tinting the cells `highlights` name in that frame's own grid. Rows
    /// outside the frame are ignored: a selection or search was made against
    /// the live surface, which a repaint may already have replaced.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 299 lines")]
    pub fn paint_frame(
        &mut self,
        frame: &FrameData,
        origin: Point<Pixels>,
        available: Option<Size<Pixels>>,
        ground: Ground,
        cell_width: f32,
        font: &Font,
        highlights: &[Highlight],
        panes: &[PaneSurfacePane],
        part: Option<Part<'_>>,
        images: Option<PlacedImages<'_>>,
        window: &mut Window,
        cx: &mut App,
    ) {
        if frame.width == 0 {
            return;
        }
        // CPU scene construction only: this does not measure GPU completion.
        let started = Instant::now();
        let mut paint_errors = 0_u64;
        self.configure(font);
        let cached = true;
        #[cfg(feature = "integration-test")]
        let cached = cached && !self.uncached;
        #[cfg(feature = "integration-test")]
        let mut counts = crate::performance::Counts::default();
        let metrics = self.metrics(cell_width);
        let grid = Bounds::new(origin, metrics.extent(frame.width, frame.height));
        // Only fill a sub-cell remainder. Retained frames during resize and
        // mirrored groups must not stretch across whole missing rows/columns.
        // Popups have no remainder; their background stays inside their grid.
        let background = available.map_or(grid.size, |available| {
            background_extent(grid.size, available, metrics.cell())
        });
        let extend_last_row =
            background.height > grid.size.height && !last_row_has_separator(frame);
        let whole_area;
        let area = match part {
            Some(part) => part.area,
            None => {
                whole_area = whole(frame);
                &whole_area
            }
        };
        let draws = |layer| part.is_none_or(|part| part.layer == layer);
        // Over a backdrop a hidden cell's background is not painted, so its
        // glyphs, graphics, and decorations, drawn in that color, would show.
        let shows_through =
            |cell: &CellData| ground == Ground::Backdrop && cell.modifier & HIDDEN != 0;
        // The daemon's cell scrollbar is replaced by the pixel thumb painted below.
        let bars: Vec<SurfaceRect> = panes.iter().filter_map(|p| p.scrollbar_rect).collect();
        let in_bar = |index: usize| {
            let width = usize::from(frame.width);
            bars.iter()
                .any(|r| in_rect(*r, (index % width) as u16, (index / width) as u16))
        };
        let placed = images.map_or_else(Vec::new, |images| {
            self.images.prepare(
                images,
                origin,
                (cell_width, self.cell_height),
                grid,
                window,
                cx,
            )
        });
        let (below, above): (Vec<_>, Vec<_>) =
            placed.into_iter().partition(|(z, ..)| below_text(*z));
        // Text shares the backgrounds' layer unless something must come
        // between them: images below the text, or another region's cells.
        let text_with_backgrounds = part.is_none() && below.is_empty();
        // A layer gives all its primitives one draw order, skipping GPUI's
        // per-primitive BoundsTree insert that dominates large grids. Within a
        // layer quads draw before glyphs, so decorations and the cursor take a
        // second layer above the text.
        if draws(Layer::Backgrounds) {
            window.paint_layer(Bounds::new(origin, background), |window| {
                // Backgrounds precede all glyphs, including wide graphemes' skip cells.
                let width = usize::from(frame.width);
                for range in cell_ranges(frame, area) {
                    let y = range.start / width;
                    let row = &frame.cells[y * width..((y + 1) * width).min(frame.cells.len())];
                    let columns = range.start - y * width..range.end - y * width;
                    let mut paint = |start: usize, end: usize, color| {
                        let right = if end == usize::from(frame.width) {
                            background.width
                        } else {
                            px(end as f32 * cell_width)
                        };
                        let bottom = if extend_last_row && y + 1 == usize::from(frame.height) {
                            background.height
                        } else {
                            px((y + 1) as f32 * self.cell_height)
                        };
                        let bounds = grid_rect(
                            window,
                            origin,
                            point(
                                px(start as f32 * cell_width),
                                px(y as f32 * self.cell_height),
                            ),
                            point(right, bottom),
                        );
                        window.paint_quad(fill(bounds, rgb(color)));
                        #[cfg(feature = "integration-test")]
                        {
                            counts.quads += 1;
                        }
                    };
                    // Over a backdrop the ground is the theme's default; a
                    // color the program chose stays solid, even that one.
                    let shown = |default: bool| ground == Ground::Opaque || !default;
                    if cached {
                        for (start, end, color, default) in
                            background_spans(row, columns, &self.theme)
                        {
                            if shown(default) {
                                paint(start, end, color);
                            }
                        }
                    } else {
                        for x in columns {
                            if shown(on_default_background(&row[x])) {
                                paint(x, x + 1, cell_colors(&row[x], &self.theme).1);
                            }
                        }
                    }
                }
                // Between the backgrounds and the glyphs, so the tint reads as chosen
                // without hiding either.
                for Highlight { row, columns, tint } in &clip(highlights, area) {
                    let (start, end) =
                        (columns.start.min(frame.width), columns.end.min(frame.width));
                    if *row >= frame.height || start >= end {
                        continue;
                    }
                    let (top, bottom) = (f32::from(*row), f32::from(*row) + 1.);
                    window.paint_quad(fill(
                        grid_rect(
                            window,
                            origin,
                            point(
                                px(f32::from(start) * cell_width),
                                px(top * self.cell_height),
                            ),
                            point(
                                px(f32::from(end) * cell_width),
                                px(bottom * self.cell_height),
                            ),
                        ),
                        tint.color(&self.theme),
                    ));
                    #[cfg(feature = "integration-test")]
                    {
                        counts.quads += 1;
                    }
                }
                if text_with_backgrounds {
                    paint_errors += self.paint_text(
                        frame,
                        area,
                        origin,
                        cell_width,
                        font,
                        cached,
                        &in_bar,
                        &shows_through,
                        window,
                        #[cfg(feature = "integration-test")]
                        &mut counts,
                    );
                }
            });
        }
        if !text_with_backgrounds && draws(Layer::Text) {
            paint_errors += self.paint_images(&below, window);
            window.paint_layer(grid, |window| {
                paint_errors += self.paint_text(
                    frame,
                    area,
                    origin,
                    cell_width,
                    font,
                    cached,
                    &in_bar,
                    &shows_through,
                    window,
                    #[cfg(feature = "integration-test")]
                    &mut counts,
                );
            });
        }
        if draws(Layer::Decorations) {
            window.paint_layer(grid, |window| {
                // Box and block graphics are quads, so they share this layer to stay
                // above the backgrounds. Decorations cover the grid, including spaces
                // and wide-glyph continuation cells.
                for index in cell_ranges(frame, area).flatten() {
                    let cell = &frame.cells[index];
                    // Hidden cells draw in their background's color, which
                    // over a backdrop's skipped quad would show.
                    if in_bar(index) || shows_through(cell) {
                        continue;
                    }
                    let position = origin
                        + point(
                            px((index % usize::from(frame.width)) as f32 * cell_width),
                            px((index / usize::from(frame.width)) as f32 * self.cell_height),
                        );
                    if let Some(graphic) = (!cell.skip)
                        .then(|| Graphic::from_symbol(&cell.symbol))
                        .flatten()
                    {
                        let color = rgb(cell_colors(cell, &self.theme).0);
                        graphic.rectangles(
                            Bounds::new(position, size(px(cell_width), px(self.cell_height))),
                            window.scale_factor(),
                            |bounds| {
                                window.paint_quad(fill(bounds, color));
                                #[cfg(feature = "integration-test")]
                                {
                                    counts.quads += 1;
                                }
                            },
                        );
                    }
                    let row = index / usize::from(frame.width);
                    for (y, left, right) in
                        decoration_spans(frame, index, cell_width, self.cell_height)
                    {
                        // Absolute corners keep a run of underlined cells one unbroken line.
                        let top = row as f32 * self.cell_height + y;
                        window.paint_quad(fill(
                            grid_rect(
                                window,
                                origin,
                                point(px(left), px(top)),
                                point(px(right), px(top + 1.)),
                            ),
                            rgb(cell_colors(cell, &self.theme).0),
                        ));
                        #[cfg(feature = "integration-test")]
                        {
                            counts.decorations += 1;
                        }
                    }
                }
                if let Some(cursor) = frame.cursor.as_ref().filter(|c| {
                    c.visible && c.x < frame.width && c.y < frame.height && covers(area, c.x, c.y)
                }) {
                    let position = origin + metrics.origin(cursor.x, cursor.y);
                    let (offset, dimensions) = match cursor.shape {
                        3 | 4 => (
                            point(px(0.), px(self.cell_height - 2.)),
                            size(px(cell_width), px(2.)),
                        ),
                        5 | 6 => (point(px(0.), px(0.)), size(px(2.), px(self.cell_height))),
                        _ => (
                            point(px(0.), px(0.)),
                            size(px(cell_width), px(self.cell_height)),
                        ),
                    };
                    window.paint_quad(fill(
                        Bounds::new(position + offset, dimensions),
                        rgba((self.theme.cursor << 8) | 0x80),
                    ));
                    #[cfg(feature = "integration-test")]
                    {
                        counts.decorations += 1;
                    }
                }
                // A scrollbar paints with the cells at its top, so one paint owns it.
                for bar in panes
                    .iter()
                    .filter(|pane| {
                        pane.scrollbar_rect
                            .is_some_and(|rect| covers(area, rect.x, rect.y))
                    })
                    .filter_map(|pane| Scrollbar::new(pane, metrics))
                {
                    let width =
                        (f32::from(bar.track.size.width) - 2. * SCROLLBAR_INSET).clamp(2., 4.);
                    window.paint_quad(
                        fill(
                            Bounds::new(
                                origin
                                    + point(
                                        bar.track.right() - px(width + SCROLLBAR_INSET),
                                        bar.thumb.top(),
                                    ),
                                size(px(width), bar.thumb.size.height),
                            ),
                            rgba((self.theme.muted << 8) | SCROLLBAR_ALPHA),
                        )
                        .corner_radii(px(width / 2.)),
                    );
                }
            });
        }
        // Kitty draws `z >= 0` over the text, decorations and cursor included.
        paint_errors += self.paint_images(&above, window);
        #[cfg(feature = "integration-test")]
        {
            let total = cx.default_global::<crate::performance::Counts>();
            total.shapes += counts.shapes;
            total.quads += counts.quads;
            total.glyphs += counts.glyphs;
            total.paths += counts.paths;
            total.decorations += counts.decorations;
            total.paint_errors += counts.paint_errors;
            // A region paints its three layers as three calls; count the
            // region once, on its backgrounds, so `paints` stays a grid paint.
            total.paints += usize::from(draws(Layer::Backgrounds));
        }
        #[cfg(not(feature = "integration-test"))]
        let _ = cx;
        let now = Instant::now();
        if let Some(timing) = self.diagnostics.record(now, now.duration_since(started)) {
            let mean_ms = timing.total.as_secs_f64() * 1000. / timing.count as f64;
            let max_ms = timing.max.as_secs_f64() * 1000.;
            if timing.slow_count > 0 {
                tracing::warn!(
                    count = timing.count,
                    mean_ms,
                    max_ms,
                    slow_count = timing.slow_count,
                    "Terminal CPU paint timing"
                );
            } else {
                tracing::debug!(
                    count = timing.count,
                    mean_ms,
                    max_ms,
                    slow_count = timing.slow_count,
                    "Terminal CPU paint timing"
                );
            }
        }
        if let Some(count) = self.diagnostics.take_errors(now, paint_errors) {
            tracing::warn!(category = "glyph_paint", count, "Terminal paint failed");
        }
    }

    /// Paints images in order, returning how many failed.
    fn paint_images(
        &mut self,
        placed: &[(i32, ImageGeometry, Arc<RenderImage>)],
        window: &mut Window,
    ) -> u64 {
        let mut failed = 0;
        for (_z, geometry, texture) in placed {
            let painted = window.paint_image(
                geometry.visible,
                geometry.image,
                Corners::default(),
                texture.clone(),
                0,
                false,
            );
            failed += u64::from(painted.is_err());
            #[cfg(test)]
            self.painted_images.push((*_z, geometry.visible));
        }
        failed
    }
}
