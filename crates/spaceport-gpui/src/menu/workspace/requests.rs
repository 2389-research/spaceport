//! Carrying a workspace dialog's request to the daemon.

use super::chosen_label;
use crate::{
    SpaceportWindow,
    menu::{Page, WorkspaceAction},
};
use gpui::*;

impl SpaceportWindow {
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
            let text = self
                .menu
                .input
                .as_ref()
                .map(|input| input.text.as_str())
                .unwrap_or("");
            let (method, mut params) = target.request(snapshot, action, text)?;
            if matches!(
                action,
                WorkspaceAction::NewTab | WorkspaceAction::NewWorkspace
            ) && let Some(label) = chosen_label(text, self.menu.suggested_name.as_deref())
            {
                params["label"] = label.into();
            }
            let handle = self.endpoints[self.selected_endpoint]
                .connection
                .handle
                .as_ref()
                .ok_or(crate::Error::NotConnected)?;
            handle
                .request(&target.boot_id, method, params)
                .map(|_| action != WorkspaceAction::Rename)
                .map_err(|source| crate::Error::Request { method, source })
        })();
        match result {
            Ok(focus_changed) => {
                self.notices.clear();
                if focus_changed {
                    self.fence_focus_change(None);
                }
                self.dismiss_menu(window, cx);
            }
            Err(error) => {
                self.menu.error = Some(error.to_string());
                cx.notify();
            }
        }
    }
}
