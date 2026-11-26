//! Stars flying out of the screen: a table scattered once from a fixed hash,
//! each star a point projected from its own depth, so the field is the same
//! on every launch and a frame depends on the time alone.

use super::LOOP;
use gpui::{Bounds, Pixels, Window, fill, point, px, rgb, size};
use std::sync::LazyLock;

/// How many stars the field holds.
pub(in crate::about) const STARS: usize = 96;
/// How many times a star flies from the far plane to the near one in a loop.
const TRIPS: f32 = 6.;
/// The depth a star has reached when it leaves, as a share of the far plane:
/// near enough that it has flown past the stage's edge.
const NEAR: f32 = 0.12;
/// A star's width in pixels, whatever its depth: the field does not fill in
/// as stars arrive.
const SPECK: f32 = 2.;
/// How much of the theme's ink a star at the near plane shows.
const GLOW: f32 = 0.7;
/// How far outside the field a star is still drawn, so one leaving the
/// stage's edge is whole rather than cut off.
const EDGE: f32 = 1.05;

/// A star: its place in the far plane, -1..1 both ways, and how far into its
/// flight it starts.
#[derive(Clone, Copy, Debug)]
pub(in crate::about) struct Star {
    pub x: f32,
    pub y: f32,
    pub launch: f32,
}

/// Where a star shows at `t`: `at` is in units of the field's half-width
/// from its centre, and `brightness` runs 0 at the far plane to 1 at the
/// near one.
#[derive(Clone, Copy, Debug)]
pub(in crate::about) struct Sample {
    pub at: (f32, f32),
    pub brightness: f32,
}

static FIELD: LazyLock<[Star; STARS]> = LazyLock::new(|| {
    std::array::from_fn(|index| Star {
        x: scatter(index, 1) * 2. - 1.,
        y: scatter(index, 2) * 2. - 1.,
        launch: scatter(index, 3),
    })
});

/// The stars, in the order they are painted.
pub(in crate::about) fn stars() -> &'static [Star] {
    FIELD.as_slice()
}

/// A number in 0..1 for `index` and `salt`, hashed from them alone: the
/// field is the same on every launch, and a test can name a star's place.
pub(in crate::about) fn scatter(index: usize, salt: u32) -> f32 {
    let mut hash = (index as u32)
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(salt.wrapping_mul(0x85EB_CA6B));
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x2C1B_3C6D);
    hash ^= hash >> 12;
    (hash % 10_000) as f32 / 10_000.
}

/// Where `star` is `t` seconds into the loop, or `None` once it has flown
/// past the field's edge: a point at depth 1 reads its own place, and one
/// nearer the viewer is thrown outward as the depth shrinks.
pub(in crate::about) fn project(star: &Star, t: f32) -> Option<Sample> {
    let flight = (star.launch + TRIPS * t / LOOP).fract();
    let depth = 1. + (NEAR - 1.) * flight;
    let at = (star.x / depth, star.y / depth);
    (at.0.abs() <= EDGE && at.1.abs() <= EDGE).then_some(Sample {
        at,
        brightness: flight,
    })
}

/// Paints the field over `bounds`, its stars centred on the stage.
pub(in crate::about) fn paint(spark: u32, bounds: Bounds<Pixels>, t: f32, window: &mut Window) {
    let half = f32::from(bounds.size.width.min(bounds.size.height)) / 2.;
    let centre = bounds.center();
    for star in stars() {
        let Some(sample) = project(star, t) else {
            continue;
        };
        let at = point(
            centre.x + px(sample.at.0 * half),
            centre.y + px(sample.at.1 * half),
        );
        let speck = Bounds::new(at, size(px(SPECK), px(SPECK)));
        window.paint_quad(fill(speck, rgb(spark).opacity(sample.brightness * GLOW)));
    }
}
