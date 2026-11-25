//! Stars projected out of a field that is the same on every launch.

use super::super::LOOP;
use super::super::starfield::{STARS, Sample, Star, project, scatter, stars};

#[test]
fn the_field_is_scattered_once_and_sits_on_its_own_plane() {
    assert_eq!(stars().len(), STARS);
    for star in stars() {
        assert!((-1. ..=1.).contains(&star.x), "{star:?}");
        assert!((-1. ..=1.).contains(&star.y), "{star:?}");
        assert!((0. ..1.).contains(&star.launch), "{star:?}");
    }
}

#[test]
fn the_same_star_is_scattered_to_the_same_place_every_time() {
    assert_eq!(scatter(7, 2), scatter(7, 2));
    assert_ne!(scatter(7, 2), scatter(8, 2));
    for index in 0..STARS {
        assert!((0. ..1.).contains(&scatter(index, 1)), "{index}");
    }
}

#[test]
fn a_star_at_the_far_plane_reads_its_own_place_and_burns_none() {
    let star = Star {
        x: 0.5,
        y: -0.5,
        launch: 0.,
    };
    let sample = project(&star, 0.).expect("a star at the far plane is on the field");
    assert_eq!(sample.at, (0.5, -0.5));
    assert_eq!(sample.brightness, 0.);
}

/// Depth shrinks as a star flies, so its point is thrown outward and brightens
/// as it comes nearer.
#[test]
fn a_star_is_thrown_outward_and_brightens_as_it_comes_nearer() {
    let star = Star {
        x: 0.25,
        y: -0.25,
        launch: 0.,
    };
    let mut last = project(&star, 0.).expect("on the field");
    for step in 1..=8 {
        let t = step as f32 * LOOP / 96.;
        let sample = project(&star, t).expect("still on the field");
        assert!(sample.at.0 > last.at.0, "{t}");
        assert!(sample.brightness > last.brightness, "{t}");
        last = sample;
    }
}

#[test]
fn a_star_leaves_the_field_once_it_has_flown_past_the_edge() {
    let star = Star {
        x: 1.,
        y: 0.,
        launch: 0.,
    };
    assert!(project(&star, 0.).is_some());
    let left = (1..=48)
        .map(|step| step as f32 * LOOP / 96.)
        .find(|t| project(&star, *t).is_none());
    assert!(left.is_some());
}

/// The loop's phase is all a star reads, so the field a loop on is the field
/// now: a star that was on the stage before is back where it was, as bright
/// as it was.
#[test]
fn the_field_repeats_with_the_loop() {
    let on_stage = |sample: &Sample| sample.at.0.abs() <= 1. && sample.at.1.abs() <= 1.;
    let (now, later) = stars()
        .iter()
        .find_map(|star| {
            let now = project(star, 0.5)?;
            let later = project(star, 0.5 + LOOP)?;
            (on_stage(&now) && on_stage(&later)).then_some((now, later))
        })
        .expect("a star is on the stage a whole loop apart");
    // The loop's phase repeats exactly; only the two times' own rounding
    // differs, so a star comes back to the same point to within a thousandth
    // of the stage's half-width and the same brightness to within a millionth.
    let close = |a: f32, b: f32, tolerance: f32| (a - b).abs() < tolerance;
    assert!(close(now.at.0, later.at.0, 1e-3), "{now:?} {later:?}");
    assert!(close(now.at.1, later.at.1, 1e-3), "{now:?} {later:?}");
    assert!(
        close(now.brightness, later.brightness, 1e-6),
        "{now:?} {later:?}"
    );
}
