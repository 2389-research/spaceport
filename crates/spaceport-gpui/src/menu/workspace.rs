//! The workspace a menu row targets and its dialogs with the daemon.

// i18n: migrated
pub(super) mod popover;
mod render;
mod requests;

use super::{Page, WorkspaceAction, WorkspaceMenuAction};
use crate::{SpaceportWindow, dialog_input::DialogInput};
use gpui::{prelude::*, *};
use popover::Label;
use spaceport_client::{
    Method,
    protocol::{ClientShellSnapshot, ClientShellWorkspace, ClientShellWorktree},
};

pub(crate) struct WorkspaceTarget {
    pub(super) boot_id: String,
    pub(super) id: String,
    pub(super) label: String,
    pub(super) worktree: Option<ClientShellWorktree>,
    /// Whether linked worktrees of this parent's repository are open, so its
    /// row folds a group even when another parent closes on its own.
    pub(super) heads_group: bool,
    pub(super) branch: Option<String>,
    /// Whether the user muted the workspace's notifications when the menu
    /// opened, which picks what the toggle in its header does.
    pub(super) muted: bool,
}

impl WorkspaceTarget {
    pub(super) fn new(snapshot: &ClientShellSnapshot, workspace: &ClientShellWorkspace) -> Self {
        Self {
            boot_id: snapshot.boot_id.clone(),
            id: workspace.workspace_id.clone(),
            label: workspace.label.clone(),
            worktree: known_worktree(workspace),
            heads_group: heads_group(snapshot, workspace),
            branch: workspace.branch.clone(),
            muted: false,
        }
    }

    /// The worktree key this workspace heads, when linked checkouts hang off it.
    pub(super) fn group_key(&self) -> Option<&str> {
        self.worktree
            .as_ref()
            .filter(|_| self.heads_group)
            .map(|tree| tree.key.as_str())
    }

    pub(super) fn request(
        &self,
        snapshot: &ClientShellSnapshot,
        action: WorkspaceAction,
        text: &str,
    ) -> crate::Result<(Method, serde_json::Value)> {
        if snapshot.boot_id != self.boot_id
            || !snapshot
                .workspaces
                .iter()
                .any(|workspace| workspace.workspace_id == self.id)
        {
            return Err(crate::Error::StaleWorkspace);
        }
        let params = match action {
            WorkspaceAction::Rename => {
                let label = text.trim();
                if label.is_empty() {
                    return Err(crate::Error::EmptyWorkspaceLabel);
                }
                (
                    Method::WorkspaceRename,
                    serde_json::json!({"workspace_id": self.id, "label": label}),
                )
            }
            // The label, when there is one, is added by the dialog that knows
            // which name it proposed.
            WorkspaceAction::NewTab => (
                Method::TabCreate,
                serde_json::json!({"workspace_id": self.id, "focus": true}),
            ),
            WorkspaceAction::NewWorkspace => (
                Method::WorkspaceCreate,
                serde_json::json!({"focus": true, "source_workspace_id": self.id}),
            ),
            WorkspaceAction::Note => return Err(crate::Error::LocalWorktreeNote),
        };
        Ok(params)
    }
}

/// Whether `workspace` is a main checkout whose directory is no longer inside
/// the repository Herdr keeps for it. A linked checkout's own path is not in
/// the snapshot, so it is never judged.
fn left_checkout(workspace: &ClientShellWorkspace) -> bool {
    workspace.worktree.as_ref().is_some_and(|tree| {
        !tree.is_linked_worktree
            && crate::worktree::left_checkout(&tree.key, &workspace.new_workspace_cwd)
    })
}

/// The repository `workspace` is in, as far as it can be trusted: none for a
/// main checkout whose shell has left the repository Herdr keeps for it.
pub(super) fn known_worktree(workspace: &ClientShellWorkspace) -> Option<ClientShellWorktree> {
    workspace
        .worktree
        .clone()
        .filter(|_| !left_checkout(workspace))
}

/// The name a new tab dialog proposes: the next number in its workspace, as
/// Herdr numbers an unnamed tab.
pub(super) fn suggested_tab_name(snapshot: &ClientShellSnapshot, workspace: &str) -> String {
    let count = snapshot
        .tabs
        .iter()
        .filter(|tab| tab.workspace_id == workspace)
        .count();
    (count + 1).to_string()
}

/// The name a new workspace dialog proposes: the folder it opens in. Herdr
/// also consults Git there, which only the daemon's host can do, so leaving
/// this proposal unchanged still lets the daemon choose the real name.
pub(super) fn suggested_workspace_name(cwd: &str) -> String {
    let trimmed = cwd.trim_end_matches('/');
    match trimmed.rsplit('/').next() {
        Some(name) if !name.is_empty() => name.to_owned(),
        _ if cwd.is_empty() => "workspace".to_owned(),
        _ => "/".to_owned(),
    }
}

/// The label a creation sends. An empty name or the unchanged proposal sends
/// none, so the daemon names the tab or workspace as Herdr's own prompt does.
pub(super) fn chosen_label<'a>(text: &'a str, suggested: Option<&str>) -> Option<&'a str> {
    let text = text.trim();
    (!text.is_empty() && Some(text) != suggested).then_some(text)
}

/// Whether `workspace` is a parent with linked worktrees of its repository open.
fn heads_group(snapshot: &ClientShellSnapshot, workspace: &ClientShellWorkspace) -> bool {
    workspace
        .worktree
        .as_ref()
        .filter(|tree| !tree.is_linked_worktree)
        .is_some_and(|tree| {
            snapshot.workspaces.iter().any(|w| {
                w.worktree
                    .as_ref()
                    .is_some_and(|other| other.key == tree.key && other.is_linked_worktree)
            })
        })
}

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
    pub(super) fn workspace_items(&self) -> Vec<(WorkspaceMenuAction, Label)> {
        use WorkspaceMenuAction::Dialog;
        let Some(target) = &self.menu.target else {
            return vec![];
        };
        let mut items = vec![if target.muted {
            (WorkspaceMenuAction::Unmute, Label::UNMUTE)
        } else {
            (WorkspaceMenuAction::Mute, Label::MUTE)
        }];
        items.push((Dialog(WorkspaceAction::Rename), Label::RENAME));
        if self.note_checkout().is_some() {
            items.push((Dialog(WorkspaceAction::Note), Label::NOTE));
        }
        // Only a workspace that heads a group of checkouts can fold anything.
        if let Some(key) = target.group_key() {
            items.push(if self.collapsed_repos_for_selection().contains(key) {
                (WorkspaceMenuAction::Expand, Label::EXPAND_GROUP)
            } else {
                (WorkspaceMenuAction::Collapse, Label::COLLAPSE_GROUP)
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
        let actions: Vec<_> = self
            .workspace_items()
            .into_iter()
            .map(|(action, _)| action)
            .collect();
        actions
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
