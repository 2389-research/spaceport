//! The field the whole stage sits in.

use super::super::LOOP;
use super::super::plasma::{colour, field};
use super::STOPS;

/// The field never leaves the range its palette is read over, at any time
/// and anywhere on the stage.
#[test]
fn the_field_stays_between_its_darkest_and_its_brightest() {
    for step in 0..=12 {
        let t = step as f32 * LOOP / 12.;
        for across in 0..=8 {
            for down in 0..=8 {
                let at = (across as f32 / 8., down as f32 / 8.);
                let v = field(at, t);
                assert!((0. ..=1.).contains(&v), "{v} at {at:?} {t}");
            }
        }
    }
}

/// Every time term is a whole number of turns in a loop, so the field is
/// back where it started one loop later and has moved a quarter of the way
/// through it.
#[test]
fn the_field_repeats_with_the_loop() {
    for at in [(0., 0.), (0.25, 0.75), (1., 1.), (0.4, 0.1)] {
        let now = field(at, 1.25);
        assert!(
            (field(at, 1.25 + LOOP) - now).abs() < 1e-4,
            "{at:?} repeats"
        );
        assert!(
            (field(at, 1.25 + LOOP / 4.) - now).abs() > 1e-3,
            "{at:?} moves"
        );
    }
}

#[test]
fn a_cell_takes_its_colour_from_the_field() {
    assert_eq!(colour(0., &STOPS), STOPS[0]);
    assert_eq!(colour(0.5, &STOPS), STOPS[1]);
    assert_eq!(colour(1., &STOPS), STOPS[2]);
}
