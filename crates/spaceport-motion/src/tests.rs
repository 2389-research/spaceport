use super::*;

#[test]
fn slow_time_passes_a_tenth_as_fast_from_when_it_was_turned_on() {
    let on = Instant::now();
    let later = on + Duration::from_secs(10);
    assert_eq!(at(later, None), later);
    assert_eq!(at(later, Some(on)), on + Duration::from_secs(1));
    assert_eq!(at(on, Some(on)), on, "no jump when it turns on");
    // Time read before the switch stays where it was.
    let earlier = on.checked_sub(Duration::from_secs(1)).unwrap_or(on);
    assert_eq!(at(earlier, Some(on)), on);
}

#[test]
fn slow_animations_last_ten_times_longer() {
    let length = Duration::from_millis(180);
    assert_eq!(stretch(length, false), length);
    assert_eq!(stretch(length, true), Duration::from_millis(1800));
}

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
