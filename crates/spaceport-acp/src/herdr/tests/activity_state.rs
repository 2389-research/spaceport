// ABOUTME: Tests activity delivery transitions without sockets or threads.
// ABOUTME: Covers acknowledgement, retry retention, and latest-value coalescing.

use super::*;

#[test]
fn only_an_attempted_report_needs_withdrawal() {
    let mut report = ActivityReport::default();
    assert!(!report.was_attempted());
    assert!(!report.is_queued());
    assert_eq!(report.transition(Event::Begin), None);
    report.transition(Event::Queue(Activity::Working));
    report.transition(Event::Queue(Activity::Idle));
    assert!(report.is_queued());
    assert!(!report.was_attempted());
    assert_eq!(report.transition(Event::Begin), Some(Activity::Idle));
    assert!(report.was_attempted());
}

#[test]
fn acknowledgement_stops_retries_until_another_update() {
    let mut report = ActivityReport::default();
    report.transition(Event::Queue(Activity::Idle));
    report.transition(Event::Begin);
    report.transition(Event::Complete {
        acknowledged: false,
    });
    assert!(report.needs_retry());
    assert!(!report.is_queued());
    assert_eq!(report.transition(Event::Begin), Some(Activity::Idle));
    report.transition(Event::Complete { acknowledged: true });
    assert!(!report.needs_retry());
    assert_eq!(report.transition(Event::Begin), None);

    report.transition(Event::Queue(Activity::Working));
    assert!(report.was_attempted());
    assert_eq!(report.transition(Event::Begin), Some(Activity::Working));
}

#[test]
fn completion_keeps_only_the_latest_in_flight_update() {
    for acknowledged in [false, true] {
        let mut report = ActivityReport::default();
        report.transition(Event::Queue(Activity::Working));
        report.transition(Event::Begin);
        report.transition(Event::Queue(Activity::Blocked));
        report.transition(Event::Queue(Activity::Idle));
        report.transition(Event::Complete { acknowledged });
        assert!(report.is_queued());
        assert!(report.was_attempted());
        assert_eq!(report.transition(Event::Begin), Some(Activity::Idle));
    }
}
