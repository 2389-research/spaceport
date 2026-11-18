use super::*;

#[test]
fn the_card_passes_a_unit_once_its_edge_crosses_that_unit_s_middle() {
    // Four 20px units; the card is unit 1, resting at 20..40.
    let spans = [
        Some((0., 20.)),
        Some((20., 40.)),
        Some((40., 60.)),
        Some((60., 80.)),
    ];
    let at = |lift: f32| slot_for(&spans, 1, (20. + lift, 40. + lift));
    assert_eq!(at(0.), 1);
    // Up: the top edge must pass unit 0's middle at 10.
    assert_eq!(at(-9.), 1);
    assert_eq!(at(-11.), 0);
    // Down: the bottom edge must pass unit 2's middle at 50, then 3's at 70.
    assert_eq!(at(9.), 1);
    assert_eq!(at(11.), 3);
    assert_eq!(at(31.), 4);
    // A hidden unit is skipped rather than blocking.
    let hidden = [None, Some((20., 40.)), Some((40., 60.))];
    assert_eq!(slot_for(&hidden, 1, (0., 20.)), 1);
}

#[test]
fn a_slide_eases_to_its_target_and_turns_around_from_where_it_is() {
    let start = Instant::now();
    let duration = Duration::from_millis(100);
    let at = |ms| start + Duration::from_millis(ms);
    let slide = Slide {
        from: 0.,
        to: 40.,
        start,
    };
    assert_eq!(slide.at(start, duration), 0.);
    // Eased out: past half the distance at half the time.
    assert_eq!(slide.at(at(50), duration), 35.);
    assert_eq!(slide.at(at(100), duration), 40.);
    assert_eq!(slide.at(at(500), duration), 40.);
    // The same target keeps the slide going.
    assert_eq!(slide.toward(40., at(50), duration), slide);
    // A new target starts from where the row is.
    let back = slide.toward(0., at(50), duration);
    assert_eq!((back.from, back.to, back.start), (35., 0., at(50)));
    assert_eq!(Slide { start, ..slide }.at(at(50), Duration::ZERO), 40.);
}

#[test]
fn the_preview_closes_the_carried_place_and_opens_the_gap() {
    let heights = [10., 20., 30., 40.];
    // Over its own place, nothing moves.
    assert_eq!(preview(&heights, 1, 1), [0.; 4]);
    assert_eq!(preview(&heights, 1, 2), [0.; 4]);
    // Down past two units: they rise by its height, it drops by theirs.
    assert_eq!(preview(&heights, 1, 4), [0., 70., -20., -20.]);
    // Up past one: it rises by that unit's height, which moves down.
    assert_eq!(preview(&heights, 1, 0), [20., -10., 0., 0.]);
}
