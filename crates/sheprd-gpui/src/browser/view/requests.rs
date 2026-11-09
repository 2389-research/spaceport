//! Control target matching for app requests.

use crate::{SheprdWindow, browser::Scope, control::Target};

impl SheprdWindow {
    /// The endpoint and workspace a control request names, if this window
    /// shows it. `strict` requires the caller's own daemon; otherwise any
    /// endpoint showing the named workspace qualifies. Only the Unix control
    /// socket asks.
    pub(crate) fn control_target(
        &self,
        target: &Target<'_>,
        strict: bool,
    ) -> Option<(usize, String)> {
        self.endpoints
            .iter()
            .enumerate()
            .find_map(|(index, endpoint)| {
                if strict {
                    let matches = match target.daemon {
                        Some(daemon) => {
                            endpoint.connection.target.socket_path().ok().as_deref() == Some(daemon)
                        }
                        None => index == self.selected_endpoint,
                    };
                    if !matches {
                        return None;
                    }
                } else if target.workspace.is_none() {
                    return None;
                }
                let live = if index == self.selected_endpoint {
                    &self.live
                } else {
                    &endpoint.live
                };
                let snapshot = live.snapshot.as_ref()?;
                let workspace = match target.workspace {
                    Some(id) => id,
                    None => snapshot.focused_workspace_id.as_deref()?,
                };
                snapshot
                    .workspaces
                    .iter()
                    .any(|candidate| candidate.workspace_id == workspace)
                    .then(|| (index, workspace.to_owned()))
            })
    }

    /// Whether the host this window shows is the daemon at `daemon`, listing
    /// workspaces or not, connected or not. Window-wide commands act on the
    /// shown host, so only that one may stand for the caller's daemon.
    pub(crate) fn shows_daemon(&self, daemon: &std::path::Path) -> bool {
        self.endpoints
            .get(self.selected_endpoint)
            .is_some_and(|endpoint| {
                endpoint.connection.target.socket_path().ok().as_deref() == Some(daemon)
            })
    }

    /// The endpoint of this window whose live snapshot lists `workspace` of
    /// `scope`, whether or not the window shows it now.
    pub(crate) fn endpoint_showing(&self, scope: &Scope, workspace: &str) -> Option<usize> {
        self.endpoints
            .iter()
            .enumerate()
            .find_map(|(index, endpoint)| {
                if &super::scope(endpoint) != scope {
                    return None;
                }
                let live = if index == self.selected_endpoint {
                    &self.live
                } else {
                    &endpoint.live
                };
                live.snapshot
                    .as_ref()?
                    .workspaces
                    .iter()
                    .any(|candidate| candidate.workspace_id == workspace)
                    .then_some(index)
            })
    }
}
