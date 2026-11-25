//! The stops a layer's palette is mixed from.

use super::super::ramp::{mix, ramp};
use super::STOPS;

#[test]
fn a_ramp_ends_on_its_own_stops() {
    assert_eq!(ramp(&STOPS, 0.), STOPS[0]);
    assert_eq!(ramp(&STOPS, 0.5), STOPS[1]);
    assert_eq!(ramp(&STOPS, 1.), STOPS[2]);
}

#[test]
fn a_ramp_is_clamped_at_either_end() {
    assert_eq!(ramp(&STOPS, -2.), STOPS[0]);
    assert_eq!(ramp(&STOPS, 2.), STOPS[2]);
}

/// These stops climb in red, so a ramp between them climbs too: a layer's
/// palette never folds back on itself.
#[test]
fn a_ramp_climbs_between_its_stops() {
    let red = |colour: u32| (colour >> 16) & 0xff;
    let mut last = 0;
    for step in 0..=20 {
        let colour = ramp(&STOPS, step as f32 / 20.);
        assert!(red(colour) >= last, "{colour:#08x} at {step}");
        last = red(colour);
    }
}

#[test]
fn a_mix_runs_from_one_colour_to_the_other() {
    assert_eq!(mix(0x123456, 0xabcdef, 0.), 0x123456);
    assert_eq!(mix(0x123456, 0xabcdef, 1.), 0xabcdef);
    assert_eq!(mix(0x000000, 0xffffff, 0.5), 0x808080);
    assert_eq!(mix(0x000000, 0xffffff, 3.), 0xffffff);
}
