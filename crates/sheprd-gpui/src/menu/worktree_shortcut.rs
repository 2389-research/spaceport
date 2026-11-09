//! Herdr's `open_worktree` shortcut opens its dialog for the focused workspace.

// i18n: migrated
use super::{Page, WorkspaceAction, workspace::WorkspaceTarget};
use crate::{SheprdWindow, i18n::fl};
use gpui::{Context, Point, Window};
use sheprd_client::protocol::ClientShellSnapshot;

/// A worktree shortcut opening the workspace menu's dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorktreeShortcut {
    /// "Open worktree...", listing the repository's other checkouts.
    Open,
}

/// Why a worktree shortcut found no workspace to act on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Unavailable {
    Disconnected,
    NoWorkspace,
    NotGit,
    /// A linked checkout opens worktrees through its main checkout, which is
    /// not open.
    MainCheckoutClosed,
}

impl Unavailable {
    pub(super) fn message(self) -> String {
        match self {
            Self::Disconnected => fl!("worktree-popover-change-disconnected"),
            Self::NoWorkspace => fl!("worktree-popover-no-workspace"),
            Self::NotGit => fl!("worktree-popover-not-git"),
            Self::MainCheckoutClosed => fl!("worktree-popover-open-main-closed"),
        }
    }
}

impl WorktreeShortcut {
    fn action(self) -> WorkspaceAction {
        match self {
            Self::Open => WorkspaceAction::OpenWorktree,
        }
    }

    /// The workspace whose menu row this shortcut stands in for. Herdr lists
    /// and opens checkouts only through a repository's main checkout, so a
    /// focused linked checkout opens through it, as a new worktree does.
    pub(super) fn target(self, snapshot: &ClientShellSnapshot) -> Result<String, Unavailable> {
        let focused = snapshot
            .workspaces
            .iter()
            .find(|w| Some(&w.workspace_id) == snapshot.focused_workspace_id.as_ref())
            .ok_or(Unavailable::NoWorkspace)?;
        // The same tests the menu uses to offer each row.
        let target = WorkspaceTarget::new(snapshot, focused);
        if !target.can_create() && !target.is_linked_checkout() {
            return Err(Unavailable::NotGit);
        }
        let linked = focused
            .worktree
            .as_ref()
            .filter(|tree| tree.is_linked_worktree);
        let row = match linked {
            Some(tree) => super::workspace::main_checkout(snapshot, tree)
                // A main checkout whose shell wandered off would list the
                // repository it moved to, so the dialog opens through the
                // linked checkout and names the repository's root instead.
                .or_else(|| {
                    snapshot
                        .workspaces
                        .iter()
                        .any(|w| super::workspace::is_main_of(w, tree))
                        .then_some(focused)
                })
                .ok_or(Unavailable::MainCheckoutClosed)?,
            None => focused,
        };
        Ok(row.workspace_id.clone())
    }
}

impl SheprdWindow {
    /// Opens the dialog `shortcut` names for the focused workspace, as its
    /// menu row would. When
    /// there is no such row, a flash says why rather than the shortcut doing
    /// nothing visible.
    pub(crate) fn open_worktree_shortcut(
        &mut self,
        shortcut: WorktreeShortcut,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let target = match &self.live.snapshot {
            Some(snapshot) if self.live.status.is_connected() => shortcut.target(snapshot),
            _ => Err(Unavailable::Disconnected),
        };
        let id = match target {
            Ok(id) => id,
            Err(reason) => {
                self.show_flash(crate::window::Flash::warning(reason.message()), cx);
                return;
            }
        };
        self.open_workspace_menu(&id, Point::default(), window, cx);
        if self.menu.page == Some(Page::Workspace) {
            self.open_workspace_dialog(shortcut.action(), window, cx);
        }
    }
}
