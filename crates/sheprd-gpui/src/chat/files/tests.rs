// ABOUTME: Proves file work counts, cancellation, and exact completion ownership.
// ABOUTME: Exercises conversation fencing without sockets or a native desktop.
use super::*;
use crate::chat::{Route, Source, Surface};
use sheprd_chat_protocol::EndpointId;

fn target() -> Target {
    Target {
        pane_id: "fixture".into(),
        route: Route::Local,
        source: Source::Program(EndpointId::random()),
        surface: Surface::Herdr,
    }
}

fn cursor(epoch: u128, seq: u64) -> Option<Cursor> {
    Some(Cursor {
        epoch: uuid::Uuid::from_u128(epoch),
        seq,
    })
}

#[test]
fn concurrent_work_counts_files_and_finishes_each_operation_once() {
    let target = target();
    let mut work = FileWork::default();
    let dialog = work.begin(&target, cursor(1, 0), 0);
    let image = work.begin(&target, cursor(1, 0), 1);
    let upload = work.begin(&target, cursor(1, 0), 2);
    assert_eq!(work.pending(&target, cursor(1, 1)), 3);
    assert!(work.finish(&target, cursor(1, 1), &image));
    assert!(!work.finish(&target, cursor(1, 1), &image));
    assert_eq!(work.pending(&target, cursor(1, 1)), 2);
    assert!(work.finish(&target, cursor(1, 1), &upload));
    assert_eq!(work.pending(&target, cursor(1, 1)), 0);
    assert!(work.finish(&target, cursor(1, 1), &dialog));
    assert!(matches!(work.state, State::Idle));
}

#[test]
fn replacing_an_epoch_cancels_old_work_without_debiting_new_work() {
    let target = target();
    let mut work = FileWork::default();
    let old = work.begin(&target, cursor(1, 0), 4);
    let current = work.begin(&target, cursor(2, 0), 2);
    assert!(old.cancelled.load(Ordering::Acquire));
    assert!(!work.finish(&target, cursor(2, 0), &old));
    assert_eq!(work.pending(&target, cursor(2, 0)), 2);
    assert!(!current.cancelled.load(Ordering::Acquire));
    assert!(work.finish(&target, cursor(2, 0), &current));
}

#[test]
fn changing_a_target_or_losing_the_cursor_cancels_work() {
    let target = target();
    let mut work = FileWork::default();
    let old = work.begin(&target, cursor(1, 0), 1);
    let mut replacement = target.clone();
    replacement.route = Route::Ssh("other-host".into());
    work.synchronize(&replacement, cursor(1, 0));
    assert!(old.cancelled.load(Ordering::Acquire));
    assert!(!work.finish(&replacement, cursor(1, 0), &old));
    let old = work.begin(&replacement, cursor(1, 0), 1);
    assert_eq!(work.pending(&replacement, None), 0);
    work.synchronize(&replacement, None);
    assert!(old.cancelled.load(Ordering::Acquire));
}

#[test]
fn dropping_a_pane_cancels_its_worker() {
    let mut work = FileWork::default();
    let operation = work.begin(&target(), cursor(1, 0), 1);
    drop(work);
    assert!(operation.cancelled.load(Ordering::Acquire));
}

#[test]
fn an_old_ticket_cannot_finish_work_when_the_same_target_returns() {
    let target = target();
    let mut work = FileWork::default();
    let old = work.begin(&target, cursor(1, 0), 1);
    drop(work);
    work = FileWork::default();
    let current = work.begin(&target, cursor(1, 0), 2);
    assert!(!work.finish(&target, cursor(1, 0), &old));
    assert_eq!(work.pending(&target, cursor(1, 0)), 2);
    assert!(work.finish(&target, cursor(1, 0), &current));
}
