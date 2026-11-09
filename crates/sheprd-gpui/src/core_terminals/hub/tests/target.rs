// ABOUTME: Checks that an exact Core view stays on its original runtime.
// ABOUTME: Prevents a later join or start request from retargeting it.

use super::*;
use std::sync::Condvar;

static EXACT_CALLS: Mutex<Vec<Start>> = Mutex::new(Vec::new());

fn record_join(start: Start) -> Result<CoreBackend, Error> {
    EXACT_CALLS.lock().unwrap().push(start);
    Err(Error::NotInstalled)
}

fn unavailable(_: Start) -> Result<CoreBackend, Error> {
    Err(Error::NotInstalled)
}

#[test]
fn an_exact_core_join_never_starts_a_replacement() {
    EXACT_CALLS.lock().unwrap().clear();
    let mut hub = Hub::with_join(record_join);
    hub.bind_exact("core-a").unwrap();
    hub.connect(Start::IfMissing);
    settle(&mut hub);
    assert_eq!(*EXACT_CALLS.lock().unwrap(), [Start::Join]);
}

#[test]
fn an_exact_core_join_rejects_a_different_core_id() {
    let mut hub = Hub::with_join(unavailable);
    hub.bind_exact("core-a").unwrap();
    assert!(hub.check_identity("core-a").is_ok());
    assert!(matches!(
        hub.check_identity("core-b"),
        Err(Error::WrongCore { expected, actual })
            if expected == "core-a" && actual == "core-b"
    ));
}

#[test]
fn an_exact_core_join_cannot_change_to_a_second_identity() {
    let mut hub = Hub::with_join(unavailable);
    hub.bind_exact("core-a").unwrap();
    assert!(matches!(
        hub.bind_exact("core-b"),
        Err(Error::WrongCore { expected, actual })
            if expected == "core-a" && actual == "core-b"
    ));
}

#[test]
fn identity_check_uses_the_latest_binding_after_clear_and_rebind() {
    let mut hub = Hub::with_join(unavailable);
    hub.bind_exact("core-a").unwrap();
    hub.connect(Start::Join);
    hub.clear_exact();
    hub.bind_exact("core-b").unwrap();
    assert!(matches!(
        hub.check_identity("core-a"),
        Err(Error::WrongCore { expected, actual })
            if expected == "core-b" && actual == "core-a"
    ));
}

static JOIN_GATE: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());
static JOIN_CALLS: Mutex<Vec<Start>> = Mutex::new(Vec::new());

fn held_join(start: Start) -> Result<CoreBackend, Error> {
    JOIN_CALLS.lock().unwrap().push(start);
    let (open, wake) = &JOIN_GATE;
    let mut open = open.lock().unwrap();
    while !*open {
        open = wake.wait(open).unwrap();
    }
    Err(Error::NotInstalled)
}

#[test]
fn binding_an_exact_core_cancels_a_queued_start() {
    JOIN_CALLS.lock().unwrap().clear();
    *JOIN_GATE.0.lock().unwrap() = false;
    let mut hub = Hub::with_join(held_join);
    hub.connect(Start::Join);
    hub.connect(Start::IfMissing);
    hub.bind_exact("core-a").unwrap();
    *JOIN_GATE.0.lock().unwrap() = true;
    JOIN_GATE.1.notify_all();
    settle(&mut hub);
    assert_eq!(*JOIN_CALLS.lock().unwrap(), [Start::Join]);
}

static START_GATE: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());

fn held_start(_: Start) -> Result<CoreBackend, Error> {
    let (open, wake) = &START_GATE;
    let mut open = open.lock().unwrap();
    while !*open {
        open = wake.wait(open).unwrap();
    }
    Err(Error::NotInstalled)
}

#[test]
fn binding_an_exact_core_refuses_a_start_already_in_flight() {
    *START_GATE.0.lock().unwrap() = false;
    let mut hub = Hub::with_join(held_start);
    hub.connect(Start::IfMissing);
    let bound = hub.bind_exact("core-a");
    *START_GATE.0.lock().unwrap() = true;
    START_GATE.1.notify_all();
    settle(&mut hub);
    assert!(bound.is_err());
}
