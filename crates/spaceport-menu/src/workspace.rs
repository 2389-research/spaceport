//! The workspace a menu row targets, the request its dialogs send, and the
//! popover its actions are drawn in.

use crate::{Error, Result};
use serde_json::json;
use spaceport_client::{
    Method,
    protocol::{ClientShellSnapshot, ClientShellWorkspace, ClientShellWorktree},
};

use super::page::WorkspaceAction;

pub mod popover;

/// The workspace one menu is about, captured from the snapshot it opened under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceTarget {
    pub boot_id: String,
    pub id: String,
    pub label: String,
    pub worktree: Option<ClientShellWorktree>,
    /// Whether linked worktrees of this parent's repository are open, so its
    /// row folds a group even when another parent closes on its own.
    pub heads_group: bool,
    pub branch: Option<String>,
    /// Whether the user muted the workspace's notifications when the menu
    /// opened, which picks what the toggle in its header does.
    pub muted: bool,
}

impl WorkspaceTarget {
    pub fn new(snapshot: &ClientShellSnapshot, workspace: &ClientShellWorkspace) -> Self {
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
    pub fn group_key(&self) -> Option<&str> {
        self.worktree
            .as_ref()
            .filter(|_| self.heads_group)
            .map(|tree| tree.key.as_str())
    }

    /// The request `action` sends for this workspace, or the reason it sends
    /// none: the workspace must still be in `snapshot` as this menu saw it.
    pub fn request(
        &self,
        snapshot: &ClientShellSnapshot,
        action: WorkspaceAction,
        text: &str,
    ) -> Result<(Method, serde_json::Value)> {
        if snapshot.boot_id != self.boot_id
            || !snapshot
                .workspaces
                .iter()
                .any(|workspace| workspace.workspace_id == self.id)
        {
            return Err(Error::StaleWorkspace);
        }
        let params = match action {
            WorkspaceAction::Rename => {
                let label = text.trim();
                if label.is_empty() {
                    return Err(Error::EmptyWorkspaceLabel);
                }
                (
                    Method::WorkspaceRename,
                    json!({"workspace_id": self.id, "label": label}),
                )
            }
            // The label, when there is one, is added by the dialog that knows
            // which name it proposed.
            WorkspaceAction::NewTab => (
                Method::TabCreate,
                json!({"workspace_id": self.id, "focus": true}),
            ),
            WorkspaceAction::NewWorkspace => (
                Method::WorkspaceCreate,
                json!({"focus": true, "source_workspace_id": self.id}),
            ),
            WorkspaceAction::Note => return Err(Error::LocalWorktreeNote),
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
fn known_worktree(workspace: &ClientShellWorkspace) -> Option<ClientShellWorktree> {
    workspace
        .worktree
        .clone()
        .filter(|_| !left_checkout(workspace))
}

/// The name a new tab dialog proposes: the next number in its workspace, as
/// Herdr numbers an unnamed tab.
pub fn suggested_tab_name(snapshot: &ClientShellSnapshot, workspace: &str) -> String {
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
pub fn suggested_workspace_name(cwd: &str) -> String {
    let trimmed = cwd.trim_end_matches('/');
    match trimmed.rsplit('/').next() {
        Some(name) if !name.is_empty() => name.to_owned(),
        _ if cwd.is_empty() => "workspace".to_owned(),
        _ => "/".to_owned(),
    }
}

/// The label a creation sends. An empty name or the unchanged proposal sends
/// none, so the daemon names the tab or workspace as Herdr's own prompt does.
pub fn chosen_label<'a>(text: &'a str, suggested: Option<&str>) -> Option<&'a str> {
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
