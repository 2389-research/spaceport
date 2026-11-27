// ABOUTME: Verifies roster observations from the same snapshot and surface fixtures as terminal sessions.
// ABOUTME: Checks that only snapshots and disconnects affect the observer's public view.
use super::*;
use crate::tests::{snapshot, surface};

#[test]
fn observer_collects_agents_without_accepting_terminal_surfaces() {
    let state = Mutex::new(Projection::new(ConnectOptions::default()));
    let revision = lock(&state).version;
    collect(
        [
            ClientEvent::Snapshot(Arc::new(snapshot())),
            ClientEvent::Surface(Arc::new(surface())),
            ClientEvent::Disconnected {
                reason: "peer closed".into(),
                ssh: None,
            },
        ],
        &state,
    );
    let view = lock(&state).view();
    assert_eq!(view.version, revision + 2);
    assert!(view.roster_observed_at_ms.is_some());
    assert_eq!(view.panes[0].status, "blocked");
    assert!(view.input_target.is_none());
    assert!(view.terminal.is_none());
    assert!(view.disconnected);
    assert_eq!(view.error.as_deref(), Some("peer closed"));
}
