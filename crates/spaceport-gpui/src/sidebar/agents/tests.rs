//! The one ordering test that reads the app's shared snapshot fixture. The
//! indicators, the status names and the state labels are
//! `spaceport_sidebar::agents`'s own tests now.

use super::sorted_agents;
use spaceport_client::protocol::AgentStatus;

fn ordered(
    snapshot: &spaceport_client::protocol::ClientShellSnapshot,
    sort: crate::preferences::AgentSort,
) -> Vec<&str> {
    sorted_agents(snapshot, sort)
        .into_iter()
        .map(|agent| agent.pane_id.as_str())
        .collect()
}

#[test]
fn a_plugin_view_order_replaces_the_local_sort_and_filters() {
    use crate::preferences::AgentSort;
    let mut snapshot = crate::sidebar::layout_tests::snapshot(2);
    snapshot.agents[1].agent_status = AgentStatus::Blocked;
    // Without a view the order is local, even when the daemon sent one.
    snapshot.agent_order = vec!["p1".into()];
    assert_eq!(ordered(&snapshot, AgentSort::Grouped), ["p0", "p1"]);
    assert_eq!(ordered(&snapshot, AgentSort::Priority), ["p1", "p0"]);

    snapshot.agent_view_label = Some("review".into());
    snapshot.agent_order = vec!["p1".into(), "gone".into(), "p0".into()];
    for sort in [AgentSort::Grouped, AgentSort::Priority] {
        // A pane the snapshot no longer lists as an agent is skipped.
        assert_eq!(ordered(&snapshot, sort), ["p1", "p0"]);
    }
    snapshot.agents[1].agent_status = AgentStatus::Idle;
    snapshot.agent_order = vec!["p0".into()];
    assert_eq!(ordered(&snapshot, AgentSort::Priority), ["p0"]);
    snapshot.agent_order.clear();
    assert!(ordered(&snapshot, AgentSort::Grouped).is_empty());
}
