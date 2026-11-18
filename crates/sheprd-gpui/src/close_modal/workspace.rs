// ABOUTME: Captures Herdr TUI workspace closure and its complete affected group.
// ABOUTME: Revalidates consent against the current boot, workspaces, tabs, and panes.
use super::{CloseConfirmation, Target as CloseTarget};
use crate::{
    Error, Result, SheprdWindow,
    i18n::{Localize, fl},
    menu::Page,
};
use gpui::{Context, Window};
use serde_json::{Value, json};
use sheprd_client::{
    Method,
    protocol::{ClientShellSnapshot, ClientShellWorkspace},
};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Target {
    boot: String,
    root: String,
    members: Vec<Member>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Member {
    id: String,
    label: String,
    tabs: BTreeSet<String>,
    panes: BTreeSet<(String, String)>,
}

impl Target {
    fn capture(snapshot: &ClientShellSnapshot, id: &str) -> Option<Self> {
        let root = snapshot
            .workspaces
            .iter()
            .find(|workspace| workspace.workspace_id == id)?;
        let mut members: Vec<_> = affected(snapshot, root)
            .into_iter()
            .map(|workspace| Member {
                id: workspace.workspace_id.clone(),
                label: workspace.label.clone(),
                tabs: snapshot
                    .tabs
                    .iter()
                    .filter(|tab| tab.workspace_id == workspace.workspace_id)
                    .map(|tab| tab.tab_id.clone())
                    .collect(),
                panes: snapshot
                    .panes
                    .iter()
                    .filter(|pane| pane.workspace_id == workspace.workspace_id)
                    .map(|pane| (pane.pane_id.clone(), pane.tab_id.clone()))
                    .collect(),
            })
            .collect();
        members.sort_by(|left, right| left.id.cmp(&right.id));
        Some(Self {
            boot: snapshot.boot_id.clone(),
            root: id.to_owned(),
            members,
        })
    }

    pub(super) fn pane_count(&self) -> usize {
        self.members.iter().map(|member| member.panes.len()).sum()
    }

    pub(super) fn workspace_count(&self) -> usize {
        self.members.len()
    }

    fn labels(&self) -> String {
        self.members
            .iter()
            .map(|member| {
                if member.label.is_empty() {
                    member.id.as_str()
                } else {
                    member.label.as_str()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn request(&self, snapshot: &ClientShellSnapshot) -> Result<(Method, Value)> {
        if Self::capture(snapshot, &self.root).as_ref() != Some(self) {
            return Err(Error::StaleCloseTarget);
        }
        Ok((
            Method::WorkspaceClose,
            json!({
                "workspace_id": self.root, "close_group": self.members.len() > 1,
            }),
        ))
    }
}

/// Matches Herdr TUI's workspace_close_indices: a primary owns its group unless
/// another primary remains open; linked workspaces always close independently.
fn affected<'a>(
    snapshot: &'a ClientShellSnapshot,
    root: &'a ClientShellWorkspace,
) -> Vec<&'a ClientShellWorkspace> {
    let Some(tree) = root
        .worktree
        .as_ref()
        .filter(|tree| !tree.is_linked_worktree)
    else {
        return vec![root];
    };
    let group: Vec<_> = snapshot
        .workspaces
        .iter()
        .filter(|workspace| {
            workspace
                .worktree
                .as_ref()
                .is_some_and(|member| member.key == tree.key)
        })
        .collect();
    if group.iter().any(|workspace| {
        workspace.workspace_id != root.workspace_id
            && workspace
                .worktree
                .as_ref()
                .is_some_and(|member| !member.is_linked_worktree)
    }) {
        vec![root]
    } else {
        group
    }
}

impl CloseConfirmation {
    pub(super) fn capture_workspace(snapshot: &ClientShellSnapshot, id: &str) -> Option<Self> {
        let target = Target::capture(snapshot, id)?;
        Some(Self {
            label: target.labels(),
            target: CloseTarget::Workspace(target),
            interrupts: None,
            confirm_selected: false,
            do_not_ask_again: false,
            error: None,
        })
    }
}

impl SheprdWindow {
    pub(crate) fn open_workspace_close(
        &mut self,
        id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.selected_core_group().is_some() || !self.live.status.is_connected() {
            return false;
        }
        let Some(close) = self
            .live
            .snapshot
            .as_ref()
            .and_then(|snapshot| CloseConfirmation::capture_workspace(snapshot, id))
        else {
            return false;
        };
        match self.menu.page {
            Some(Page::Workspace)
                if self.menu_target_current() && self.workspace_menu_target() == Some(id) =>
            {
                self.present_close(close, window, cx);
            }
            None => self.show_close(Some(close), window, cx),
            _ => return false,
        }
        self.menu.page == Some(Page::ConfirmClose)
    }

    pub(super) fn send_workspace_close(
        &mut self,
        target: Target,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let request = (|| {
            if !self.menu_target_current()
                || !self.input_ready()
                || self.selected_core_group().is_some()
            {
                return Err(Error::StaleConnection);
            }
            target.request(self.live.latest_snapshot().ok_or(Error::NotConnected)?)?;
            // New metadata may await the next UI poll or its matching surface.
            let inbox = self.endpoints[self.selected_endpoint]
                .connection
                .inbox
                .try_lock()
                .map_err(|_| Error::ConnectionNotReady)?;
            if !inbox.status.is_connected() {
                return Err(Error::NotConnected);
            }
            target.request(inbox.latest_snapshot().ok_or(Error::NotConnected)?)
        })();
        let error = match request {
            Ok((method, params)) => {
                if self.request_focus_change(method.as_str(), None, |handle, boot| {
                    handle.request(boot, method, params)
                }) {
                    self.dismiss_menu(window, cx);
                    return;
                }
                fl!("close-modal-workspace-not-queued")
            }
            Err(error) => error.localize(),
        };
        if let Some(close) = &mut self.menu.close {
            close.error = Some(error);
        }
        cx.notify();
    }
}

#[cfg(test)]
mod tests;
