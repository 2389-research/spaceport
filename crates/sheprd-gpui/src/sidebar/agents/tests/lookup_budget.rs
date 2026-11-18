// ABOUTME: Verifies agent ordering and captions stay linear in snapshot metadata size.
// ABOUTME: Preserves first-match duplicates, missing IDs, and repeated plugin rows.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{
    preferences::AgentSort,
    sidebar::agents::{agent_name, agent_place, pane_name, visits},
};
use sheprd_client::protocol::ClientShellSnapshot;

fn snapshot(count: usize) -> ClientShellSnapshot {
    let mut snapshot: ClientShellSnapshot = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    )))
    .unwrap();
    let workspace = snapshot.workspaces[0].clone();
    let tab = snapshot.tabs[0].clone();
    let pane = snapshot.panes[0].clone();
    let agent = snapshot.agents[0].clone();
    snapshot.workspaces.clear();
    snapshot.tabs.clear();
    snapshot.panes.clear();
    snapshot.agents.clear();
    for index in 0..count {
        let mut workspace = workspace.clone();
        workspace.workspace_id = format!("w{index}");
        workspace.active_tab_id = format!("t{index}");
        workspace.number = index + 1;
        workspace.label = format!("workspace {index}");
        workspace.focused = index == 0;
        let mut tab = tab.clone();
        tab.workspace_id = workspace.workspace_id.clone();
        tab.tab_id = workspace.active_tab_id.clone();
        tab.focused = index == 0;
        let mut pane = pane.clone();
        pane.workspace_id = workspace.workspace_id.clone();
        pane.tab_id = tab.tab_id.clone();
        pane.pane_id = format!("p{index}");
        pane.label = Some(format!("pane {index}"));
        pane.focused = index == 0;
        let mut agent = agent.clone();
        agent.workspace_id = workspace.workspace_id.clone();
        agent.tab_id = tab.tab_id.clone();
        agent.pane_id = pane.pane_id.clone();
        agent.focused = index == 0;
        snapshot.workspaces.push(workspace);
        snapshot.tabs.push(tab);
        snapshot.panes.push(pane);
        snapshot.agents.push(agent);
    }
    snapshot.focused_workspace_id = Some("w0".into());
    snapshot.focused_tab_id = Some("t0".into());
    snapshot.focused_pane_id = Some("p0".into());
    snapshot.agent_view_label = Some("review".into());
    snapshot.agent_order = (0..count).rev().map(|index| format!("p{index}")).collect();
    snapshot
}

#[test]
fn plugin_order_and_every_rows_metadata_use_a_linear_visit_budget() {
    let snapshot = snapshot(128);
    let (rows, visits) = visits::measure(|| {
        let indexed = IndexedSnapshot::new(&snapshot);
        sorted_agents(&indexed, AgentSort::Grouped)
            .into_iter()
            .map(|agent| {
                let pane = pane_name(agent, &indexed, agent_name(agent));
                let place = agent_place(agent, &indexed);
                assert!(pane.is_some());
                assert!(place.is_some());
                agent.pane_id.as_str()
            })
            .collect::<Vec<_>>()
    });
    assert_eq!(rows.first().copied(), Some("p127"));
    assert_eq!(rows.last().copied(), Some("p0"));
    let size = snapshot.agents.len()
        + snapshot.agent_order.len()
        + snapshot.workspaces.len()
        + snapshot.tabs.len()
        + snapshot.panes.len();
    eprintln!("Lookup budget: {visits} actual row visits for {size} source rows");
    assert!(
        visits <= size * 8,
        "{visits} actual row visits for {size} source rows exceeds a linear budget"
    );
}

#[test]
fn duplicate_ids_keep_first_matches_and_repeated_plugin_rows() {
    let mut snapshot = snapshot(2);
    let mut duplicate = snapshot.agents[0].clone();
    duplicate.title = Some("later agent".into());
    snapshot.agents.push(duplicate);
    let mut workspace = snapshot.workspaces[0].clone();
    workspace.label = "later workspace".into();
    snapshot.workspaces.push(workspace);
    let mut tab = snapshot.tabs[0].clone();
    tab.label = "later tab".into();
    snapshot.tabs.push(tab);
    snapshot.tabs[0].label = "first tab".into();
    let mut pane = snapshot.panes[0].clone();
    pane.label = Some("later pane".into());
    snapshot.panes.push(pane);
    snapshot.panes[0].label = Some("first pane".into());
    snapshot.agent_order = vec!["p0".into(), "missing".into(), "p0".into()];
    let indexed = IndexedSnapshot::new(&snapshot);
    let ordered = sorted_agents(&indexed, AgentSort::Priority);
    assert_eq!(ordered.len(), 2);
    assert!(
        ordered
            .iter()
            .all(|agent| std::ptr::eq(*agent, &snapshot.agents[0]))
    );
    assert_eq!(
        agent_place(ordered[0], &indexed),
        Some(("workspace 0", Some("first tab")))
    );
    assert_eq!(
        pane_name(ordered[0], &indexed, "reviewer").as_deref(),
        Some("first pane")
    );
    snapshot.agent_order.clear();
    assert!(sorted_agents(&IndexedSnapshot::new(&snapshot), AgentSort::Priority).is_empty());
}
