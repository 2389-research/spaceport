//! A rotozoomed field filling the stage: a checkered texture read through a
//! matrix that turns and breathes with the scene's time.

use super::{LOOP, cell, grid, ramp::ramp};
use gpui::{Bounds, Pixels, Window, fill, rgb};
use std::f32::consts::TAU;

/// The cells the field is painted in, in pixels.
const STEP: f32 = 9.;
/// How many turns the field makes in one loop.
pub(in crate::about) const TURNS: f32 = 2.;
/// How many times it breathes in and out in one loop.
const BREATHS: f32 = 3.;
/// How far the zoom runs either side of its middle.
const ZOOM: f32 = 0.6;
/// How many tiles the texture is cut into across.
const TILES: f32 = 5.;
/// How far out the texture's bright ring reaches, in tiles.
const RING: f32 = 0.22;
/// How much of the field shows over the layers beneath it.
const VEIL: f32 = 0.16;

/// The angle the field has turned at `t`, in radians.
pub(in crate::about) fn turn(t: f32) -> f32 {
    TAU * TURNS * t / LOOP
}

/// How far in the field is zoomed at `t`: its middle, 1, at the quarter
/// points of the loop.
pub(in crate::about) fn zoom(t: f32) -> f32 {
    1. + ZOOM * (TAU * BREATHS * t / LOOP).cos()
}

/// The texture coordinate `at` reads at `t`, wrapped into one tile: `at` is
/// in the field's own unit square, -0.5..0.5 from its centre, and the field
/// is read backwards from the screen, so a point turns with the field and
/// moves outward as it zooms in.
pub(in crate::about) fn sample(at: (f32, f32), t: f32) -> (f32, f32) {
    let (sin, cos) = turn(t).sin_cos();
    let zoom = zoom(t);
    (
        wrap((at.0 * cos + at.1 * sin) / zoom),
        wrap((-at.0 * sin + at.1 * cos) / zoom),
    )
}

/// `v` wrapped into 0..1, which is one tile of the texture.
fn wrap(v: f32) -> f32 {
    v - v.floor()
}

/// The texture at `uv`, 0..1 both ways: a checker, brightest inside a ring
/// around the middle.
pub(in crate::about) fn texel(uv: (f32, f32)) -> f32 {
    let check = ((uv.0 * TILES).floor() as i64 + (uv.1 * TILES).floor() as i64).rem_euclid(2);
    let ring = ((uv.0 - 0.5).powi(2) + (uv.1 - 0.5).powi(2)).sqrt() < RING;
    match (check, ring) {
        (0, false) => 0.15,
        (1, false) => 0.45,
        (0, true) => 0.8,
        _ => 1.,
    }
}

/// The colour of the texture at `uv`.
pub(in crate::about) fn colour(uv: (f32, f32), stops: &[u32; 3]) -> u32 {
    ramp(stops, texel(uv))
}

/// Paints the field over `area`, the whole stage the demo owns.
pub(in crate::about) fn paint(stops: &[u32; 3], area: Bounds<Pixels>, t: f32, window: &mut Window) {
    let (columns, rows) = grid(area, STEP);
    let side = f32::from(area.size.width).max(1.);
    for row in 0..rows {
        for column in 0..columns {
            // A cell reads the texture from its middle, in the field's own
            // square, so the field turns about the stage's centre.
            let at = (
                (column as f32 + 0.5) * STEP / side - 0.5,
                (row as f32 + 0.5) * STEP / side - 0.5,
            );
            let hue = colour(sample(at, t), stops);
            if let Some(quad) = cell(area, column, row, STEP) {
                window.paint_quad(fill(quad, rgb(hue).opacity(VEIL)));
            }
        }
    }
}
