// ABOUTME: Checks notice expiry, connection priority, and held-input restoration fences.
// ABOUTME: Uses explicit clock values to cover deadlines without sleeps.
use super::*;

#[test]
fn operation_notice_expires_at_its_deadline() {
    let now = Instant::now();
    let mut notices = Notices::default();
    notices.transition(Transition::Show {
        cause: Cause::Transfer,
        text: "files busy".into(),
        now,
    });
    assert_eq!(
        notices.text_at(now + OPERATION_DURATION - Duration::from_nanos(1)),
        Some("files busy")
    );
    assert_eq!(notices.text_at(now + OPERATION_DURATION), None);
    assert!(notices.expire(now + OPERATION_DURATION));
    assert!(!notices.expire(now + OPERATION_DURATION));
}

#[test]
fn connection_reason_outlives_operation_deadlines_and_late_failures() {
    let now = Instant::now();
    let mut notices = Notices::default();
    notices.show(Cause::Connection, "socket closed".into());
    notices.show(Cause::Daemon, "late refusal".into());
    assert_eq!(
        notices.text_at(now + OPERATION_DURATION * 100),
        Some("socket closed")
    );
    assert!(!notices.expire(now + OPERATION_DURATION * 100));
    notices.clear();
    notices.show(Cause::Input, "input failed".into());
    assert!(notices.connection_reason().is_none());
}

#[test]
fn held_notice_keeps_its_cause_and_deadline() {
    let mut notices = Notices::default();
    notices.show(Cause::Transfer, "copy refused".into());
    let before = notices.current.clone();
    let held = notices.take();
    assert!(notices.is_none());
    notices.restore(held);
    assert_eq!(notices.current, before);
}

#[test]
fn held_notice_cannot_restore_after_a_clear_or_newer_notice() {
    for clear in [false, true] {
        let mut notices = Notices::default();
        notices.show(Cause::Input, "old".into());
        let held = notices.take();
        if clear {
            notices.clear();
        } else {
            notices.show(Cause::Command, "new".into());
        }
        notices.restore(held);
        assert_eq!(notices.text(), if clear { None } else { Some("new") });
    }
}

#[test]
fn expired_held_notice_cannot_return() {
    let mut notices = Notices::default();
    notices.transition(Transition::Show {
        cause: Cause::Daemon,
        text: "old".into(),
        now: Instant::now() - OPERATION_DURATION,
    });
    let held = notices.take();
    notices.restore(held);
    assert!(notices.current.is_none());
}
