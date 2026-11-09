#![allow(clippy::unwrap_used)]
use super::*;
use core::prelude::v1::test;

fn bounds(left: f32, width: f32) -> Bounds<Pixels> {
    Bounds::new(point(px(left), px(0.)), size(px(width), px(600.)))
}

#[test]
fn dragging_an_edge_follows_the_pointer_from_the_panel_s_far_side() {
    let mut notes = PanelWidth::new(300., 200.);
    assert_eq!(notes.width(0.), 300.);
    // A right-hand panel at 900..1200 dragged by its left edge to 800.
    assert!(notes.drag(Side::Right, bounds(900., 300.), px(800.)));
    assert_eq!(notes.width(0.), 400.);
    assert!(
        !notes.drag(Side::Right, bounds(800., 400.), px(800.)),
        "unchanged"
    );
    // Never narrower than its minimum.
    notes.drag(Side::Right, bounds(800., 400.), px(1150.));
    assert_eq!(notes.width(0.), 200.);
    // A left-hand panel grows to the right.
    let mut navigation = PanelWidth::new(184., 140.);
    navigation.drag(Side::Left, bounds(0., 184.), px(260.));
    assert_eq!(navigation.width(0.), 260.);
    navigation.reset();
    assert_eq!(navigation.width(0.), 184.);
}

#[test]
fn a_panel_keeps_to_its_share_of_a_narrow_window() {
    let mut notes = PanelWidth::new(300., 200.);
    notes.drag(Side::Right, bounds(0., 300.), px(-600.));
    assert_eq!(notes.width(2000.), 900.);
    assert_eq!(notes.width(1000.), 600.);
    // The minimum holds while the window has room for it.
    assert_eq!(notes.width(300.), 200.);
    assert_eq!(notes.width(150.), 150.);
}
