// ABOUTME: Checks a walk takes one level per step and stops when it is overtaken.
// ABOUTME: Covers a destination that vanished, one whose agent moved, and the budget.
use super::*;
use crate::route::Step;
use crate::{OwnedNavigationTarget, ROUTE_BUDGET, Route};
use std::time::{Duration, Instant};

fn route(now: Instant) -> Route {
    Route::new("infra".into(), "deploy".into(), "watch".into(), None, now)
}

fn focused(
    mut snapshot: ClientShellSnapshot,
    workspace: &str,
    tab: &str,
    pane: &str,
) -> ClientShellSnapshot {
    snapshot.focused_workspace_id = Some(workspace.into());
    snapshot.focused_tab_id = Some(tab.into());
    snapshot.focused_pane_id = Some(pane.into());
    snapshot
}

#[test]
fn a_walk_takes_the_shallowest_level_the_snapshot_has_not_focused() {
    let mut walk = route(Instant::now());
    let mut snapshot = two_workspaces();
    assert!(walk.present(&snapshot));

    assert_eq!(
        walk.step(&snapshot),
        Step::Focus(OwnedNavigationTarget::Workspace("infra".into()))
    );
    snapshot = focused(snapshot, "infra", "w1:t1", "w1:p1");
    assert_eq!(
        walk.step(&snapshot),
        Step::Focus(OwnedNavigationTarget::Tab("deploy".into()))
    );
    snapshot = focused(snapshot, "infra", "deploy", "w1:p1");
    assert_eq!(
        walk.step(&snapshot),
        Step::Focus(OwnedNavigationTarget::Pane("watch".into()))
    );
    snapshot = focused(snapshot, "infra", "deploy", "watch");
    assert_eq!(walk.step(&snapshot), Step::Arrived);
}

#[test]
fn a_walk_that_is_overtaken_stops_instead_of_replaying_its_step() {
    let mut walk = route(Instant::now());
    let snapshot = two_workspaces();
    assert_eq!(
        walk.step(&snapshot),
        Step::Focus(OwnedNavigationTarget::Workspace("infra".into()))
    );
    // Someone focused another workspace before the step landed.
    let elsewhere = focused(snapshot, "w1", "w1:t1", "w1:p1");
    assert_eq!(walk.step(&elsewhere), Step::Stale);
}

#[test]
fn a_walk_whose_destination_vanished_is_no_longer_present() {
    let mut snapshot = two_workspaces();
    snapshot.workspaces.retain(|w| w.workspace_id != "infra");
    assert!(!route(Instant::now()).present(&snapshot));
}

#[test]
fn a_walk_expecting_an_agent_is_stale_once_that_agent_leaves_the_pane() {
    let mut snapshot = two_workspaces();
    let mut agent = snapshot.agents[0].clone();
    agent.pane_id = "watch".into();
    agent.workspace_id = "infra".into();
    agent.tab_id = "deploy".into();
    let name = spaceport_client::text::agent_name(&agent).to_owned();
    snapshot.agents = vec![agent];
    let walk = Route::new(
        "infra".into(),
        "deploy".into(),
        "watch".into(),
        Some(name.clone()),
        Instant::now(),
    );
    assert!(walk.present(&snapshot));

    snapshot.agents[0].display_agent = Some("Someone else".into());
    snapshot.agents[0].name = None;
    snapshot.agents[0].agent = None;
    snapshot.agents[0].title = None;
    assert!(!walk.present(&snapshot), "another agent holds the pane");
}

#[test]
fn a_walk_stops_at_the_budget_it_was_given() {
    let now = Instant::now();
    let walk = route(now);
    assert!(!walk.expired(now + ROUTE_BUDGET - Duration::from_millis(1)));
    assert!(walk.expired(now + ROUTE_BUDGET));
}
