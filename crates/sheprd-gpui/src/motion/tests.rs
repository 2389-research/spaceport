// ABOUTME: Checks manual motion timing and reversible transitions.
// ABOUTME: Keeps deterministic animation tests separate from drawing code.
use super::*;

#[test]
fn progress_eases_out_and_ends() {
    let start = Instant::now();
    assert_eq!(progress(start, start, ENTER), Some(0.));
    let half = progress(start, start + ENTER / 2, ENTER).unwrap_or_default();
    assert!(half > 0.8 && half < 1., "{half}");
    assert_eq!(progress(start, start + ENTER, ENTER), None);
    // A start still ahead reads as not begun.
    assert_eq!(progress(start + ENTER, start, ENTER), Some(0.));
}

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

mod reduced_motion;
