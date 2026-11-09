//! Read-only Git probes for the focused local checkout and listed workspaces.

use crate::SheprdWindow;

impl SheprdWindow {
    /// Follow the focused checkout and drain worker results. Called from the
    /// window's poll task, never from a render or input path.
    pub(crate) fn update_git(&mut self) -> bool {
        let now = std::time::Instant::now();
        let mut changed = self.git.track(self.git_input(), self.active, now);
        self.git.set_counted(
            self.config.sidebar_style.change_counts == crate::config::ChangeCounts::Local,
        );
        self.git.set_read_ahead(self.menu.github.connected());
        self.git
            .track_listed(self.listed_git_inputs(), self.active, now);
        changed |= self.git.poll(now);
        changed
    }

    /// Every listed local checkout, so the sidebar can mark the ones holding
    /// uncommitted work. Empty when the endpoint is not the owned local daemon.
    fn listed_git_inputs(&self) -> Vec<crate::pull_request::Input> {
        if !self.local_git_endpoint() {
            return Vec::new();
        }
        self.live
            .snapshot
            .iter()
            .flat_map(|snapshot| snapshot.workspaces.iter())
            .filter_map(|workspace| {
                crate::pull_request::repository_input(
                    workspace.worktree.as_ref(),
                    workspace.branch.as_deref(),
                )
                .ok()
            })
            .collect()
    }

    /// Local, owned daemon sockets only: the same trust boundary the PR lookup
    /// uses, because both run Git against the user's own checkouts.
    fn local_git_endpoint(&self) -> bool {
        self.selected_endpoint == 0
            && self.live.local_daemon_peer
            && self.live.status.is_connected()
    }

    /// The focused workspace's checkout, when it is one
    /// this client may run Git in. The daemon only attaches worktree metadata
    /// to a worktree made through Herdr, so a space merely opened in a
    /// repository is named by its directory, which the worker resolves and
    /// verifies as the pull request lookup does; see `workspace_input`.
    pub(crate) fn git_input(&self) -> Option<crate::pull_request::Input> {
        if !self.local_git_endpoint() {
            return None;
        }
        let snapshot = self.live.snapshot.as_ref()?;
        let workspace = snapshot
            .workspaces
            .iter()
            .find(|workspace| {
                Some(workspace.workspace_id.as_str()) == snapshot.focused_workspace_id.as_deref()
            })
            .or_else(|| {
                snapshot
                    .workspaces
                    .iter()
                    .find(|workspace| workspace.focused)
            })?;
        crate::pull_request::workspace_input(workspace, &crate::pull_request::Origin::Local).ok()
    }
}

#[cfg(test)]
mod tests;
