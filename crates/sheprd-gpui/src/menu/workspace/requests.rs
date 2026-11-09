//! Carrying a workspace dialog's request to the daemon and applying its answer.

use super::chosen_label;
use crate::{
    NavigationTarget, SheprdWindow,
    i18n::fl,
    menu::{Page, Submission, WorkspaceAction, endpoint_error},
};
use gpui::*;

impl SheprdWindow {
    /// Apply the daemon's answer to the open worktree dialog.
    pub(crate) fn update_workspace_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.menu.worktree_open.is_some() && !self.worktree_open_current() {
            self.dismiss_menu(window, cx);
            return;
        }
        if self
            .menu
            .worktree_open
            .as_ref()
            .is_some_and(|picker| picker.pending.is_some())
        {
            if let Some((id, Some(result))) = &self.live.dialog_response {
                self.menu.apply_worktree_list_response(id, result.clone());
                cx.notify();
            }
            return;
        }
        if self.menu.opening.is_none() {
            return;
        }
        if !self.menu_target_current()
            || !self.live.status.is_connected()
            || self.menu.target.as_ref().is_some_and(|target| {
                self.live
                    .snapshot
                    .as_ref()
                    .is_none_or(|snapshot| snapshot.boot_id != target.boot_id)
            })
        {
            self.menu.reset();
            return;
        }
        let Some((id, Some(result))) = &self.live.dialog_response else {
            return;
        };
        if self.menu.opening.as_deref() != Some(id.as_str()) {
            return;
        }
        let result = result.clone();
        self.menu.opening = None;
        self.apply_open_response(result, window, cx);
    }

    /// Follow an opened checkout once its correlated answer arrives.
    pub(in crate::menu) fn apply_open_response(
        &mut self,
        result: crate::state::DialogResponse,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let response = match result {
            Ok(response) => response,
            Err(error) => {
                self.menu.error = Some(error.to_string());
                cx.notify();
                return;
            }
        };
        if let Some(error) = response.get("error") {
            let (code, message) = endpoint_error(error);
            self.menu.error = Some(format!("{code}: {message}"));
            cx.notify();
            return;
        }
        let result = &response["result"];
        let Some(opened) = (result["type"] == "worktree_opened")
            .then(|| result["workspace"]["workspace_id"].as_str())
            .flatten()
            .filter(|id| !id.is_empty())
        else {
            self.menu.error = Some(fl!("workspace-dialog-unexpected"));
            cx.notify();
            return;
        };
        let endpoint = self.endpoints[self.selected_endpoint].id.clone();
        let group = self
            .menu
            .target
            .as_ref()
            .and_then(|target| target.worktree.as_ref())
            .map(|worktree| worktree.key.clone())
            .or_else(|| {
                self.menu
                    .worktree_open
                    .as_ref()?
                    .source
                    .as_ref()
                    .map(|source| source.repo_key.clone())
            });
        self.dismiss_menu(window, cx);
        if let Some(group) = group {
            self.collapsed_repos_mut().remove(&group);
        }
        self.navigate_endpoint(&endpoint, NavigationTarget::Workspace(opened), cx);
    }

    pub(in crate::menu) fn submit_workspace_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(Page::Dialog(action)) = self.menu.page else {
            return;
        };
        if action == WorkspaceAction::Note {
            self.save_worktree_note(window, cx);
            return;
        }
        if action == WorkspaceAction::OpenWorktree
            && self
                .menu
                .worktree_open
                .as_ref()
                .is_some_and(|picker| picker.search.read(cx).is_composing())
        {
            return;
        }
        if self
            .menu
            .input
            .as_ref()
            .is_some_and(|input| input.marked.is_some())
        {
            return;
        }
        let result = (|| {
            if !self.menu_target_current() {
                return Err(crate::Error::StaleConnection);
            }
            if !self.live.status.is_connected() {
                return Err(crate::Error::NotConnected);
            }
            let target = self
                .menu
                .target
                .as_ref()
                .ok_or(crate::Error::StaleWorkspace)?;
            let snapshot = self
                .live
                .snapshot
                .as_ref()
                .ok_or(crate::Error::NoSnapshot)?;
            let text = if action == WorkspaceAction::OpenWorktree {
                self.menu
                    .worktree_open
                    .as_ref()
                    .filter(|picker| picker.pending.is_none())
                    .and_then(|picker| picker.entry(picker.selected))
                    .map(|entry| entry.path.as_str())
                    .ok_or(crate::Error::WorktreeSelection)?
            } else {
                self.menu
                    .input
                    .as_ref()
                    .map(|input| input.text.as_str())
                    .unwrap_or("")
            };
            let (method, mut params) = target.request(snapshot, action, text)?;
            if matches!(
                action,
                WorkspaceAction::NewTab | WorkspaceAction::NewWorkspace
            ) && let Some(label) = chosen_label(text, self.menu.suggested_name.as_deref())
            {
                params["label"] = label.into();
            }
            if action == WorkspaceAction::OpenWorktree {
                if self.menu.opening.is_some() {
                    return Ok(Submission::Awaiting {
                        focus_changed: false,
                    });
                }
                // Correlated, so the daemon's failure reaches the dialog and the
                // opened checkout can be focused once it appears in the snapshot.
                let boot = target.boot_id.clone();
                let id = self.endpoints[self.selected_endpoint]
                    .connection
                    .request_dialog(&boot, method, params)?;
                self.menu.opening = Some(id);
                self.menu.error = None;
                return Ok(Submission::Awaiting {
                    focus_changed: true,
                });
            }
            let handle = self.endpoints[self.selected_endpoint]
                .connection
                .handle
                .as_ref()
                .ok_or(crate::Error::NotConnected)?;
            handle
                .request(&target.boot_id, method, params)
                .map(|_| Submission::Queued {
                    focus_changed: action != WorkspaceAction::Rename,
                })
                .map_err(|source| crate::Error::Request { method, source })
        })();
        match result {
            Ok(submission) => {
                self.local_error = None;
                if submission.focus_changed() {
                    self.fence_focus_change(None);
                }
                match submission {
                    Submission::Queued { .. } => self.dismiss_menu(window, cx),
                    Submission::Awaiting { .. } => cx.notify(),
                }
            }
            Err(error) => {
                self.menu.error = Some(error.to_string());
                cx.notify();
            }
        }
    }
}
