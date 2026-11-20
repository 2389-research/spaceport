// ABOUTME: Verifies departure data releases when no popup frame ever became visible.
// ABOUTME: Covers first-frame cancellation without relying on animation timing.
use super::*;

#[test]
fn closing_before_the_first_frame_releases_captured_paint() {
    let mut appear = live();
    let now = motion::now();
    appear.keep(Page::Menu, Kept::default());
    appear.observe(None, None, now);
    appear.settle(now);
    assert!(appear.leaving(now).is_none());
    assert!(!appear.moving(now));
    assert!(
        appear.kept.is_none(),
        "an invisible popup retained its paint"
    );
}

#[test]
fn replacing_a_popup_before_the_next_frame_releases_its_captured_paint() {
    let mut appear = live();
    let now = motion::now();
    let anchor = Some(Shown::Popover(point(px(20.), px(30.))));
    appear.observe(anchor, Some(Page::Menu), now);
    // Dismiss and reopen at the same anchor before another frame can observe closure.
    appear.keep(Page::Menu, Kept::default());
    appear.observe(anchor, Some(Page::Group), now + ENTER);
    appear.settle(now + ENTER);
    assert!(appear.leaving(now + ENTER).is_none());
    assert!(
        appear.kept.is_none(),
        "the replacement retained the closed popup's paint"
    );
}
