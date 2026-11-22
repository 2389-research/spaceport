#![allow(clippy::unwrap_used)]
use super::*;

#[test]
fn a_toggle_opens_closes_and_starts_still() {
    let start = Instant::now();
    let mut panel = Toggle::default();
    panel.set(true, start, true);
    assert_eq!(panel.shown(start), 1.);
    assert!(!panel.moving(start));
    panel.set(false, start, false);
    assert_eq!(panel.shown(start), 1.);
    assert!(panel.moving(start));
    assert_eq!(panel.shown(start + LEAVE), 0.);
    assert!(!panel.moving(start + LEAVE));
    panel.set(true, start + LEAVE, false);
    assert_eq!(panel.shown(start + LEAVE), 0.);
    assert_eq!(panel.shown(start + LEAVE + ENTER), 1.);
    // Closing half-open continues from where it is.
    let mid = start + LEAVE + ENTER + ENTER;
    panel.set(false, mid, false);
    panel.set(true, mid, false);
    let shown = panel.shown(mid);
    assert!(shown > 0.9, "{shown}");
}
