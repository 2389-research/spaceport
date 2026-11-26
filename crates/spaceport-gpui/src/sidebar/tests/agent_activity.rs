//! Where a painted agent's activity comes from: the daemon's listing alone, so
//! a working dot never outlives the agent that reported it.

use super::*;
use crate::{LiveState, preferences::AgentSort, sidebar::agents::panel_agents};
use spaceport_client::ClientEvent;
use std::sync::Arc;

/// A daemon's listing of `(pane, status)` agents.
fn listing(agents: &[(&str, AgentStatus)]) -> Arc<ClientShellSnapshot> {
    let mut snapshot = layout_tests::snapshot(1);
    let template = snapshot.agents[0].clone();
    snapshot.agents = agents
        .iter()
        .map(|&(pane, agent_status)| ClientShellAgent {
            pane_id: pane.into(),
            agent_status,
            ..template.clone()
        })
        .collect();
    Arc::new(snapshot)
}

/// What the agents panel paints: each listed pane with the status it shows.
fn painted(live: &LiveState) -> Vec<(&str, AgentStatus)> {
    panel_agents(
        std::iter::once((0, live)),
        AgentSort::Grouped,
        spaceport_client::agent_idle::now_ms(),
        Default::default(),
    )
    .into_iter()
    .map(|(_, agent)| (agent.pane_id.as_str(), agent.agent_status))
    .collect()
}

#[test]
fn a_working_dot_does_not_outlive_the_agent_the_daemon_listed() {
    // Bug 57b2: agents that "pop up and show active" while nothing runs there.
    // The panel paints the daemon's word and nothing of its own, so the one
    // way it can carry activity past its agent is by keeping an entry the
    // daemon has dropped. It must not: an exited agent leaves no row, and no
    // stale working dot, behind.
    let mut live = LiveState::default();
    live.apply(
        ClientEvent::Snapshot(listing(&[("w1:p1", AgentStatus::Working)])),
        crate::state::describe,
        crate::state::localize,
    );
    assert_eq!(painted(&live), [("w1:p1", AgentStatus::Working)]);
    live.apply(
        ClientEvent::Snapshot(listing(&[])),
        crate::state::describe,
        crate::state::localize,
    );
    assert!(painted(&live).is_empty());
}

#[test]
fn a_restarted_daemon_paints_the_status_it_reports_now() {
    // A new boot reuses pane ids for whatever runs there next, so the row left
    // from before must not keep the old agent's working dot.
    let mut live = LiveState::default();
    live.apply(
        ClientEvent::Snapshot(listing(&[("w1:p1", AgentStatus::Working)])),
        crate::state::describe,
        crate::state::localize,
    );
    assert_eq!(painted(&live), [("w1:p1", AgentStatus::Working)]);
    let mut reboot = (*listing(&[("w1:p1", AgentStatus::Idle)])).clone();
    reboot.boot_id = "next-boot".into();
    live.apply(
        ClientEvent::Snapshot(Arc::new(reboot)),
        crate::state::describe,
        crate::state::localize,
    );
    assert_eq!(painted(&live), [("w1:p1", AgentStatus::Idle)]);
}
