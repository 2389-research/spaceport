//! The tab menu's target and its rows. A target is captured from the snapshot
//! the menu opened under, and a move is refused unless the tab's panes are
//! still the ones it captured.

use crate::{Error, Result};
use serde_json::{Value, json};
use spaceport_client::protocol::ClientShellSnapshot;

/// The tab one menu is about, captured from the snapshot it opened under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub boot: String,
    pub workspace: String,
    pub tab: String,
    pub label: String,
    /// The tab's panes in snapshot order, moved together to another workspace.
    pub panes: Vec<String>,
}

/// What the tab's surroundings offer its menu beyond the tab itself: whether
/// this endpoint's host can run `herdr pane move`, and the mute row that says
/// how the tab, its workspace and its device are muted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offers {
    pub movable: bool,
    pub mute: Action,
}

impl Target {
    pub fn capture(snapshot: &ClientShellSnapshot, id: &str) -> Option<Self> {
        let tab = snapshot.tabs.iter().find(|tab| tab.tab_id == id)?;
        let panes = snapshot
            .panes
            .iter()
            .filter(|pane| pane.tab_id == tab.tab_id && pane.workspace_id == tab.workspace_id)
            .map(|pane| pane.pane_id.clone())
            .collect();
        Some(Self {
            boot: snapshot.boot_id.clone(),
            workspace: tab.workspace_id.clone(),
            tab: tab.tab_id.clone(),
            label: tab.label.clone(),
            panes,
        })
    }

    /// Whether `snapshot` still holds the workspace and tab this menu captured.
    pub fn validate(&self, snapshot: &ClientShellSnapshot) -> Result<()> {
        if snapshot.boot_id != self.boot
            || !snapshot
                .workspaces
                .iter()
                .any(|w| w.workspace_id == self.workspace)
            || !snapshot
                .tabs
                .iter()
                .any(|t| t.tab_id == self.tab && t.workspace_id == self.workspace)
        {
            return Err(Error::StaleTab);
        }
        Ok(())
    }

    /// Whether `snapshot` still holds the tab and exactly the panes a move
    /// carries out of it.
    pub fn validate_move(&self, snapshot: &ClientShellSnapshot) -> Result<()> {
        self.validate(snapshot)?;
        let current = snapshot
            .panes
            .iter()
            .filter(|p| p.tab_id == self.tab && p.workspace_id == self.workspace);
        if self.panes.is_empty()
            || current.clone().count() != self.panes.len()
            || !self
                .panes
                .iter()
                .all(|id| current.clone().any(|p| p.pane_id == *id))
        {
            return Err(Error::StaleTab);
        }
        Ok(())
    }

    pub fn rename_params(&self, label: &str) -> Result<Value> {
        let label = label.trim();
        if label.is_empty() {
            return Err(Error::EmptyTabName);
        }
        Ok(json!({"tab_id": self.tab, "label": label}))
    }

    /// The rows this menu offers: moving to a workspace needs a host the
    /// `herdr` CLI can run on and at least one pane in the tab. This client's
    /// mute row comes before closing, which stays last.
    pub fn actions(&self, offers: Offers) -> Vec<Action> {
        let mut actions: Vec<_> = ACTIONS
            .into_iter()
            .filter(|action| match action {
                Action::MoveToWorkspace => offers.movable && !self.panes.is_empty(),
                _ => true,
            })
            .collect();
        let close = actions
            .iter()
            .position(|action| *action == Action::Close)
            .unwrap_or(actions.len());
        actions.insert(close, offers.mute);
        actions
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    NewTab,
    Rename,
    MoveToWorkspace,
    /// Silence this tab's notifications, or let them through again.
    Mute,
    Unmute,
    /// The tab is silent because its workspace is muted: lift that.
    UnmuteWorkspace,
    /// The tab is silent because its device is muted: lift that.
    UnmuteDevice,
    Close,
}

/// Herdr's own tab menu, in its order, before this machine's offers filter it.
pub const ACTIONS: [Action; 4] = [
    Action::NewTab,
    Action::Rename,
    Action::MoveToWorkspace,
    Action::Close,
];
