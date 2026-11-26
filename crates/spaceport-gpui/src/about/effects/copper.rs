//! Copper bars: warm columns riding across the stage in a sine, each lit
//! across its width from a copper ramp so it reads as a lit tube.

use super::{LOOP, ramp::ramp};
use gpui::{Bounds, Pixels, Window, fill, point, px, rgb, size};
use std::f32::consts::{PI, TAU};

/// How many bars ride at once.
pub(in crate::about) const BARS: usize = 5;
/// How wide one is, as a share of the stage.
pub(in crate::about) const WIDTH: f32 = 0.06;
/// The share of the stage the bars' leading edges span, leaving a margin
/// either side for them to swing in.
const SPREAD: f32 = 0.9;
/// How far a bar swings either side of the place it owns, as a share of the
/// stage: less than half the gap between bars, so they keep their order.
const SWING: f32 = 0.05;
/// How many times the bars swing across in one loop.
const SWINGS: f32 = 2.;
/// The slices a bar is painted in across its width, so its ramp shows.
const SLICES: usize = 8;
/// How much of a bar shows over the layers beneath it.
const VEIL: f32 = 0.16;

/// Where bar `index` starts at `t`, 0..1 across the stage, at its leading
/// edge.
pub(in crate::about) fn place(index: usize, t: f32) -> f32 {
    let margin = (1. - SPREAD) / 2.;
    let own = index as f32 / BARS as f32;
    margin + SPREAD * own + SWING * (TAU * (SWINGS * t / LOOP + own)).sin()
}

/// How lit a bar is `across` its width: dark at either edge and brightest
/// down its middle.
pub(in crate::about) fn light(across: f32) -> f32 {
    (PI * across.clamp(0., 1.)).sin()
}

/// The colour of a bar `across` its width.
pub(in crate::about) fn colour(across: f32, stops: &[u32; 3]) -> u32 {
    ramp(stops, light(across))
}

/// Paints the bars over `bounds`.
pub(in crate::about) fn paint(
    stops: &[u32; 3],
    bounds: Bounds<Pixels>,
    t: f32,
    window: &mut Window,
) {
    let width = f32::from(bounds.size.width);
    let bar = width * WIDTH;
    let slice = bar / SLICES as f32;
    for index in 0..BARS {
        let left = place(index, t) * width;
        for step in 0..SLICES {
            let across = (step as f32 + 0.5) / SLICES as f32;
            let hue = colour(across, stops);
            let at = bounds.origin + point(px(left + step as f32 * slice), px(0.));
            let quad = Bounds::new(at, size(px(slice), bounds.size.height));
            window.paint_quad(fill(quad, rgb(hue).opacity(VEIL)));
        }
    }
}
