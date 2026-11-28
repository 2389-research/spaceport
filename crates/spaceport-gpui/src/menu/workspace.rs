//! The workspace a menu row targets and its dialogs with the daemon.

// i18n: migrated
pub(super) mod popover;
mod render;
mod requests;

use super::{Page, WorkspaceAction, WorkspaceMenuAction};
use crate::{SpaceportWindow, dialog_input::DialogInput};
use gpui::{prelude::*, *};
use popover::{Item, Label};

pub(crate) use spaceport_menu::workspace::{
    WorkspaceTarget, chosen_label, suggested_tab_name, suggested_workspace_name,
};

impl SpaceportWindow {
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    pub(crate) fn native_workspace_dialog_steps(&self, action: WorkspaceAction) -> Option<usize> {
        let actions = self.workspace_menu_actions();
        let target = actions
            .iter()
            .position(|item| *item == WorkspaceMenuAction::Dialog(action))?;
        let selected = self
            .menu
            .workspace_selected
            .and_then(|selected| actions.iter().position(|item| *item == selected));
        Some(selected.map_or(target + 1, |selected| {
            (target + actions.len() - selected) % actions.len()
        }))
    }

    /// The selected endpoint's workspace a workspace menu is open for. A
    /// hover menu can target a row that is not focused, so the row marks
    /// itself while the pointer is over the menu instead.
    pub(crate) fn workspace_menu_target(&self) -> Option<&str> {
        if self.menu.page != Some(Page::Workspace) {
            return None;
        }
        self.menu.target.as_ref().map(|target| target.id.as_str())
    }

    pub(crate) fn open_workspace_menu(
        &mut self,
        id: &str,
        anchor: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.menu.page.is_some() || !self.live.status.is_connected() {
            return;
        }
        let Some(snapshot) = &self.live.snapshot else {
            return;
        };
        let Some(workspace) = snapshot.workspaces.iter().find(|w| w.workspace_id == id) else {
            return;
        };
        self.hover = None;
        self.hover_menu = None;
        self.menu.reset();
        self.menu.endpoint_target = (
            self.selection_epoch,
            self.endpoints[self.selected_endpoint].generation,
        );
        let muted = self.muted_on_selected(crate::notification_mutes::Scope::Workspace, id, cx);
        self.menu.target = Some(WorkspaceTarget {
            muted,
            ..WorkspaceTarget::new(snapshot, workspace)
        });
        self.menu.anchor = anchor;
        self.menu.page = Some(Page::Workspace);
        self.discard_composition(cx);
        window.focus(&self.menu.focus, cx);
        cx.notify();
    }

