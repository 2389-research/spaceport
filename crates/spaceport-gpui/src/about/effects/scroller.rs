//! The marquee that rides the top of the stage: the greeting drawn a glyph at
//! a time, each lifted by a sine of where it is across the stage, so the line
//! crosses as a wave.

use super::LOOP;
use std::f32::consts::TAU;

/// How wide one glyph is, as a share of the stage's width.
pub(in crate::about) const ADVANCE: f32 = 0.05;
/// How many times the line crosses the stage in one loop.
const PASSES: f32 = 2.;
/// How many waves ride across the stage.
const WAVES: f32 = 2.5;
/// How far a glyph rises above the band's middle, in pixels, at a crest.
pub(in crate::about) const LIFT: f32 = 3.;

/// How far the glyph at `x` rises above the band's middle, in pixels.
pub(in crate::about) fn lift(x: f32) -> f32 {
    LIFT * (TAU * x * WAVES).sin()
}

/// Where the glyph at `index` of `count` sits at `t`: its leading edge as a
/// share of the stage's width, and how far it is lifted above the band's
/// middle in pixels. The line enters from beyond the stage's end and leaves
/// past its start, so a glyph outside 0..1 is not on screen.
pub(in crate::about) fn glyph(index: usize, count: usize, t: f32) -> (f32, f32) {
    // The whole line travels its own length plus the stage's, so its last
    // glyph leaves before its first returns.
    let span = 1. + count as f32 * ADVANCE;
    let x = 1. - (PASSES * t / LOOP).fract() * span + index as f32 * ADVANCE;
    (x, lift(x))
}
