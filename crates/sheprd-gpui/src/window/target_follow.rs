//! Follow an exact endpoint, workspace, or pane after a user-requested move.
//! The source operation owns execution; this window only steers its view.

use crate::{NavigationTarget, SheprdWindow};
use gpui::Context;
use std::time::{Duration, Instant};

const FOLLOW_FOR: Duration = Duration::from_secs(20);
const FOLLOW_RETRY: Duration = Duration::from_millis(750);

/// The workspace or pane a completed operation should leave focused.
pub(crate) struct Follow {
    endpoint_id: String,
    workspace_id: String,
    pane_id: Option<String>,
    until: Instant,
    next: Option<Instant>,
}

impl Follow {
    pub(crate) fn new(endpoint_id: String, workspace_id: String) -> Self {
        Self {
            endpoint_id,
            workspace_id,
            pane_id: None,
            until: Instant::now() + FOLLOW_FOR,
            next: None,
        }
    }

    pub(crate) fn pane(endpoint_id: String, workspace_id: String, pane_id: String) -> Self {
        Self {
            pane_id: Some(pane_id),
            ..Self::new(endpoint_id, workspace_id)
        }
    }
}

impl SheprdWindow {
    /// Retry briefly while the exact endpoint connects or its target appears.
    pub(crate) fn follow_target(&mut self, cx: &mut Context<Self>) {
        let Some(follow) = &mut self.target_follow else {
            return;
        };
        let now = Instant::now();
        if now >= follow.until {
            self.target_follow = None;
            return;
        }
        if self.menu.page.is_some() || follow.next.is_some_and(|next| now < next) {
            return;
        }
        follow.next = Some(now + FOLLOW_RETRY);
        let (endpoint, workspace) = (follow.endpoint_id.clone(), follow.workspace_id.clone());
        let pane = follow.pane_id.clone();
        let target = match &pane {
            Some(pane) => NavigationTarget::Pane(pane.as_str()),
            None => NavigationTarget::Workspace(workspace.as_str()),
        };
        if self.endpoints[self.selected_endpoint].id != endpoint {
            self.navigate_endpoint(&endpoint, target, cx);
            return;
        }
        let Some(snapshot) = &self.live.snapshot else {
            return;
        };
        let (focused, listed) = match &pane {
            Some(pane) => (
                snapshot.focused_pane_id.as_deref() == Some(pane.as_str()),
                snapshot.panes.iter().any(|p| p.pane_id == *pane),
            ),
            None => (
                snapshot.focused_workspace_id.as_deref() == Some(workspace.as_str()),
                snapshot
                    .workspaces
                    .iter()
                    .any(|w| w.workspace_id == workspace),
            ),
        };
        if focused {
            self.target_follow = None;
            return;
        }
        if listed && self.pending_navigation.is_none() {
            self.navigate(target, cx);
        }
    }
}