    /// Opens the focused workspace's dialog as its menu row would.
    pub(crate) fn open_focused_workspace_dialog(
        &mut self,
        action: WorkspaceAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(id) = self
            .live
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.focused_workspace_id.clone())
        else {
            return;
        };
        self.open_workspace_dialog_for(&id, action, Point::default(), window, cx);
    }

    /// Opens `id`'s dialog for `action`, as its menu's row would, so the
    /// dialog checks and targets the workspace exactly as the menu does.
    pub(crate) fn open_workspace_dialog_for(
        &mut self,
        id: &str,
        action: WorkspaceAction,
        anchor: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_workspace_menu(id, anchor, window, cx);
        if self.menu.page == Some(Page::Workspace) {
            self.open_workspace_dialog(action, cx);
        }
    }

    /// Asks for a name before `command` creates a tab or workspace, when the
    /// local Herdr config's `ui.prompt_new_tab_name` or
    /// `ui.prompt_new_workspace_name` says to. Returns whether the dialog
    /// opened; otherwise the caller creates at once.
    pub(crate) fn open_name_prompt(
        &mut self,
        command: crate::controls::Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        use crate::controls::Command;
        let prompts = self
            .settings
            .shared
            .as_ref()
            .map(|shared| shared.name_prompts)
            .unwrap_or_default();
        let action = match command {
            Command::Tab if prompts.tab => WorkspaceAction::NewTab,
            Command::Workspace if prompts.workspace => WorkspaceAction::NewWorkspace,
            _ => return false,
        };
        // Only where creating at once would be allowed, so the prompt never
        // offers what the command itself would refuse.
        if self.live.activation.fenced()
            || !self.endpoints[self.selected_endpoint].surface_requested()
        {
            return false;
        }
        let Some(id) = self
            .live
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.focused_workspace_id.clone())
        else {
            return false;
        };
        self.open_workspace_menu(&id, Point::default(), window, cx);
        if self.menu.page != Some(Page::Workspace) {
            return false;
        }
        self.open_workspace_dialog(action, cx);
        true
    }

    /// The actions this workspace offers, in the order the popover shows them
    /// and arrow keys walk them: the header's mute toggle, the tile grid row
    /// by row, then the plain rows.
    pub(super) fn workspace_items(&self) -> Vec<Item> {
        use WorkspaceMenuAction::Dialog;
        let Some(target) = &self.menu.target else {
            return vec![];
        };
        let mut items = vec![if target.muted {
            popover::item(
                WorkspaceMenuAction::Unmute,
                Label::UNMUTE,
                Some(Label::UNMUTE_BUTTON),
            )
        } else {
            popover::item(
                WorkspaceMenuAction::Mute,
                Label::MUTE,
                Some(Label::MUTE_BUTTON),
            )
        }];
        items.push(popover::item(
            Dialog(WorkspaceAction::Rename),
            Label::RENAME,
            Some(Label::RENAME_TILE),
        ));
        if self.note_checkout().is_some() {
            items.push(popover::item(
                Dialog(WorkspaceAction::Note),
                Label::NOTE,
                None,
            ));
        }
        // Only a workspace that heads a group of checkouts can fold anything.
        if let Some(key) = target.group_key() {
            items.push(if self.collapsed_repos_for_selection().contains(key) {
                popover::item(WorkspaceMenuAction::Expand, Label::EXPAND_GROUP, None)
            } else {
                popover::item(WorkspaceMenuAction::Collapse, Label::COLLAPSE_GROUP, None)
            });
        }
        items
    }

    /// The collapsed set the sidebar paints for the selected endpoint.
    pub(super) fn collapsed_repos_for_selection(&self) -> &std::collections::HashSet<String> {
        &self.endpoints[self.selected_endpoint].collapsed_repos
    }

    /// The collapsed set the sidebar paints for the selected endpoint.
    pub(super) fn collapsed_repos_mut(&mut self) -> &mut std::collections::HashSet<String> {
        &mut self.endpoints[self.selected_endpoint].collapsed_repos
    }

    pub(super) fn toggle_selected_group(&mut self, cx: &mut Context<Self>) {
        let Some(key) = self
            .menu
            .target
            .as_ref()
            .and_then(|target| target.group_key())
            .map(str::to_owned)
        else {
            return;
        };
        let collapsed = self.collapsed_repos_mut();
        if !collapsed.remove(&key) {
            collapsed.insert(key);
        }
        self.menu.reset();
        cx.notify();
    }

    pub(super) fn open_workspace_dialog(
        &mut self,
        action: WorkspaceAction,
        cx: &mut Context<Self>,
    ) {
        let note = (action == WorkspaceAction::Note).then(|| self.note_draft(cx));
        let Some(target) = &self.menu.target else {
            return;
        };
        let workspace = self.live.snapshot.as_ref().and_then(|snapshot| {
            let workspace = snapshot
                .workspaces
                .iter()
                .find(|w| w.workspace_id == target.id && snapshot.boot_id == target.boot_id)?;
            Some((snapshot, workspace))
        });
        // Proposed selected, as Herdr does, so typing replaces it.
        self.menu.suggested_name = match (action, workspace) {
            (WorkspaceAction::NewTab, Some((snapshot, _))) => {
                Some(suggested_tab_name(snapshot, &target.id))
            }
            (WorkspaceAction::NewWorkspace, Some((_, workspace))) => {
                Some(suggested_workspace_name(&workspace.new_workspace_cwd))
            }
            _ => None,
        };
        self.menu.input = match action {
            WorkspaceAction::Rename => Some(DialogInput::new(target.label.clone())),
            WorkspaceAction::NewTab | WorkspaceAction::NewWorkspace => Some(DialogInput::new(
                self.menu.suggested_name.clone().unwrap_or_default(),
            )),
            // Propose the daemon's own branch shape, selected so typing replaces it.
            WorkspaceAction::Note => note.map(DialogInput::new),
        };
        self.menu.page = Some(Page::Dialog(action));
        self.menu.error = None;
        cx.notify();
    }

    pub(super) fn workspace_menu_actions(&self) -> Vec<WorkspaceMenuAction> {
        self.workspace_items()
            .into_iter()
            .map(|item| item.action)
            .collect()
    }

    pub(super) fn activate_workspace_menu(
        &mut self,
        action: WorkspaceMenuAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            WorkspaceMenuAction::Dialog(action) => self.open_workspace_dialog(action, cx),
            WorkspaceMenuAction::Collapse | WorkspaceMenuAction::Expand => {
                self.toggle_selected_group(cx)
            }
            WorkspaceMenuAction::Mute | WorkspaceMenuAction::Unmute => {
                if let Some(target) = &self.menu.target {
                    let (id, boot) = (target.id.clone(), target.boot_id.clone());
                    self.set_muted_on_selected(
                        crate::notification_mutes::Scope::Workspace,
                        &id,
                        &boot,
                        action == WorkspaceMenuAction::Mute,
                        cx,
                    );
                }
                self.dismiss_menu(window, cx);
            }
        }
    }
}
