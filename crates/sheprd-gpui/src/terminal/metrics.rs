//! The pixel size of one terminal cell, and the conversions between cells and
//! pixels that painting, hit testing, and IME placement share.

use gpui::{Bounds, Pixels, Point, Size, point, px, size};
use sheprd_client::protocol::SurfaceRect;

/// One cell's width and height in pixels. Both come from one appearance (font,
/// font size, line height), so a width never travels without its height.
///
/// Every helper multiplies a cell count by one side and nothing else, so its
/// result is bit-for-bit what `f32::from(count) * side` gives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CellMetrics {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl CellMetrics {
    pub(crate) const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Whether both sides can divide a pixel distance into cells.
    pub(crate) fn usable(self) -> bool {
        self.width.is_finite() && self.height.is_finite() && self.width > 0. && self.height > 0.
    }

    /// The size of one cell.
    pub(crate) fn cell(self) -> Size<Pixels> {
        size(px(self.width), px(self.height))
    }

    /// The top-left corner of the cell at `column`, `row`, from the grid's origin.
    pub(crate) fn origin(self, column: u16, row: u16) -> Point<Pixels> {
        point(
            px(f32::from(column) * self.width),
            px(f32::from(row) * self.height),
        )
    }

    /// The size of `columns` by `rows` cells.
    pub(crate) fn extent(self, columns: u16, rows: u16) -> Size<Pixels> {
        size(
            px(f32::from(columns) * self.width),
            px(f32::from(rows) * self.height),
        )
    }

    /// The pixels `rect` covers, from the grid's origin.
    pub(crate) fn bounds(self, rect: SurfaceRect) -> Bounds<Pixels> {
        Bounds::new(
            self.origin(rect.x, rect.y),
            self.extent(rect.width, rect.height),
        )
    }

    /// The cell holding the point `x`, `y`, in pixels from the grid's origin:
    /// each axis divided and floored, saturating at the `u16` range. Callers
    /// check `usable` and their own bounds first.
    pub(crate) fn cell_at(self, x: f32, y: f32) -> (u16, u16) {
        (
            (x / self.width).floor() as u16,
            (y / self.height).floor() as u16,
        )
    }
}
