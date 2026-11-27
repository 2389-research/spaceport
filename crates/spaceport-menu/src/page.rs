// ABOUTME: Defines the closed set of popup pages and workspace actions.
// ABOUTME: Keeps popup targets typed across rendering and input routing.
//! What a menu popup is currently showing, and the workspace actions a row
//! can trigger. Closed sets, so a page is never a string tag.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Page {
    // keep-sorted start
    About,
    AddDevice,
    /// Picking a WSL distribution to save as a device.
    AddWsl,
    /// The one-time offer to install the agent skill for browser tabs.
    AgentSkill,
    /// The Linux window's application menu, from the native menu definitions.
    Application,
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
    /// A daemon refused the handshake until one side is updated.
    VersionMismatch,
    Workspace,
    // keep-sorted end
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceAction {
    Rename,
    /// Names a tab before `tab.create`, when `ui.prompt_new_tab_name` asks.
    NewTab,
    /// Names a workspace before `workspace.create`, when
    /// `ui.prompt_new_workspace_name` asks. Targets the source workspace.
    NewWorkspace,
    /// Writes the note this app keeps on the checkout; never a daemon request.
    Note,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceMenuAction {
    Dialog(WorkspaceAction),
    /// Fold or unfold the worktree group this workspace heads. Applied at once:
    /// it changes only the sidebar's own view, never the daemon's state.
    Collapse,
    Expand,
    /// Silence the workspace's notifications, or let them through again.
    /// Drawn as the toggle in the popover's header rather than as a row, and
    /// applied at once: the mute belongs to this client, never the daemon.
    Mute,
    Unmute,
}

impl Page {
    /// Context menus open where the pointer asked for them, rather than
    /// centred over a dimmed window as a dialog is.
    pub fn pointer_anchored(self) -> bool {
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
    pub fn pane_menu(self) -> bool {
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
    pub fn in_header(self) -> bool {
        matches!(self, Self::Mute | Self::Unmute)
    }
}
