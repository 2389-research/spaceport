//! Keeps each window's groups in step with tabs in its selected workspace.
//! Core groups and legacy workspaces use the same layout owner and picks.

use super::{GroupId, Pick};
use crate::SheprdWindow;
use gpui::{prelude::*, *};

impl SheprdWindow {
    /// Keeps the focused workspace's groups in step with its tabs, on every
    /// tick: forgets the tabs that closed, names the group in use as the
    /// owner of each tab that has none, gives a group whose tab closed
    /// another of its own, and closes a group whose last tab closed.
    pub(crate) fn settle_groups(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(key) = self.active_browser_key(cx) else {
            return;
        };
        if self.selected_core_group().is_none()
            && self
                .live
                .snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.focused_workspace_id.as_ref())
                != Some(&key.1)
        {
            return;
        }
        let open = self.open_workspace_tabs(cx);
        self.browser.hidden.retain_open(&key, &open);
        let tabs = self.workspace_tabs(cx);
        let Some(layout) = self.browser.layouts.get_mut(&key) else {
            return;
        };
        let closed: Vec<Pick> = layout
            .tabs()
            .filter(|pick| !tabs.contains(pick))
            .cloned()
            .collect();
        // With the groups whose tab was found closed before this ran.
        let mut emptied: Vec<GroupId> = layout
            .slots()
            .map(|slot| slot.id)
            .filter(|group| self.browser.emptied.contains(group))
            .collect();
        self.browser
            .emptied
            .retain(|group| !emptied.contains(group));
        for pick in &closed {
            emptied.extend(layout.forget(pick));
        }
        layout.adopt(tabs);
        let mut changed = !closed.is_empty();
        if layout.len() > 1 {
            for slot in self.group_slots() {
                if self.group_pick(slot.id).is_some() {
                    continue;
                }
                match self.group_tabs(slot.id, cx).into_iter().next() {
                    Some(next) => {
                        if let Some(layout) = self.browser.layouts.get_mut(&key) {
                            layout.show(slot.id, next);
                        }
                        // The window's connection goes to its group's tab;
                        // a parked one is steered there as it is reconciled.
                        self.steer_primary_group(slot.id, cx);
                        changed = true;
                    }
                    None if emptied.contains(&slot.id) => {
                        self.close_group(slot.id, window, cx);
                        changed = true;
                    }
                    None => {}
                }
            }
        }
        if changed {
            cx.notify();
        }
    }
}
