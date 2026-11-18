// ABOUTME: Checks bounded warning retention, shared text, and paused display deadlines.
// ABOUTME: Preserves connection priority without losing later failure explanations.
#![allow(clippy::unwrap_used)]
use super::*;

#[test]
fn history_is_bounded_and_survives_clear_and_expiry() {
    let mut notices = Notices::default();
    for index in 0..HISTORY_LIMIT + 5 {
        notices.show(Cause::Command, format!("warning {index}"));
    }
    let latest = notices.history().next_back().unwrap();
    assert!(Arc::ptr_eq(
        &latest.text,
        &notices.current.as_ref().unwrap().text
    ));
    notices.expire(Instant::now() + OPERATION_DURATION * 2);
    notices.clear();
    assert_eq!(notices.history().count(), HISTORY_LIMIT);
    assert_eq!(notices.history().next().unwrap().text.as_ref(), "warning 5");
    assert!(notices.text().is_none());
}

#[test]
fn history_keeps_refused_messages_without_overwriting_connection_reason() {
    let mut notices = Notices::default();
    notices.show(Cause::Connection, "Connection closed".into());
    notices.show(Cause::Command, "Could not open tab".into());
    notices.record(Cause::Command, "Could not copy".into());
    assert_eq!(notices.connection_reason(), Some("Connection closed"));
    assert_eq!(notices.history().count(), 3);
    assert_eq!(
        notices.history().next_back().unwrap().text.as_ref(),
        "Could not copy"
    );
}

#[test]
fn notice_text_stays_bounded_at_a_utf8_boundary() {
    let mut notices = Notices::default();
    notices.show(Cause::Command, "🛰".repeat(TEXT_LIMIT));
    let text = notices.text().unwrap();
    assert!(text.len() <= TEXT_LIMIT);
    assert!(text.ends_with('…'));
    assert_eq!(notices.history().next_back().unwrap().text.as_ref(), text);
}

#[test]
fn paused_notice_resumes_with_its_remaining_reading_time() {
    let mut notices = Notices::default();
    let start = Instant::now();
    notices.transition(Transition::Show {
        cause: Cause::Command,
        text: "Failed".into(),
        now: start,
    });
    notices.pause(true, start + Duration::from_secs(2));
    assert!(!notices.expire(start + Duration::from_secs(100)));
    assert_eq!(
        notices.text_at(start + Duration::from_secs(100)),
        Some("Failed")
    );
    notices.pause(false, start + Duration::from_secs(102));
    assert_eq!(
        notices.text_at(start + Duration::from_secs(109)),
        Some("Failed")
    );
    assert!(notices.expire(start + Duration::from_secs(110)));
}

#[test]
fn held_input_restoration_preserves_a_paused_notice() {
    let mut notices = Notices::default();
    let start = Instant::now();
    notices.show(Cause::Input, "Input refused".into());
    notices.pause(true, start);
    let held = notices.take();
    notices.restore(held);
    assert!(!notices.expire(start + Duration::from_secs(100)));
    assert_eq!(
        notices.text_at(start + Duration::from_secs(100)),
        Some("Input refused")
    );
}
