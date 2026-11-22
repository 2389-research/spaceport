// ABOUTME: Checks that scanner cancellation wakes and stays stopped.
// ABOUTME: Uses a real worker thread to exercise wakeup and latched cancellation.
#![allow(clippy::unwrap_used)]
use super::Stop;
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

#[test]
fn a_stopped_worker_wakes_at_once() {
    let stop = Arc::new(Stop::default());
    assert!(stop.wait(Instant::now()), "a due sample is taken");
    let waiter = {
        let stop = stop.clone();
        thread::spawn(move || stop.wait(Instant::now() + Duration::from_secs(3600)))
    };
    stop.stop();
    assert!(!waiter.join().unwrap());
    assert!(!stop.wait(Instant::now()), "stopped stays stopped");
}
