//! Where a card lands and what the cache keeps: the grid's geometry, its
//! fence against a listing it no longer draws, and its bound.
use super::*;
use core::prelude::v1::test;

fn choice(name: &str) -> Choice {
    Choice {
        scope: spaceport_settings::theme::Scope::App,
        name: name.into(),
    }
}

#[test]
fn a_width_holds_between_two_and_six_cards() {
    assert_eq!(columns(680. - 240.), 2);
    assert_eq!(columns(960. - 240.), 4);
    assert_eq!(columns(4000.), 6);
    assert_eq!(columns(10.), 2);
}

#[test]
fn a_step_moves_a_row_and_stops_at_the_ends() {
    assert_eq!(navigate(5, 400, 4, false), 9);
    assert_eq!(navigate(5, 400, 4, true), 1);
    assert_eq!(navigate(398, 400, 4, false), 399);
    assert_eq!(navigate(2, 400, 4, true), 0);
}

#[test]
fn palettes_loaded_for_a_listing_the_grid_dropped_are_not_drawn() {
    let mut grid = Grid::default();
    let old = grid.epoch;
    grid.invalidate();
    grid.finish(old, vec![(choice("old"), Some(Theme::default()))]);
    assert!(grid.palettes.is_empty());
    assert!(!grid.running);
    grid.finish(grid.epoch, vec![(choice("kept"), None)]);
    assert_eq!(grid.palette(&choice("kept")), Some(&None));
    assert_eq!(grid.palette(&choice("absent")), None);
}

#[test]
fn the_cache_keeps_the_most_recent_palettes_and_no_more() {
    let mut grid = Grid::default();
    let epoch = grid.epoch;
    for index in 0..CACHE_LIMIT * 4 {
        grid.finish(epoch, vec![(choice(&index.to_string()), None)]);
    }
    assert_eq!(grid.palettes.len(), CACHE_LIMIT);
    assert_eq!(
        grid.palettes
            .front()
            .map(|(choice, _)| choice.name.as_str()),
        Some("192")
    );
}

#[test]
fn only_the_cards_on_screen_owe_a_palette() {
    let mut grid = Grid::default();
    let listing: Vec<_> = ["Nord", "Dracula", "Vesper"].map(choice).into();
    grid.visible = 0..2;
    assert!(!grid.covers(&listing));
    let epoch = grid.epoch;
    grid.finish(
        epoch,
        vec![(choice("Nord"), None), (choice("Dracula"), None)],
    );
    assert!(grid.covers(&listing), "the third card is off screen");
    grid.visible = 0..3;
    assert!(!grid.covers(&listing));
}
