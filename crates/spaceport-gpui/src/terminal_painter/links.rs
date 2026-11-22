//! The solid underline a hovered link paints over its cells.

use super::TerminalPainter;
use super::decorations::underline_offset;
use super::grid::grid_rect;
use crate::terminal::cell_colors;
use gpui::{Bounds, Pixels, Point, Window, fill, point, px, rgb};
use spaceport_client::protocol::{FrameData, SurfaceRect};

/// The cell rectangle covering every in-frame cell of `rows`, if any.
///
/// gpui-free: `spaceport-cells` takes it with the rest of the cell-rect algebra.
pub(super) fn link_bounds(
    frame: &FrameData,
    rows: &[(u16, std::ops::Range<u16>)],
) -> Option<SurfaceRect> {
    let mut cells = rows.iter().filter_map(|(row, columns)| {
        let end = columns.end.min(frame.width);
        (*row < frame.height && columns.start < end).then_some((*row, columns.start, end))
    });
    let (row, start, end) = cells.next()?;
    let (top, bottom, left, right) = cells.fold(
        (row, row, start, end),
        |(top, bottom, left, right), (row, start, end)| {
            (
                top.min(row),
                bottom.max(row),
                left.min(start),
                right.max(end),
            )
        },
    );
    Some(SurfaceRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top + 1,
    })
}

impl TerminalPainter {
    /// Underlines the cells of a hovered link, painted after `paint_frame`
    /// for the same frame and origin. Each cell takes its own text color, as
    /// an SGR underline would, and spaces along the link are underlined too.
    pub fn paint_link(
        &self,
        frame: &FrameData,
        origin: Point<Pixels>,
        cell_width: f32,
        rows: &[(u16, std::ops::Range<u16>)],
        window: &mut Window,
    ) {
        let Some(bounds) = link_bounds(frame, rows) else {
            return;
        };
        let metrics = self.metrics(cell_width);
        let layer = Bounds::new(
            origin + metrics.origin(bounds.x, bounds.y),
            metrics.extent(bounds.width, bounds.height),
        );
        window.paint_layer(layer, |window| {
            for (row, columns) in rows {
                if *row >= frame.height {
                    continue;
                }
                for column in columns.start..columns.end.min(frame.width) {
                    let index = usize::from(*row) * usize::from(frame.width) + usize::from(column);
                    let Some(data) = frame.cells.get(index) else {
                        continue;
                    };
                    // The same snapped corners as the dashes this line replaces.
                    let top =
                        f32::from(*row) * self.cell_height + underline_offset(self.cell_height);
                    window.paint_quad(fill(
                        grid_rect(
                            window,
                            origin,
                            point(px(f32::from(column) * cell_width), px(top)),
                            point(px(f32::from(column + 1) * cell_width), px(top + 1.)),
                        ),
                        rgb(cell_colors(data, &self.theme).0),
                    ));
                }
            }
        });
    }
}
