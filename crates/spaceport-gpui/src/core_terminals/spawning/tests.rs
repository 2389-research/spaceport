// ABOUTME: Pins spawn queue bounds, target fencing, and uncertain completion semantics.
// ABOUTME: Coordinates real private Core connections without daemon lifecycle side effects.
use super::*;
use crate::browser::Scope;

fn purpose() -> Purpose {
    Purpose::Tab {
        scope: Scope::endpoint("fixture"),
        workspace: "w".into(),
    }
}

#[test]
fn a_full_queue_refuses_only_the_new_request() {
    let mut spawning = Spawning::default();
    for _ in 0..CAPACITY {
        assert!(matches!(
            spawning.transition(Event::Queue(purpose(), Box::default(), None, None)),
            Outcome::Accepted
        ));
    }
    assert!(matches!(
        spawning.transition(Event::Queue(purpose(), Box::default(), None, None)),
        Outcome::Full
    ));
    assert!(matches!(&spawning, Spawning::Queued(queued) if queued.len() == CAPACITY));
    assert!(matches!(
        spawning.transition(Event::Unavailable),
        Outcome::Cleared
    ));
    assert!(matches!(spawning, Spawning::Idle));
    assert!(matches!(
        spawning.transition(Event::Unavailable),
        Outcome::None
    ));
}

#[cfg(unix)]
#[test]
fn queued_work_cannot_move_to_another_core() {
    let (backend, _peer) = crate::core_terminals::tests::scripted_core::connect("core-b");
    let mut spawning = Spawning::default();
    spawning.transition(Event::Queue(
        purpose(),
        Box::default(),
        Some("core-a".into()),
        None,
    ));
    assert!(matches!(
        spawning.transition(Event::Start(&backend, &Infos::default(), spawn_worker)),
        Outcome::Stale
    ));
    assert!(matches!(spawning, Spawning::Idle));
}

#[cfg(unix)]
#[test]
fn connection_failure_discards_waiting_work_but_consumes_the_running_request_once() {
    let (backend, peer) = crate::core_terminals::tests::scripted_core::connect("core-a");
    let mut spawning = Spawning::default();
    let infos = Infos::default();
    spawning.transition(Event::Queue(purpose(), Box::default(), None, None));
    spawning.transition(Event::Start(&backend, &infos, spawn_worker));
    spawning.transition(Event::Queue(purpose(), Box::default(), None, None));
    assert!(matches!(
        spawning.transition(Event::Unavailable),
        Outcome::Cleared
    ));
    assert!(matches!(&spawning, Spawning::Running { queued, .. } if queued.is_empty()));
    drop(peer);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        match spawning.transition(Event::Poll) {
            Outcome::Finished(_, outcome, _) if outcome.is_err() => break,
            Outcome::None => {}
            _ => panic!("lost connection must finish with an error"),
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert!(matches!(spawning, Spawning::Idle));
    assert!(matches!(spawning.transition(Event::Poll), Outcome::None));
    assert!(matches!(
        spawning.transition(Event::Start(&backend, &infos, spawn_worker)),
        Outcome::None
    ));
    assert!(matches!(spawning, Spawning::Idle));
}

#[cfg(unix)]
#[test]
fn a_worker_launch_error_preserves_its_typed_io_source_until_completion() {
    fn fail(_: CoreBackend, _: Spawn, _: mpsc::SyncSender<Completion>) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "fixture launch refusal",
        ))
    }
    let (backend, _peer) = crate::core_terminals::tests::scripted_core::connect("core-a");
    let mut spawning = Spawning::default();
    spawning.transition(Event::Queue(purpose(), Box::default(), None, None));
    spawning.transition(Event::Start(&backend, &Infos::default(), fail));
    let Outcome::Finished(_, outcome, _) = spawning.transition(Event::Poll) else {
        panic!("failed launch must finish through the normal completion path");
    };
    let error = outcome.unwrap_err();
    assert!(
        matches!(&error, super::super::Error::Worker(source) if source.kind() == std::io::ErrorKind::PermissionDenied)
    );
    assert_eq!(
        std::error::Error::source(&error).unwrap().to_string(),
        "fixture launch refusal"
    );
    assert!(matches!(spawning, Spawning::Idle));
    assert!(matches!(spawning.transition(Event::Poll), Outcome::None));
}
