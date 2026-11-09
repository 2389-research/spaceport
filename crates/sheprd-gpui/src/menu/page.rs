//! What a menu popup is currently showing, and the workspace actions a row
//! can trigger. Closed sets, so a page is never a string tag.
// i18n: migrated

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Page {
    // keep-sorted start
    About,
    AddDevice,
    /// Picking a WSL distribution to save as a device.
    AddWsl,
    /// The one-time offer to install the agent skill for browser tabs.
    AgentSkill,
    /// The Linux window's application menu, from the native menu definitions.
    Application,
    AppUpdate,
    ConfirmClose,
    CoreWorkspaceActions,
    CoreWorkspaceCreate,
    CoreWorkspaceDelete,
    CoreWorkspaceRename,
    Devices,
    /// Groups a device by tag.
    DeviceTags,
    /// Picks a saved SSH device's terminal theme.
    DeviceTheme,
    Dialog(WorkspaceAction),
    /// Names a saved SSH device's port to forward to this computer.
    ForwardPort,
    GitHub,
    /// A group's "…" menu: opening tabs other than a terminal, closing
    /// tabs, and splitting.
    Group,
    /// A saved SSH device's context menu, from its sidebar host header.
    Host,
    Keybinds,
    /// Confirming the processes chosen there should end.
    KillProcesses,
    Menu,
    /// Where the pane menu's pane can move: new places, then other tabs.
    MovePane,
    /// Where the tab menu's tab can move: a new workspace, then other workspaces.
    MoveTab,
    /// Asks before opening a terminal hyperlink with an unlisted scheme.
    OpenLink,
    Palette,
    Pane,
    /// The processes under the pane menu's pane.
    PaneProcesses,
    RemoveDevice,
    /// Confirming a saved WSL distribution should be forgotten.
    RemoveWsl,
    RenameDevice,
    RenamePane,
    RenameTab,
    Tab,
    Themes,
    Update,
    /// Plan usage details for one agent on the selected host.
    Usage(crate::usage::Provider),
    /// A daemon refused the handshake until one side is updated.
    VersionMismatch,
    Workspace,
    /// Asks whether to trust a repository's worktree script before it runs.
    WorktreeScript,
    // keep-sorted end
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkspaceAction {
    Rename,
    Close,
    OpenWorktree,
    DeleteWorktree,
    /// Names a tab before `tab.create`, when `ui.prompt_new_tab_name` asks.
    NewTab,
    /// Names a workspace before `workspace.create`, when
    /// `ui.prompt_new_workspace_name` asks. Targets the source workspace.
    NewWorkspace,
    /// Writes the note this app keeps on the checkout; never a daemon request.
    Note,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkspaceMenuAction {
    Dialog(WorkspaceAction),
    /// Fold or unfold the worktree group this workspace heads. Applied at once:
    /// it changes only the sidebar's own view, never the daemon's state.
    Collapse,
    Expand,
    PullRequest,
    /// Run one of the repository's worktree scripts in a new tab.
    Script(crate::worktree_scripts::ScriptKind),
    /// Silence the workspace's notifications, or let them through again.
    /// Drawn as the toggle in the popover's header rather than as a row, and
    /// applied at once: the mute belongs to this client, never the daemon.
    Mute,
    Unmute,
}

impl Page {
    /// Context menus open where the pointer asked for them, rather than
    /// centred over a dimmed window as a dialog is.
    pub(crate) fn pointer_anchored(self) -> bool {
        matches!(
            self,
            Self::Workspace
                | Self::CoreWorkspaceActions
                | Self::Tab
                | Self::RenameTab
                | Self::MoveTab
                | Self::Group
                | Self::Pane
                | Self::RenamePane
                | Self::MovePane
                | Self::PaneProcesses
                | Self::KillProcesses
                | Self::Host
                | Self::RemoveDevice
                | Self::RemoveWsl
        )
    }

    /// The pane menu and the pages it opens, all about its one pane.
    pub(crate) fn pane_menu(self) -> bool {
        matches!(
            self,
            Self::Pane
                | Self::RenamePane
                | Self::MovePane
                | Self::PaneProcesses
                | Self::KillProcesses
        )
    }
}

impl WorkspaceMenuAction {
    /// Whether the popover draws the action in its header, not with the
    /// tiles and rows.
    pub(crate) fn in_header(self) -> bool {
        matches!(self, Self::Mute | Self::Unmute)
    }

    /// Embedded icon for the row, so each action is recognizable before reading.
    /// The pull request section draws its own header rather than a menu row.
    pub(crate) fn icon(self) -> Option<&'static str> {
        Some(match self {
            Self::Dialog(WorkspaceAction::Rename) => "icons/pencil.svg",
            Self::Dialog(WorkspaceAction::Close) => "icons/close.svg",
            Self::Dialog(WorkspaceAction::OpenWorktree) => "icons/chevron-down.svg",
            Self::Dialog(WorkspaceAction::DeleteWorktree) => "icons/trash.svg",
            Self::Dialog(WorkspaceAction::Note) => "icons/note.svg",
            Self::Dialog(WorkspaceAction::NewTab | WorkspaceAction::NewWorkspace) => {
                "icons/plus.svg"
            }
            Self::Collapse => "icons/chevron-up.svg",
            Self::Expand => "icons/chevron-down.svg",
            Self::Script(crate::worktree_scripts::ScriptKind::Setup) => "icons/refresh.svg",
            Self::Script(_) => "icons/play.svg",
            Self::Mute => crate::icons::MUTED,
            Self::Unmute => "icons/bell.svg",
            Self::PullRequest => return None,
        })
    }
}
