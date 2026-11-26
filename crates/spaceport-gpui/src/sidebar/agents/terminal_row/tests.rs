// ABOUTME: Checks terminal tab numbers on sidebar rows for split panes.
// ABOUTME: Covers tab renumbering and a partial snapshot without the matching tab.
use super::*;
use anyhow::{Context as _, Result};

#[test]
fn agent_rows_follow_their_tabs_reported_number() -> Result<()> {
    let mut snapshot = crate::sidebar::layout_tests::snapshot(1);
    let mut agent = snapshot.agents[0].clone();
    agent.terminal_title_stripped = Some("Review".into());
    let label = |agent: &ClientShellAgent, snapshot: &ClientShellSnapshot| {
        TerminalRow::from_snapshot(agent, snapshot, None)
            .context("missing terminal row")
            .map(|row| row.tab_number)
    };
    snapshot.tabs.reverse();
    for pane in ["wAB:pX", "wAB:pY", "wAB:pZ", "pane-42"] {
        agent.pane_id = pane.into();
        assert_eq!(label(&agent, &snapshot)?.as_deref(), Some("#1"));
    }
    let tab = snapshot
        .tabs
        .iter_mut()
        .find(|t| t.tab_id == agent.tab_id)
        .context("missing tab in the test snapshot")?;
    tab.number = 8;
    assert_eq!(label(&agent, &snapshot)?.as_deref(), Some("#8"));
    agent.workspace_id = "another-workspace".into();
    assert_eq!(label(&agent, &snapshot)?, None);
    snapshot.tabs.clear();
    assert_eq!(label(&agent, &snapshot)?, None);
    Ok(())
}

#[test]
fn directory_uses_a_distinct_text_role_from_other_metadata() {
    let row = TerminalRow {
        title: Some("Review".into()),
        tab_number: Some("#3".into()),
        directory: Some("spaceport".into()),
    };
    let lines = row.lines("Codex", "Relay", None);
    assert_eq!(
        lines[1],
        [
            text("Codex", TextRole::Agent),
            text("#3", TextRole::Secondary),
            text("spaceport", TextRole::Directory),
            text("Relay", TextRole::Secondary),
        ]
    );
}
