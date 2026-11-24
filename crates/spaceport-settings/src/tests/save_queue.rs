//! The bounded queue behind Settings' saves: what waits behind a running
//! write, what a full queue refuses, and what a close or quit still owes.

use crate::Config;
use crate::save_queue::{Kind, Lifecycle, Operation, QUEUE_LIMIT, Request};

/// A queued write that loads nothing and cannot fail.
fn request() -> Request<(), ()> {
    Request::new(Operation::plain(|| Ok(())), Box::new(|| Ok(())), false)
}

#[test]
fn pending_writes_are_bounded_and_a_full_queue_is_refused() {
    let mut queue: Lifecycle<Request<(), ()>> = Lifecycle::default();
    queue.begin_save(Kind::Ordinary);
    for queued in 0..QUEUE_LIMIT {
        assert!(queue.enqueue(request()), "write {queued} waits");
    }
    assert!(!queue.enqueue(request()), "a full queue refuses the next");
    assert!(queue.busy());
    assert_eq!(queue.take_pending().len(), QUEUE_LIMIT);
}

#[test]
fn a_quit_drains_what_a_running_write_left_pending() {
    let mut queue: Lifecycle<Request<(), ()>> = Lifecycle::default();
    queue.begin_save(Kind::Ordinary);
    queue.enqueue(request());
    queue.quit();
    assert!(queue.quitting());
    assert!(queue.busy(), "a quit waits for the running write");
    assert!(
        queue.take_next().is_none(),
        "no queued write starts during a quit"
    );
    assert_eq!(queue.take_pending().len(), 1);
    assert!(matches!(queue.finish_save(), Kind::Ordinary));
}

#[test]
fn a_queued_write_starts_once_the_running_one_finishes() {
    let mut queue: Lifecycle<Request<(), ()>> = Lifecycle::default();
    queue.begin_save(Kind::Ordinary);
    queue.enqueue(request());
    assert!(queue.take_next().is_none());
    assert!(matches!(queue.finish_save(), Kind::Ordinary));
    assert!(!queue.busy());
    assert!(queue.take_next().is_some());
    assert!(queue.take_next().is_none());
}

#[test]
fn a_close_waits_for_the_running_write_and_is_taken_once() {
    let mut queue: Lifecycle<Request<(), ()>> = Lifecycle::default();
    queue.begin_save(Kind::Ordinary);
    queue.close();
    assert!(queue.closing());
    assert!(
        !queue.enqueue(request()),
        "a window on its way out queues nothing"
    );
    assert!(matches!(queue.finish_save(), Kind::Ordinary));
    assert!(queue.take_close(), "the close the window asked for");
    assert!(!queue.take_close(), "it is taken once");
    assert!(!queue.closing());
}

#[test]
fn a_failed_save_leaves_the_editing_goal_and_drops_the_queued_sidebar() {
    let mut queue: Lifecycle<Request<(), ()>> = Lifecycle::default();
    let intent = crate::sidebar_draft::SidebarIntent::of(&Config::default());
    queue.queue_sidebar(Some(intent));
    assert!(queue.sidebar_queued());
    queue.fail();
    assert!(!queue.sidebar_queued(), "a failed write drops its draft");
}

#[test]
fn a_quit_keeps_the_running_kind_for_shutdown_to_compare() {
    let mut queue: Lifecycle<Request<(), ()>> = Lifecycle::default();
    queue.begin_save(Kind::Theme(3));
    queue.quit();
    assert!(matches!(queue.kind(), Some(Kind::Theme(3))));
    queue.abort_drain();
    assert!(queue.quitting(), "an aborted drain keeps the quit");
}
