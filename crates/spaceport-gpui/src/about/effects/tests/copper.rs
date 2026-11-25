//! Copper bars riding across the stage.

use super::super::LOOP;
use super::super::copper::{BARS, WIDTH, colour, light, place};
use super::STOPS;

/// The bars keep their order — a bar never passes the one before it — and
/// the whole set stays on the stage.
#[test]
fn the_bars_keep_their_order_and_stay_on_the_stage() {
    for step in 0..=12 {
        let t = step as f32 * LOOP / 12.;
        for index in 1..BARS {
            assert!(
                place(index, t) > place(index - 1, t),
                "{index} at {t} passed the one before it"
            );
        }
        assert!(place(0, t) >= 0., "{t}");
        assert!(place(BARS - 1, t) + WIDTH <= 1., "{t}");
    }
}

#[test]
fn the_bars_swing_with_the_loop_and_repeat_with_it() {
    let quarter: Vec<f32> = (1..=8).map(|step| step as f32 * LOOP / 8.).collect();
    for index in 0..BARS {
        let now = place(index, 0.);
        assert!((place(index, LOOP) - now).abs() < 1e-4, "{index} repeats");
        assert!(
            quarter
                .iter()
                .any(|t| (place(index, *t) - now).abs() > 1e-3),
            "{index} swings"
        );
    }
}

#[test]
fn a_bar_is_lit_down_its_middle_and_dark_at_its_edges() {
    assert!(light(0.).abs() < 1e-6);
    assert!(light(1.).abs() < 1e-6);
    assert!((light(0.5) - 1.).abs() < 1e-6);
    assert!(light(0.25) > light(0.) && light(0.25) < light(0.5));
}

#[test]
fn a_bars_colour_runs_through_its_ramp() {
    assert_eq!(colour(0., &STOPS), STOPS[0]);
    assert_eq!(colour(0.5, &STOPS), STOPS[2]);
    assert_eq!(colour(1., &STOPS), STOPS[0]);
}
