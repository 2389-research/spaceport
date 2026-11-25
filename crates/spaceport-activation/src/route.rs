// ABOUTME: Walks a pane destination one level per focus request.
// ABOUTME: Refuses to replay a step and stops at the budget the whole walk gets.
//!
//! A client that has to confirm every hop reaches a pane in three requests:
//! its workspace, then its tab, then the pane itself. Each request is fenced
//! like any other activation, and the next is issued only once the one before
//! it has settled, so no request ever aims at a destination a concurrent focus
//! change has already moved.
//!
//! A client that can focus a pane across workspaces in one request — the
//! desktop does — has no use for a walk and sends the single request. The walk
//! is a rule, not a policy: it exists here so that a client needing it calls it
//! rather than writing it again.
use crate::{NavigationTarget, OwnedNavigationTarget};
use spaceport_client::protocol::ClientShellSnapshot;
use std::time::{Duration, Instant};

/// The budget one whole walk gets, from the first step to the last.
///
/// Distinct from [`crate::TIMEOUT`], which is what one barrier gets: a walk is
/// up to three barriers with a settled projection between each, so it needs a
/// longer bound than any one of them.
pub const ROUTE_BUDGET: Duration = Duration::from_secs(16);

/// What a walk is waiting for next.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Focus this level, then ask again once it settles.
    Focus(OwnedNavigationTarget),
    /// Every level already holds its destination.
    Arrived,
    /// The destination the walk last aimed at is no longer there: a concurrent
    /// focus change moved it, so replaying the step could focus the wrong pane.
    Stale,
}

/// One pane reached through its authoritative workspace and tab.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    workspace: String,
    tab: String,
    pane: String,
    /// The agent expected in the pane, when the caller named one. An agent that
    /// has since moved makes the whole route stale rather than merely unfocused.
    expected_agent: Option<String>,
    last_step: Option<OwnedNavigationTarget>,
    deadline: Instant,
}

impl Route {
    pub fn new(
        workspace: String,
        tab: String,
        pane: String,
        expected_agent: Option<String>,
        now: Instant,
    ) -> Self {
        Self {
            workspace,
            tab,
            pane,
            expected_agent,
            last_step: None,
            deadline: now + ROUTE_BUDGET,
        }
    }

    /// Whether the snapshot still holds the whole chain this walk aimed at.
    pub fn present(&self, snapshot: &ClientShellSnapshot) -> bool {
        self.expected_agent.as_ref().is_none_or(|name| {
            snapshot.agents.iter().any(|agent| {
                agent.pane_id == self.pane
                    && agent.workspace_id == self.workspace
                    && agent.tab_id == self.tab
                    && spaceport_client::text::agent_name(agent) == name
            })
        }) && snapshot
            .workspaces
            .iter()
            .any(|workspace| workspace.workspace_id == self.workspace)
            && snapshot
                .tabs
                .iter()
                .any(|tab| tab.tab_id == self.tab && tab.workspace_id == self.workspace)
            && snapshot.panes.iter().any(|pane| {
                pane.pane_id == self.pane
                    && pane.tab_id == self.tab
                    && pane.workspace_id == self.workspace
            })
    }

    /// The first level the snapshot has not focused yet, shallowest first.
    pub fn next(&self, snapshot: &ClientShellSnapshot) -> Option<OwnedNavigationTarget> {
        if snapshot.focused_workspace_id.as_ref() != Some(&self.workspace) {
            Some(NavigationTarget::Workspace(self.workspace.clone()))
        } else if snapshot.focused_tab_id.as_ref() != Some(&self.tab) {
            Some(NavigationTarget::Tab(self.tab.clone()))
        } else if snapshot.focused_pane_id.as_ref() != Some(&self.pane) {
            Some(NavigationTarget::Pane(self.pane.clone()))
        } else {
            None
        }
    }

    /// The step to issue against `snapshot`, recording it as issued.
    pub fn step(&mut self, snapshot: &ClientShellSnapshot) -> Step {
        // A concurrent focus change must never replay a previous step.
        if self
            .last_step
            .as_ref()
            .is_some_and(|last| !last.is_focused_in(snapshot) || !last.reachable_in(snapshot))
        {
            return Step::Stale;
        }
        match self.next(snapshot) {
            Some(step) => {
                self.last_step = Some(step.clone());
                Step::Focus(step)
            }
            None => Step::Arrived,
        }
    }

    pub fn expired(&self, now: Instant) -> bool {
        now >= self.deadline
    }
}
