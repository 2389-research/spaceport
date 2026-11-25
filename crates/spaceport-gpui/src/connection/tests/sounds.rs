//! The sound queue the connection owns: how it is bounded, what a
//! reconnection or a detach retires, and the policy that drains it.

use super::*;
use spaceport_client::protocol::{
    AgentStatus, ClientShellSnapshot, SemanticNotification, SemanticNotificationKind as Kind,
    SemanticNotificationSound as Sound, ServerMessage,
};
use spaceport_sound::Policy;
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
        "../../../../spaceport-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    ))
    .unwrap();
    snapshot.agents.truncate(1);
    snapshot.agents[0].pane_id = "pane".into();
    snapshot.agents[0].agent_status = status;
    snapshot.focused_tab_id = Some("tab".into());
    snapshot.focused_workspace_id = Some("workspace".into());
    // A committed snapshot is a connected endpoint.
    let mut live = LiveState::default();
    live.apply(
        ClientEvent::Snapshot(Arc::new(snapshot)),
        crate::state::describe,
        crate::state::localize,
    );
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

#[test]
fn bridge_moves_bounded_notifications_and_reload_once() {
    let bridge = bridge();
    {
        let mut state = bridge.inbox.lock().unwrap();
        state.status = ConnectionStatus::Connected;
        for i in 0..spaceport_sound::MAX_PENDING + 5 {
            let mut event = event(Kind::Custom);
            event.title = i.to_string();
            state.apply(
                ClientEvent::Message(ServerMessage::SemanticNotification(event)),
                crate::state::describe,
                crate::state::localize,
            );
        }
        state.apply(
            ClientEvent::Message(ServerMessage::ReloadSoundConfig),
            crate::state::describe,
            crate::state::localize,
        );
    }
    let first = bridge.take_update().unwrap();
    assert_eq!(first.sound_events.len(), spaceport_sound::MAX_PENDING);
    assert_eq!(first.sound_events[0].1.title, "5");
    // Toast coalescing must not consume the independent sound delivery queue.
    assert_eq!(first.notifications.len(), 1);
    assert_eq!(
        first.notifications[0].title,
        (spaceport_sound::MAX_PENDING + 4).to_string()
    );
    assert!(first.reload_sound);
    assert!(bridge.take_update().is_none());
    bridge.inbox.lock().unwrap().set_outer_focus(true);
    let next = bridge.take_update().unwrap();
    assert!(next.sound_events.is_empty());
    assert!(next.notifications.is_empty());
    assert!(!next.reload_sound);
}

#[test]
fn disconnect_boot_and_detach_cancel_pending_and_queued_generations() {
    let now = Instant::now();
    for boot_change in [false, true] {
        let mut state = live(AgentStatus::Done);
        let token = state.sound_cancel.clone();
        state.sound_events.push_back((now, event(Kind::Finished)));
        let mut policy = Policy::default();
        poll_policy(&mut policy, &mut state, Some(1), false, now, |_| panic!());
        if boot_change {
            let mut snapshot = state.snapshot.clone().unwrap();
            Arc::make_mut(&mut snapshot).boot_id = "new-boot".into();
            state.apply(
                ClientEvent::Snapshot(snapshot),
                crate::state::describe,
                crate::state::localize,
            );
        } else {
            state.apply(
                ClientEvent::Disconnected {
                    reason: "test".into(),
                    ssh: None,
                },
                crate::state::describe,
                crate::state::localize,
            );
        }
        assert!(token.load(Ordering::Acquire));
        poll_policy(
            &mut policy,
            &mut state,
            Some(1),
            false,
            now + Duration::from_secs(1),
            |_| panic!("retired generation"),
        );
        assert_eq!(policy.waiting(), 0, "the retired generation left nothing");
    }
    let mut bridge = bridge();
    let old = bridge.inbox.clone();
    let token = old.lock().unwrap().sound_cancel.clone();
    bridge.detach(false);
    assert!(token.load(Ordering::Acquire));
    old.lock().unwrap().apply(
        ClientEvent::Message(ServerMessage::SemanticNotification(event(Kind::Custom))),
        crate::state::describe,
        crate::state::localize,
    );
    assert!(bridge.take_update().unwrap().sound_events.is_empty());
    let token = bridge.inbox.lock().unwrap().sound_cancel.clone();
    drop(bridge);
    assert!(token.load(Ordering::Acquire));
}
