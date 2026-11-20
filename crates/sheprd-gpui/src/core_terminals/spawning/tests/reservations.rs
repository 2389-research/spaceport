// ABOUTME: Checks the spawn FIFO releases Store reservations on every terminal path.
// ABOUTME: Keeps unknown runtime outcomes separate from client capacity ownership.
use super::*;
use crate::browser::Store;

fn available(store: &mut Store) -> usize {
    let mut held = Vec::new();
    while let Ok(reserved) = store.reserve() {
        held.push(reserved);
    }
    held.len()
}

#[test]
fn refused_and_unavailable_queues_release_their_reservations() {
    let mut store = Store::default();
    let total = available(&mut store);
    let mut spawning = Spawning::default();
    for _ in 0..CAPACITY {
        assert!(matches!(
            spawning.transition(Event::Queue(
                purpose(),
                Box::default(),
                None,
                Some(store.reserve().unwrap()),
                None
            )),
            Outcome::Accepted
        ));
    }
    let remaining = available(&mut store);
    assert_eq!(remaining, total - CAPACITY);
    assert!(matches!(
        spawning.transition(Event::Queue(
            purpose(),
            Box::default(),
            None,
            Some(store.reserve().unwrap()),
            None
        )),
        Outcome::Full
    ));
    assert_eq!(available(&mut store), remaining);
    assert!(matches!(
        spawning.transition(Event::Unavailable),
        Outcome::Cleared
    ));
    assert_eq!(available(&mut store), total);
}

#[cfg(unix)]
#[test]
fn stale_running_failed_and_dropped_requests_release_only_their_owned_slots() {
    fn fail(_: CoreBackend, _: Spawn, _: mpsc::SyncSender<Completion>) -> std::io::Result<()> {
        Err(std::io::Error::other("fixture worker refusal"))
    }
    let (backend, _peer) = crate::core_terminals::tests::scripted_core::connect("core-a");
    let mut store = Store::default();
    let total = available(&mut store);
    let mut spawning = Spawning::default();
    spawning.transition(Event::Queue(
        purpose(),
        Box::default(),
        Some("other".into()),
        Some(store.reserve().unwrap()),
        None,
    ));
    assert!(matches!(
        spawning.transition(Event::Start(&backend, &Infos::default(), fail)),
        Outcome::Stale
    ));
    assert_eq!(available(&mut store), total);
    spawning.transition(Event::Queue(
        purpose(),
        Box::default(),
        None,
        Some(store.reserve().unwrap()),
        None,
    ));
    spawning.transition(Event::Start(&backend, &Infos::default(), fail));
    spawning.transition(Event::Queue(
        purpose(),
        Box::default(),
        None,
        Some(store.reserve().unwrap()),
        None,
    ));
    assert_eq!(available(&mut store), total - 2);
    assert!(matches!(
        spawning.transition(Event::Unavailable),
        Outcome::Cleared
    ));
    assert_eq!(available(&mut store), total - 1);
    let finished = spawning.transition(Event::Poll);
    assert!(matches!(&finished, Outcome::Finished(_, result, _, _) if result.is_err()));
    assert_eq!(available(&mut store), total - 1);
    drop(finished);
    assert_eq!(available(&mut store), total);
    spawning.transition(Event::Queue(
        purpose(),
        Box::default(),
        None,
        Some(store.reserve().unwrap()),
        None,
    ));
    spawning.transition(Event::Start(&backend, &Infos::default(), fail));
    spawning.transition(Event::Queue(
        purpose(),
        Box::default(),
        None,
        Some(store.reserve().unwrap()),
        None,
    ));
    drop(spawning);
    assert_eq!(available(&mut store), total);
}
