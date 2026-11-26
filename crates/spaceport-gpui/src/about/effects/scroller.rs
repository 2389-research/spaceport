//! The marquee that rides the top of the stage: the greeting drawn a glyph at
//! a time, each lifted by a sine of where it is across the stage, so the line
//! crosses as a wave.
//!
//! A demo picks the waves its marquee is made of, so one rides a single flat
//! wave and another crosses two lines against each other.

use super::LOOP;
use std::f32::consts::TAU;

/// How wide one glyph is, as a share of the stage's width.
pub(in crate::about) const ADVANCE: f32 = 0.05;

/// One wave a marquee's line rides: how fast the line travels and how the
/// sine under it is shaped.
#[derive(Clone, Copy, Debug)]
pub(in crate::about) struct Wave {
    /// How many times the line crosses the stage in one loop. Negative, it
    /// crosses the other way.
    pub passes: f32,
    /// How many waves ride across the stage.
    pub waves: f32,
    /// How far a glyph rises above the band's middle, in pixels, at a crest.
    pub lift: f32,
}

/// The wave a demo's greeting rides when its marquee is one line.
pub(in crate::about) const RIDE: Wave = Wave {
    passes: 2.,
    waves: 2.5,
    lift: 3.,
};

/// The wave of a marquee's second line: shallower, taller, and crossing
/// against the first, so the two read as one moving figure. A whole number of
/// passes, like every wave's, so the loop repeats without a seam.
pub(in crate::about) const COUNTER: Wave = Wave {
    passes: -1.,
    waves: 1.5,
    lift: 5.,
};

/// How far the glyph at `x` rises above the band's middle, in pixels.
pub(in crate::about) fn lift(wave: &Wave, x: f32) -> f32 {
    wave.lift * (TAU * x * wave.waves).sin()
}

/// Where the glyph at `index` of `count` sits at `t`: its leading edge as a
/// share of the stage's width, and how far it is lifted above the band's
/// middle in pixels. The line enters from beyond the stage's end and leaves
/// past its start, so a glyph outside 0..1 is not on screen.
pub(in crate::about) fn glyph(wave: &Wave, index: usize, count: usize, t: f32) -> (f32, f32) {
    // The whole line travels its own length plus the stage's, so its last
    // glyph leaves before its first returns.
    let span = 1. + count as f32 * ADVANCE;
    let travel = (wave.passes.abs() * t / LOOP).fract() * span;
    let x = if wave.passes >= 0. {
        1. - travel
    } else {
        // The other way: the line waits off the stage's start and crosses it
        // as it travels.
        -(count as f32) * ADVANCE + travel
    } + index as f32 * ADVANCE;
    (x, lift(wave, x))
}
