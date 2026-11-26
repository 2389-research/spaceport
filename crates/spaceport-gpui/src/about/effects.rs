//! The demoscene layers the About box paints under its credits. Each layer is
//! a function of the scene's time alone: no clock, no state, and nothing
//! allocated but the element tree, so a frame never depends on the one before
//! it and a test can pin the maths rather than the pixels.
//!
//! The stage is one canvas, so the layers paint into the same bounds in turn:
//! [`plasma`] fills it, [`starfield`] and [`copper`] move over that, and
//! [`rotozoom`] replaces it with its own field. [`scroller`] rides above them as
//! real text.
//! Each layer completes a whole number of its own turns in [`LOOP`] seconds,
//! so the scene repeats without a seam and the animation element that drives
//! it can simply loop.

pub(super) mod copper;
pub(super) mod plasma;
pub(super) mod ramp;
pub(super) mod rotozoom;
pub(super) mod scroller;
pub(super) mod starfield;

use gpui::{Bounds, Pixels, point, px, size};

/// The seconds one turn of the scene takes. Every layer's time term is a
/// whole number of turns in it.
pub(super) const LOOP: f32 = 12.;

/// The columns and rows a grid of cells `step` pixels wide lays over
/// `bounds`.
pub(super) fn grid(bounds: Bounds<Pixels>, step: f32) -> (usize, usize) {
    let columns = (f32::from(bounds.size.width) / step).ceil().max(1.);
    let rows = (f32::from(bounds.size.height) / step).ceil().max(1.);
    (columns as usize, rows as usize)
}

/// The cell at `column` and `row` of such a grid, cut to `bounds` so no layer
/// paints past the stage; `None` for a cell that falls outside it.
pub(super) fn cell(
    bounds: Bounds<Pixels>,
    column: usize,
    row: usize,
    step: f32,
) -> Option<Bounds<Pixels>> {
    let left = column as f32 * step;
    let top = row as f32 * step;
    let width = (f32::from(bounds.size.width) - left).min(step);
    let height = (f32::from(bounds.size.height) - top).min(step);
    (width > 0. && height > 0.).then(|| {
        Bounds::new(
            point(bounds.origin.x + px(left), bounds.origin.y + px(top)),
            size(px(width), px(height)),
        )
    })
}

#[cfg(test)]
mod tests;
