// ABOUTME: Keeps start permission within the connection attempt that requested it.
// ABOUTME: Covers successful joins, disconnections, and worker creation failures.

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(unix)]
#[test]
fn a_successful_join_discards_its_pending_start_request() {
    use crate::core_terminals::tests::scripted_core;
    static CALLS: Mutex<Vec<Start>> = Mutex::new(Vec::new());
    fn fail_join(start: Start) -> Result<CoreBackend, Error> {
        CALLS.lock().unwrap().push(start);
        Err(Error::NotInstalled)
    }
    let mut hub = opted_in_hub(fail_join);
    let (send, receive) = std::sync::mpsc::channel();
    hub.transition(Event::SetState(State::Connecting {
        outcome: receive,
        start: Start::Join,
        upgrade: false,
    }));
    hub.connect(Start::IfMissing);
    let (backend, peer) = scripted_core::connect("core-start-intent");
    send.send(Ok(backend.clone())).unwrap();
    assert!(hub.poll());
    assert!(hub.backend().is_some());

    scripted_core::lose(&backend, peer);
    assert!(hub.poll());
    hub.connect(Start::Join);
    settle(&mut hub);
    assert_eq!(*CALLS.lock().unwrap(), [Start::Join]);
}

#[test]
fn a_worker_creation_failure_waits_before_retrying() {
    static SPAWNS: AtomicUsize = AtomicUsize::new(0);
    fn cannot_spawn(_: Join, _: Start) -> std::io::Result<Attempt> {
        SPAWNS.fetch_add(1, Ordering::SeqCst);
        Err(std::io::Error::other("fixture worker creation failed"))
    }
    fn unused_join(_: Start) -> Result<CoreBackend, Error> {
        panic!("a failed worker must never run the connector");
    }
    let mut hub = opted_in_hub(unused_join);
    hub.spawn = cannot_spawn;
    hub.connect(Start::Join);
    assert!(
        matches!(hub.state(), State::Unavailable { error, .. } if matches!(**error, Error::Worker(_)))
    );
    hub.connect(Start::Join);
    assert_eq!(SPAWNS.load(Ordering::SeqCst), 1);
    hub.connect(Start::IfMissing);
    assert_eq!(SPAWNS.load(Ordering::SeqCst), 2);
}

#[test]
fn a_dropped_worker_discards_its_pending_start_request() {
    static CALLS: Mutex<Vec<Start>> = Mutex::new(Vec::new());
    fn fail_join(start: Start) -> Result<CoreBackend, Error> {
        CALLS.lock().unwrap().push(start);
        Err(Error::NotInstalled)
    }
    let mut hub = opted_in_hub(fail_join);
    let (send, outcome) = std::sync::mpsc::channel();
    hub.transition(Event::SetState(State::Connecting {
        outcome,
        start: Start::Join,
        upgrade: false,
    }));
    hub.connect(Start::IfMissing);
    assert!(!hub.poll());
    drop(send);
    assert!(hub.poll());
    assert!(matches!(hub.state(), State::Idle));
    hub.connect(Start::Join);
    settle(&mut hub);
    assert_eq!(*CALLS.lock().unwrap(), [Start::Join]);
}

#[test]
fn a_join_retries_after_the_failure_delay() {
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    fn fail_join(_: Start) -> Result<CoreBackend, Error> {
        CALLS.fetch_add(1, Ordering::SeqCst);
        Err(Error::NotInstalled)
    }
    let mut hub = opted_in_hub(fail_join);
    hub.transition(Event::SetState(State::Unavailable {
        error: Arc::new(Error::NotInstalled),
        at: Instant::now() - JOIN_RETRY,
    }));
    hub.connect(Start::Join);
    settle(&mut hub);
    assert_eq!(CALLS.load(Ordering::SeqCst), 1);
}

#[cfg(unix)]
#[test]
fn a_join_outcome_must_match_the_latest_exact_binding() {
    use crate::core_terminals::tests::scripted_core;
    fn unused_join(_: Start) -> Result<CoreBackend, Error> {
        panic!("a completed private attempt must not dial again");
    }
    let mut hub = opted_in_hub(unused_join);
    let (send, outcome) = std::sync::mpsc::channel();
    hub.transition(Event::SetState(State::Connecting {
        outcome,
        start: Start::Join,
        upgrade: false,
    }));
    hub.bind_exact("core-a").unwrap();
    hub.clear_exact();
    hub.bind_exact("core-b").unwrap();
    let (backend, peer) = scripted_core::connect("core-a");
    send.send(Ok(backend)).unwrap();
    assert!(hub.poll());
    assert!(matches!(
        hub.state(),
        State::Unavailable { error, .. }
            if matches!(error.as_ref(), Error::WrongCore { expected, actual }
                if expected == "core-b" && actual == "core-a")
    ));
    drop(peer);
}
