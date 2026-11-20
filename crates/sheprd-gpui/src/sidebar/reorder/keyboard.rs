// ABOUTME: Reorders the focused workspace row with Alt and an arrow key.
// ABOUTME: Reuses drag grouping and fences callbacks to their rendered endpoint.
use super::Plan;
use crate::SheprdWindow;
use gpui::{Context, KeyDownEvent, Modifiers, StatefulInteractiveElement};

struct RowTarget {
    endpoint: String,
    workspace: String,
    boot: String,
    epoch: u64,
    generation: u64,
}

impl RowTarget {
    fn capture(view: &SheprdWindow, endpoint: &str, workspace: &str) -> Option<Self> {
        let selected = &view.endpoints[view.selected_endpoint];
        let snapshot = view
            .live
            .snapshot
            .as_ref()
            .filter(|_| selected.id == endpoint)?;
        Some(Self {
            endpoint: endpoint.to_owned(),
            workspace: workspace.to_owned(),
            boot: snapshot.boot_id.clone(),
            epoch: view.selection_epoch,
            generation: selected.generation,
        })
    }
}

impl SheprdWindow {
    pub(in crate::sidebar) fn workspace_reorder_keys<E: StatefulInteractiveElement>(
        &self,
        row: E,
        endpoint: &str,
        workspace: &str,
        cx: &Context<Self>,
    ) -> E {
        let Some(target) = RowTarget::capture(self, endpoint, workspace) else {
            return row;
        };
        row.aria_keyshortcuts("Alt+ArrowUp Alt+ArrowDown") // i18n: data
            .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.modifiers != Modifiers::alt() {
                    return;
                }
                let forward = match event.keystroke.key.as_str() {
                    "up" => false,
                    "down" => true,
                    _ => return,
                };
                cx.stop_propagation();
                window.prevent_default();
                if !event.is_held {
                    this.move_workspace_row(&target, forward, cx);
                }
            }))
    }

    fn move_workspace_row(&mut self, target: &RowTarget, forward: bool, cx: &mut Context<Self>) {
        let selected = &self.endpoints[self.selected_endpoint];
        if selected.id != target.endpoint
            || self.selection_epoch != target.epoch
            || selected.generation != target.generation
            || self.menu.page.is_some()
            || self.menu.application.is_some()
            || self.workspace_drag.is_some()
            || self.device_drag.is_some()
            || !self.live.status.is_connected()
        {
            return;
        }
        let Some(snapshot) = self
            .live
            .snapshot
            .as_ref()
            .filter(|snapshot| snapshot.boot_id == target.boot)
        else {
            return;
        };
        let Some(plan) = Plan::new(&snapshot.workspaces, &target.workspace) else {
            return;
        };
        let slot = if forward {
            plan.dragged() + 2
        } else {
            let Some(slot) = plan.dragged().checked_sub(1) else {
                return;
            };
            slot
        };
        let Some(request) = plan.request(&snapshot.workspaces, slot) else {
            return;
        };
        let Some(handle) = &selected.connection.handle else {
            return;
        };
        let reorder = request.reorder(&snapshot.workspaces, self.live.supports_workspace_move);
        if let Err(error) = handle.reorder_workspaces(&target.boot, &reorder) {
            self.notices.show(
                crate::notice::Cause::Command,
                crate::i18n::fl!("sidebar-workspace-move-not-sent", error = error.to_string()),
            );
            cx.notify();
        }
    }
}

#[cfg(test)]
mod tests;
