//! Sounds read the newest metadata even while its surface is still on its way.

use super::*;
use sheprd_client::protocol::{
    AgentStatus, ClientShellSnapshot, SemanticNotification, SemanticNotificationKind as Kind,
    SemanticNotificationSound as Sound,
};
use sheprd_sound::Policy;
use std::time::{Duration, Instant};

fn event(kind: Kind) -> SemanticNotification {
    SemanticNotification {
        kind,
        title: "test".into(),
        body: None,
        sound: Some(Sound::Done),
        agent: Some("claude".into()),
        workspace_id: Some("workspace".into()),
        tab_id: Some("tab".into()),
        pane_id: Some("pane".into()),
        position: None,
    }
}

/// A connected endpoint showing the fixture snapshot, its first agent in
/// `status` on the fixture's pane and focused in its workspace and tab.
fn live(status: AgentStatus) -> LiveState {
    let mut snapshot: ClientShellSnapshot = serde_json::from_str(include_str!(
        "../../../../sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    ))
    .unwrap();
    snapshot.agents.truncate(1);
    snapshot.agents[0].pane_id = "pane".into();
    snapshot.agents[0].agent_status = status;
    snapshot.focused_tab_id = Some("tab".into());
    snapshot.focused_workspace_id = Some("workspace".into());
    let mut live = LiveState::default();
    live.status = ConnectionStatus::Connected;
    live.apply(ClientEvent::Snapshot(Arc::new(snapshot)));
    live
}

/// One policy poll of what `state` queued, as the window runs it.
fn poll_policy(
    policy: &mut Policy,
    state: &mut LiveState,
    delay: Option<u64>,
    focused: bool,
    now: Instant,
    emit: impl FnMut(SemanticNotification),
) {
    let snapshot = state.latest_snapshot().cloned();
    let live = state.sound_live(snapshot.as_deref());
    policy.poll(live, delay, focused, now, emit);
}

/// The agent changed status in metadata that waits for its surface, so the
/// displayed snapshot still says it is working.
#[test]
fn staged_agent_status_validates_delayed_sounds_once() {
    for (kind, status) in [
        (Kind::NeedsAttention, AgentStatus::Blocked),
        (Kind::Finished, AgentStatus::Done),
    ] {
        let now = Instant::now();
        let mut state = live(AgentStatus::Working);
        let displayed = state.snapshot.clone().unwrap();
        state.apply(ClientEvent::Surface(surface(&displayed)));
        let mut staged = (*displayed).clone();
        staged.revision += 1;
        staged.agents[0].agent_status = status;
        let staged = Arc::new(staged);
        state.apply(ClientEvent::Snapshot(staged.clone()));
        assert!(Arc::ptr_eq(state.snapshot.as_ref().unwrap(), &displayed));
        state.sound_events.push_back((now, event(kind)));
        let mut policy = Policy::default();
        let mut sounds = 0;
        for at in [now, now + Duration::from_secs(1)] {
            poll_policy(&mut policy, &mut state, Some(1), false, at, |_| sounds += 1);
        }
        assert_eq!(sounds, 1, "{kind:?}");
        state.apply(ClientEvent::Surface(surface(&staged)));
        assert!(Arc::ptr_eq(state.snapshot.as_ref().unwrap(), &staged));
        poll_policy(
            &mut policy,
            &mut state,
            Some(1),
            false,
            now + Duration::from_secs(5),
            |_| panic!("replayed sound"),
        );
    }
}
