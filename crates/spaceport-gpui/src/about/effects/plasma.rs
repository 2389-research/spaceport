//! The field every other layer sits in: four crossing sine waves, each cell
//! of the stage painted the colour its wave lands on.

use super::{LOOP, cell, grid, ramp::ramp};
use gpui::{Bounds, Pixels, Window, fill, rgb};
use std::f32::consts::TAU;

/// One cell of the field, in pixels. The waves are smooth; drawn this coarse
/// they read as the low-colour plasma a demo scene paints in, and the stage
/// stays a bounded number of quads a frame.
const STEP: f32 = 14.;
/// How much of the field shows over the stage's own ground: enough to move,
/// little enough to keep the credits' contrast.
const VEIL: f32 = 0.16;

/// The field at `at`, 0..1 across the stage, `t` seconds into the loop: 0 at
/// its darkest and 1 at its brightest.
pub(in crate::about) fn field(at: (f32, f32), t: f32) -> f32 {
    let (x, y) = at;
    let turn = TAU * t / LOOP;
    let wave = (x * 6.5 + turn).sin()
        + (y * 7.5 - turn).sin()
        + ((x + y) * 4.5 + turn * 2.).sin()
        + (((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt() * 13. - turn * 3.).sin();
    (wave / 4. + 0.5).clamp(0., 1.)
}

/// The colour of a cell the field lights at `v`.
pub(in crate::about) fn colour(v: f32, stops: &[u32; 3]) -> u32 {
    ramp(stops, v)
}

/// Paints the field over `bounds`.
pub(in crate::about) fn paint(
    stops: &[u32; 3],
    bounds: Bounds<Pixels>,
    t: f32,
    window: &mut Window,
) {
    let (columns, rows) = grid(bounds, STEP);
    let (width, height) = (
        f32::from(bounds.size.width).max(1.),
        f32::from(bounds.size.height).max(1.),
    );
    for row in 0..rows {
        for column in 0..columns {
            let Some(quad) = cell(bounds, column, row, STEP) else {
                continue;
            };
            // The cell's middle reads the field, so the waves line up at any
            // stage size.
            let at = (
                (column as f32 + 0.5) * STEP / width,
                (row as f32 + 0.5) * STEP / height,
            );
            let hue = colour(field(at, t), stops);
            window.paint_quad(fill(quad, rgb(hue).opacity(VEIL)));
        }
    }
}
