use crate::{
    appear::{Appear, DIALOG_RISE, POPOVER_SLIDE, Shown, offset},
    page::Page,
};
use spaceport_motion::{ENTER, LEAVE};
use std::time::Instant;

#[test]
fn a_popover_slides_in_toward_its_anchor() {
    let start = Instant::now();
    let mut appear: Appear = Appear::default();
    appear.observe(Some(Shown::Popover { x: 10., y: 20. }), None, start, false);
    assert_eq!(appear.panel(start), 0.);
    assert_eq!(offset(false, appear.panel(start)), POPOVER_SLIDE);
    assert!(appear.moving(start));
    // A popover never dims the window.
    assert_eq!(appear.backdrop(start), 0.);
    let done = start + ENTER;
    assert_eq!(appear.panel(done), 1.);
    assert_eq!(offset(false, appear.panel(done)), 0.);
    assert!(!appear.moving(done));
}

#[test]
fn a_popover_opened_elsewhere_appears_again() {
    let start = Instant::now();
    let mut appear: Appear = Appear::default();
    appear.observe(Some(Shown::Popover { x: 10., y: 20. }), None, start, false);
    let later = start + ENTER;
    appear.observe(Some(Shown::Popover { x: 10., y: 20. }), None, later, false);
    assert_eq!(appear.panel(later), 1.);
    appear.observe(Some(Shown::Popover { x: 80., y: 20. }), None, later, false);
    assert_eq!(appear.panel(later), 0.);
}

#[test]
fn a_dialog_rises_over_dimming_that_lifts_after_it() {
    let start = Instant::now();
    let mut appear: Appear = Appear::default();
    appear.observe(Some(Shown::Dialog), None, start, false);
    assert_eq!(offset(true, appear.panel(start)), DIALOG_RISE);
    assert_eq!(appear.backdrop(start), 0.);
    let open = start + ENTER;
    // Another page of the same dialog, such as an Add Device source, stays put.
    appear.observe(Some(Shown::Dialog), None, open, false);
    assert_eq!(appear.panel(open), 1.);
    assert_eq!(appear.backdrop(open), 1.);
    appear.observe(None, None, open, false);
    assert_eq!(appear.backdrop(open), 1.);
    assert!(appear.moving(open));
    assert_eq!(appear.backdrop(open + LEAVE), 0.);
    assert!(!appear.moving(open + LEAVE));
}

#[test]
fn a_dialog_closed_midway_lifts_from_where_it_was() {
    let start = Instant::now();
    let mut appear: Appear = Appear::default();
    appear.observe(Some(Shown::Dialog), None, start, false);
    let midway = start + ENTER / 3;
    let shown = appear.backdrop(midway);
    appear.observe(None, None, midway, false);
    assert!((appear.backdrop(midway) - shown).abs() < 0.01);
}

#[test]
fn all_devices_sinks_away_after_it_closes() {
    let start = Instant::now();
    let mut appear: Appear = Appear::default();
    let anchor = Shown::Popover { x: 10., y: 600. };
    appear.observe(Some(anchor), Some(Page::Devices), start, false);
    let open = start + ENTER;
    appear.observe(Some(anchor), Some(Page::Devices), open, false);
    assert!(appear.leaving(open).is_none());
    appear.observe(None, None, open, false);
    assert_eq!(appear.leaving(open), Some((Page::Devices, 1.)));
    assert!(appear.moving(open));
    assert!(appear.leaving(open + LEAVE).is_none());
    assert!(!appear.moving(open + LEAVE));
    // Opening anything again ends it at once.
    appear.observe(None, None, open, false);
    appear.observe(Some(anchor), Some(Page::Devices), open, false);
    assert!(appear.leaving(open).is_none());
}

#[test]
fn a_session_form_closes_at_once_instead_of_the_picker_sinking() {
    let start = Instant::now();
    let mut appear: Appear = Appear::default();
    let anchor = Shown::Popover { x: 10., y: 600. };
    appear.observe(Some(anchor), Some(Page::Devices), start, false);
    // The form opens over the picker, then a click outside closes both.
    appear.observe(
        Some(Shown::Dialog),
        Some(Page::Devices),
        start + ENTER,
        false,
    );
    appear.observe(None, None, start + ENTER * 2, false);
    assert!(appear.leaving(start + ENTER * 2).is_none());
}

#[test]
fn a_closed_page_lets_go_of_what_it_read_once_it_has_gone() {
    let start = Instant::now();
    let mut appear: Appear = Appear::default();
    let anchor = Shown::Popover { x: 200., y: 40. };
    appear.observe(Some(anchor), Some(Page::Devices), start, false);
    appear.keep(Page::Devices, ());
    appear.observe(None, None, start, false);
    appear.settle(start);
    assert!(appear.kept.is_some(), "kept while it goes");
    appear.settle(start + LEAVE);
    assert!(appear.leaving.is_none());
    assert!(appear.kept.is_none(), "dropped once it has gone");
}

#[test]
fn other_menus_do_not_linger_after_closing() {
    let start = Instant::now();
    let mut appear: Appear = Appear::default();
    let anchor = Shown::Popover { x: 10., y: 600. };
    appear.observe(Some(anchor), Some(Page::Menu), start, false);
    appear.observe(None, None, start + ENTER, false);
    assert!(appear.leaving(start + ENTER).is_none());
}

#[test]
fn a_menu_observed_still_arrives_where_it_settles() {
    let start = Instant::now();
    let mut appear: Appear = Appear::default();
    appear.observe(Some(Shown::Dialog), None, start, true);
    assert_eq!(appear.panel(start), 1.);
    assert_eq!(appear.backdrop(start), 1.);
    assert!(!appear.moving(start));
    appear.observe(None, None, start, true);
    assert_eq!(appear.backdrop(start), 0.);
}
