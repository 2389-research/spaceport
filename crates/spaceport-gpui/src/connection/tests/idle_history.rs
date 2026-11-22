// ABOUTME: Exercises idle evidence across real bridge inbox replacements.
// ABOUTME: Checks late readers, contended resets, and explicit target changes.
use super::*;
use spaceport_client::protocol::{AgentStatus, ClientShellSnapshot};

fn snapshot(revision: u64, status: AgentStatus) -> Arc<ClientShellSnapshot> {
    let mut snap = crate::sidebar::layout_tests::snapshot(1);
    snap.revision = revision;
    snap.agents.truncate(1);
    snap.agents[0].state_change_seq = revision;
    snap.agents[0].agent_status = status;
    Arc::new(snap)
}

#[test]
fn a_same_host_reset_keeps_confirmed_idle_without_sharing_the_retired_inbox() {
    let mut bridge = ConnectionBridge::new(ConnectTarget::Local);
    let retired = bridge.inbox.clone();
    {
        let mut reader = retired.lock().unwrap();
        reader.apply(ClientEvent::Snapshot(snapshot(1, AgentStatus::Working)));
        reader.apply(ClientEvent::Snapshot(snapshot(2, AgentStatus::Idle)));
    }
    let idle = bridge.take_update().unwrap().agent_idle("p0").unwrap();
    assert!(idle.onset_observed);
    let mut reader = retired.lock().unwrap();
    bridge.detach(true);
    reader.apply(ClientEvent::Snapshot(snapshot(3, AgentStatus::Working)));
    drop(reader);

    bridge.reset(ConnectionStatus::Connecting, true);
    bridge
        .inbox
        .lock()
        .unwrap()
        .apply(ClientEvent::Snapshot(snapshot(2, AgentStatus::Idle)));
    assert_eq!(bridge.take_update().unwrap().agent_idle("p0"), Some(idle));

    bridge.target = ConnectTarget::Socket("/unrelated-test-client.sock".into());
    bridge.reset(ConnectionStatus::Connecting, true);
    bridge
        .inbox
        .lock()
        .unwrap()
        .apply(ClientEvent::Snapshot(snapshot(2, AgentStatus::Idle)));
    assert!(
        !bridge
            .take_update()
            .unwrap()
            .agent_idle("p0")
            .unwrap()
            .onset_observed
    );
}
