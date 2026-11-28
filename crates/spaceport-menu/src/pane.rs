//! The pane menu's target, its rows, and the request each row becomes. A
//! target is captured from the snapshot the menu opened under, and every row
//! that sends anything names that pane explicitly.

use crate::{Error, Result};
use serde_json::{Value, json};
use spaceport_client::{Method, pane_action, protocol::ClientShellSnapshot};

pub mod render;

/// The pane one menu is about, captured from the snapshot it opened under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub boot: String,
    pub workspace: String,
    pub tab: String,
    pub pane: String,
    pub label: String,
    /// Whether the pane's tab was zoomed when the menu opened, so the zoom
    /// row says what it will do and asks Herdr for exactly that.
    pub zoomed: bool,
    /// The focused pane when it is another one in the same tab, which "Swap
    /// with focused pane" trades places with.
    pub focused: Option<String>,
    /// Herdr routes this pane's right-clicks to its application, as seen when
    /// the menu opened; the toggle asks for the other routing.
    pub right_click_passthrough: bool,
}

/// What this machine and the endpoint offer a pane's menu beyond the pane
/// itself, which the menu decides while the pane's model stays here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Offers {
    /// Whether this endpoint's host can run `herdr pane move`.
    pub movable: bool,
    /// Whether the move picker has a destination other than a new tab.
    pub destinations: bool,
    /// Whether the pane's processes are this machine's to read.
    pub processes: bool,
}

impl Target {
    pub fn capture(snapshot: &ClientShellSnapshot, id: &str) -> Option<Self> {
        let pane = snapshot.panes.iter().find(|pane| pane.pane_id == id)?;
        let zoomed = snapshot
            .tabs
            .iter()
            .any(|tab| tab.tab_id == pane.tab_id && tab.zoomed);
        let focused = snapshot.focused_pane_id.as_ref().filter(|focused| {
            **focused != pane.pane_id
                && snapshot.panes.iter().any(|p| {
                    p.pane_id == **focused
                        && p.tab_id == pane.tab_id
                        && p.workspace_id == pane.workspace_id
                })
        });
        let target = Self {
            boot: snapshot.boot_id.clone(),
            workspace: pane.workspace_id.clone(),
            tab: pane.tab_id.clone(),
            pane: pane.pane_id.clone(),
            label: pane.label.clone().unwrap_or_default(),
            zoomed,
            focused: focused.cloned(),
            right_click_passthrough: pane.right_click_passthrough,
        };
        target.validate(snapshot).ok()?;
        Some(target)
    }

    /// Whether `snapshot` still holds the workspace, tab and pane this menu
    /// captured, so an action can name them.
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
            || !snapshot.panes.iter().any(|p| {
                p.pane_id == self.pane && p.tab_id == self.tab && p.workspace_id == self.workspace
            })
        {
            return Err(Error::StalePane);
        }
        Ok(())
    }

    pub fn rename_params(&self, label: &str) -> Value {
        pane_action::Action::Rename(label).request(&self.pane).1
    }

    /// The rows this menu offers, in Herdr's order: a swap needs another pane
    /// to trade with, moves need a host the `herdr` CLI can run on, and a
    /// process list needs the pane's processes on this machine.
    pub fn actions(&self, offers: Offers) -> Vec<Action> {
        ACTIONS
            .into_iter()
            .filter(|action| match action {
                Action::Swap => self.focused.is_some(),
                Action::MoveToNewTab | Action::MoveToNewWorkspace => offers.movable,
                Action::MoveTo => offers.movable && offers.destinations,
                Action::Processes => offers.processes,
                _ => true,
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Rename,
    SplitRight,
    SplitDown,
    Swap,
    Zoom,
    MoveToNewTab,
    MoveToNewWorkspace,
    /// Opens the picker of every destination.
    MoveTo,
    EditScrollback,
    RightClick,
    Processes,
    Close,
}

impl Action {
    pub fn request(self, target: &Target) -> Option<(Method, Value)> {
        Some(match self {
            Self::SplitRight => pane_action::Action::SplitRight.request(&target.pane),
            Self::SplitDown => pane_action::Action::SplitDown.request(&target.pane),
            // The focused pane moves to this one's place and keeps focus,
            // as Herdr's directional swaps move it.
            Self::Swap => (
                Method::PaneSwap,
                json!({"source_pane_id": target.focused.as_ref()?, "target_pane_id": target.pane}),
            ),
            // An explicit mode, so a zoom another client changed meanwhile
            // leaves the pane as the row promised rather than flipping it.
            Self::Zoom => pane_action::Action::SetZoom(!target.zoomed).request(&target.pane),
            Self::RightClick => (
                Method::PaneInputSet,
                json!({
                    "pane_id": target.pane,
                    "right_click": if target.right_click_passthrough { "herdr" } else { "pane" },
                }),
            ),
            Self::Rename
            | Self::MoveToNewTab
            | Self::MoveToNewWorkspace
            | Self::MoveTo
            | Self::EditScrollback
            | Self::Processes
            | Self::Close => return None,
        })
    }
}

/// Herdr's own pane menu, in its order, before this machine's offers filter it.
pub const ACTIONS: [Action; 12] = [
    Action::Rename,
    Action::SplitRight,
    Action::SplitDown,
    Action::Swap,
    Action::Zoom,
    Action::MoveToNewTab,
    Action::MoveToNewWorkspace,
    Action::MoveTo,
    Action::EditScrollback,
    Action::RightClick,
    Action::Processes,
    Action::Close,
];
