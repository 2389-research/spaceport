//! The rotozoomed field behind the mark.

use super::super::LOOP;
use super::super::ramp::ramp;
use super::super::rotozoom::{TURNS, colour, sample, texel, turn, zoom};
use super::STOPS;
use std::f32::consts::TAU;

/// `v` wrapped into 0..1, as the field wraps its texture.
fn wrapped(v: f32) -> f32 {
    v.rem_euclid(1.)
}

/// Whether `v` reads `expected`, allowing for the seam where one tile ends
/// and the next begins, and for the float error of a reading a whole loop
/// apart.
fn reads(v: f32, expected: f32) -> bool {
    let apart = (v - expected).rem_euclid(1.);
    apart < 1e-5 || apart > 1. - 1e-5
}

#[test]
fn the_field_turns_whole_turns_in_a_loop() {
    assert_eq!(turn(0.), 0.);
    assert!((turn(LOOP) - TAU * TURNS).abs() < 1e-3);
    assert!(turn(LOOP / 2.) > turn(LOOP / 4.));
}

#[test]
fn the_field_breathes_between_its_two_zooms() {
    assert!((zoom(0.) - 1.6).abs() < 1e-6);
    assert!((zoom(LOOP / 6.) - 0.4).abs() < 1e-5);
    for step in 0..=24 {
        let t = step as f32 * LOOP / 24.;
        assert!((0.4 - 1e-5..=1.6 + 1e-5).contains(&zoom(t)), "{t}");
    }
}

#[test]
fn the_fields_middle_always_reads_the_textures_middle() {
    for step in 0..=12 {
        let t = step as f32 * LOOP / 12.;
        assert_eq!(sample((0., 0.), t), (0., 0.));
    }
}

/// Unturned and at its widest, the field reads a point divided by its zoom.
#[test]
fn a_still_field_reads_the_point_it_covers() {
    let (x, y) = sample((0.25, -0.5), 0.);
    assert!(reads(x, 0.25 / zoom(0.)), "{x}");
    assert!(reads(y, wrapped(-0.5 / zoom(0.))), "{y}");
}

/// A quarter of a turn reads a point across what the field turned, shortened
/// by the zoom alone.
#[test]
fn a_quarter_turn_reads_the_point_across() {
    let quarter = LOOP / (4. * TURNS);
    let (x, y) = sample((0.5, 0.), quarter);
    assert!(reads(x, 0.), "{x}");
    assert!(reads(y, wrapped(-0.5 / zoom(quarter))), "{y}");
}

#[test]
fn the_field_repeats_with_the_loop() {
    for at in [(0.1, 0.2), (-0.4, 0.3), (0.5, -0.5)] {
        let (x, y) = sample(at, 1.7);
        let (later_x, later_y) = sample(at, 1.7 + LOOP);
        assert!(reads(x, later_x), "{at:?}");
        assert!(reads(y, later_y), "{at:?}");
    }
}

#[test]
fn the_texture_is_a_checker_with_a_bright_ring() {
    let mut shades = Vec::new();
    for across in 0..4 {
        for down in 0..4 {
            let uv = (across as f32 / 4., down as f32 / 4.);
            let shade = texel(uv);
            assert!((0. ..=1.).contains(&shade), "{uv:?}");
            shades.push(shade);
        }
    }
    // Neighbouring squares differ, and the ring is brighter than either.
    assert_ne!(shades[0], shades[1]);
    assert!(texel((0.5, 0.5)) > shades[0].max(shades[1]));
    assert_eq!(colour((0., 0.), &STOPS), ramp(&STOPS, texel((0., 0.))));
}
