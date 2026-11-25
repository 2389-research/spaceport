// ABOUTME: Resolves a selected pane through its authoritative Herdr workspace and tab.
// ABOUTME: Waits for ordered focus receipts and a coherent surface between requests.
use super::{NavigationTarget, Projection};
use crate::MobileError;
use spaceport_client::protocol::ClientShellSnapshot;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub(super) struct Route {
    request_version: u64,
    workspace: String,
    tab: String,
    pane: String,
    expected_agent: Option<String>,
    last_step: Option<NavigationTarget<String>>,
    deadline: Instant,
}

impl Route {
    fn present(&self, s: &ClientShellSnapshot) -> bool {
        self.expected_agent.as_ref().is_none_or(|name| {
            s.agents.iter().any(|a| {
                a.pane_id == self.pane
                    && a.workspace_id == self.workspace
                    && a.tab_id == self.tab
                    && spaceport_client::text::agent_name(a) == name
            })
        }) && s
            .workspaces
            .iter()
            .any(|w| w.workspace_id == self.workspace)
            && s.tabs
                .iter()
                .any(|t| t.tab_id == self.tab && t.workspace_id == self.workspace)
            && s.panes.iter().any(|p| {
                p.pane_id == self.pane && p.tab_id == self.tab && p.workspace_id == self.workspace
            })
    }

    fn next(&self, s: &ClientShellSnapshot) -> Option<NavigationTarget<String>> {
        if s.focused_workspace_id.as_ref() != Some(&self.workspace) {
            Some(NavigationTarget::Workspace(self.workspace.clone()))
        } else if s.focused_tab_id.as_ref() != Some(&self.tab) {
            Some(NavigationTarget::Tab(self.tab.clone()))
        } else if s.focused_pane_id.as_ref() != Some(&self.pane) {
            Some(NavigationTarget::Pane(self.pane.clone()))
        } else {
            None
        }
    }
}

impl Projection {
    pub(crate) fn begin_go_to(
        &mut self,
        boot: &str,
        workspace: String,
        tab: String,
        pane: String,
        expected_agent: Option<String>,
    ) -> Result<u64, MobileError> {
        if self.navigating() || self.route.is_some() {
            return Err(MobileError::NavigationPending);
        }
        let request_version = self.version + 1;
        let route = Route {
            request_version,
            workspace,
            tab,
            pane,
            expected_agent,
            last_step: None,
            deadline: Instant::now() + Duration::from_secs(16),
        };
        if self
            .pending
            .as_deref()
            .or(self.snapshot.as_deref())
            .is_none_or(|s| s.boot_id != boot || !route.present(s))
        {
            return Err(MobileError::StaleView);
        }
        self.note_input(Instant::now());
        self.route = Some(route);
        self.go_to_arrival = None;
        self.version = request_version;
        Ok(request_version)
    }

    pub(crate) fn cancel_go_to(&mut self) {
        // A focus request already queued still owns its fence.
        if self.route.take().is_some() {
            self.version += 1;
        }
    }

    pub(crate) fn go_to_pending(&self) -> bool {
        self.route.is_some()
    }

    pub(crate) fn go_to_step(&mut self) -> Result<Option<NavigationTarget<String>>, MobileError> {
        let Some(route) = &self.route else {
            return Ok(None);
        };
        let latest = self.pending.as_deref().or(self.snapshot.as_deref());
        if self.disconnected
            || latest.is_none_or(|s| !route.present(s))
            || Instant::now() >= route.deadline
        {
            return Err(MobileError::StaleView);
        }
        if self.navigating() || self.pending.is_some() {
            return Ok(None);
        }
        let Some((snapshot, _)) = self.coherent() else {
            return Ok(None);
        };
        let step = route.next(snapshot);
        // A concurrent focus change must never replay a previous step.
        if let Some(last) = &route.last_step
            && (!last.is_focused_in(snapshot) || !last.reachable_in(snapshot))
        {
            return Err(MobileError::StaleView);
        }
        if let Some(step) = step {
            if let Some(route) = &mut self.route {
                route.last_step = Some(step.clone());
            }
            return Ok(Some(step));
        }
        if self.focused_target().is_some()
            || (self.direct_attach && self.coherent_target().is_some_and(|t| t.popup_id.is_none()))
        {
            self.go_to_arrival = Some(route.request_version);
            self.route = None;
            self.version += 1;
        }
        Ok(None)
    }
}
