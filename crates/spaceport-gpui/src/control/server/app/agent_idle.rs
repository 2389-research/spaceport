// ABOUTME: Reads the chosen pane's idle observation from the window that owns its workspace.
// ABOUTME: Uses the sidebar's confirmation time so a cached observation keeps its last age.
use super::{Caller, Reply, Response, SpaceportWindow};
use crate::control::{
    protocol::app::AgentIdleAge,
    server::{not_found, work},
};
use spaceport_client::agent_idle::{IdleAgeGroup, now_ms};

pub(super) fn read(view: &SpaceportWindow, caller: &Caller, pane: Option<&str>) -> Response {
    let found = match work::find(view, caller) {
        Ok(found) => found,
        Err(response) => return response,
    };
    let Some(pane) = pane
        .or(caller.pane_id.as_deref())
        .or(found.snapshot.focused_pane_id.as_deref())
    else {
        return not_found("This workspace has no focused pane");
    };
    if !found
        .snapshot
        .panes
        .iter()
        .any(|p| p.pane_id == pane && p.workspace_id == found.workspace.workspace_id)
    {
        return not_found("This workspace does not contain the requested pane");
    }
    let live = view.live_of(found.index);
    let confirmed = live.agent_confirmed_at(now_ms());
    let age = live
        .agent_idle(pane)
        .filter(|idle| idle.group(confirmed) != IdleAgeGroup::Unplaced)
        .map(|idle| AgentIdleAge {
            seconds: confirmed.saturating_sub(idle.first_observed_ms) / 1000,
            lower_bound: !idle.onset_observed,
        });
    Response::App {
        reply: Reply::AgentIdle {
            pane: pane.to_owned(),
            age,
        },
    }
}
