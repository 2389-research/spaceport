//! The greeting riding the sine wave across the stage.

use super::super::LOOP;
use super::super::scroller::{ADVANCE, LIFT, glyph, lift};

/// The line enters past the stage's end, crosses it, and leaves past its
/// start: no glyph is drawn outside the stage before it arrives or after it
/// has gone.
#[test]
fn the_line_crosses_the_stage_in_one_loop() {
    let count = 20;
    assert!((glyph(0, count, 0.).0 - 1.).abs() < 1e-6);
    let span = 1. + count as f32 * ADVANCE;
    let quarter = glyph(0, count, LOOP / 4.).0;
    assert!((quarter - (1. - span / 2.)).abs() < 1e-5, "{quarter}");
    let end = glyph(0, count, LOOP - 1e-3).0;
    assert!(end < 1. - span + 1e-2, "{end}");
}

/// Glyphs keep their order along the line and stand one advance apart.
#[test]
fn the_glyphs_stand_one_advance_apart() {
    let count = 20;
    for step in 0..8 {
        let t = step as f32 * LOOP / 8.;
        for index in 1..count {
            let apart = glyph(index, count, t).0 - glyph(index - 1, count, t).0;
            assert!((apart - ADVANCE).abs() < 1e-5, "{index} at {t}");
        }
    }
}

#[test]
fn every_glyph_is_lifted_by_the_wave_under_it() {
    assert!(lift(0.).abs() < 1e-6);
    assert!(lift(1.).abs() < 1e-5);
    for step in 0..=40 {
        let x = step as f32 / 40.;
        let rise = lift(x);
        assert!(rise.abs() <= LIFT + 1e-6, "{x}");
        assert!((glyph(3, 20, LOOP / 8.).1 - lift(glyph(3, 20, LOOP / 8.).0)).abs() < 1e-6);
    }
    // A whole number of waves crosses the stage, so its edges sit on the wave's
    // own level and the crest is where a quarter wave is.
    assert!(lift(1. / 10.) > 0.);
    assert!(lift(3. / 10.) < 0.);
}

#[test]
fn the_line_repeats_with_the_loop() {
    for index in [0, 4, 19] {
        let now = glyph(index, 20, 0.75);
        let later = glyph(index, 20, 0.75 + LOOP);
        assert!((now.0 - later.0).abs() < 1e-4, "{index}");
        assert!((now.1 - later.1).abs() < 1e-4, "{index}");
    }
}
