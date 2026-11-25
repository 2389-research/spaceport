// ABOUTME: Exercises the cell map a daemon's frame rectangles make.
// ABOUTME: Pins exact tiling: every cell claimed once, and nothing partial kept.
use super::*;

fn rect(x: u16, y: u16, width: u16, height: u16) -> CellRect {
    CellRect {
        x,
        y,
        width,
        height,
    }
}

#[test]
fn a_frame_maps_each_pane_to_its_cells() {
    let map = CellMap::new(
        8,
        3,
        [(id("t1"), rect(0, 0, 5, 3)), (id("t2"), rect(5, 0, 3, 3))],
    )
    .unwrap();
    assert_eq!((map.columns(), map.rows()), (8, 3));
    assert_eq!(
        map.panes(),
        [(id("t1"), rect(0, 0, 5, 3)), (id("t2"), rect(5, 0, 3, 3))]
    );
}

#[test]
fn a_frame_map_keeps_only_layouts_that_claim_every_cell_once() {
    // The same two panes: a cell short on the right, one over, and a gap.
    assert!(CellMap::new(8, 3, [(id("t1"), rect(0, 0, 5, 3))]).is_none());
    assert!(
        CellMap::new(
            8,
            3,
            [(id("t1"), rect(0, 0, 5, 3)), (id("t2"), rect(4, 0, 4, 3))]
        )
        .is_none()
    );
    assert!(
        CellMap::new(
            9,
            3,
            [(id("t1"), rect(0, 0, 5, 3)), (id("t2"), rect(5, 0, 3, 3))]
        )
        .is_none()
    );
    assert!(
        CellMap::new(
            8,
            3,
            [(id("t1"), rect(0, 0, 5, 3)), (id("t2"), rect(5, 0, 4, 3))]
        )
        .is_none()
    );
}

#[test]
fn a_frame_map_refuses_rectangles_outside_it_or_with_no_area() {
    for bad in [
        rect(0, 0, 0, 3),
        rect(0, 0, 8, 0),
        rect(8, 0, 1, 1),
        rect(0, 3, 1, 1),
    ] {
        assert!(CellMap::new(8, 3, [(id("t1"), bad)]).is_none());
    }
    // A pane is never wider or taller than the frame it sits in.
    assert!(CellMap::new(8, 3, [(id("t1"), rect(0, 0, u16::MAX, 3))]).is_none());
}

#[test]
fn a_frame_map_names_each_pane_once() {
    // Two rectangles that tile the frame, both reported for the same pane.
    assert!(
        CellMap::new(
            8,
            3,
            [(id("t1"), rect(0, 0, 5, 3)), (id("t1"), rect(5, 0, 3, 3))]
        )
        .is_none()
    );
}

#[test]
fn a_frame_map_needs_a_pane_and_a_cell() {
    assert!(CellMap::new(8, 3, []).is_none());
    assert!(CellMap::new(0, 3, []).is_none());
    assert!(CellMap::new(8, 0, [(id("t1"), rect(0, 0, 1, 1))]).is_none());
}
