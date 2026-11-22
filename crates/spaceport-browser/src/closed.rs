//! Recently closed tabs of each workspace, both browser pages and Herdr
//! terminal tabs, in the order they were closed, so Reopen Closed Tab
//! (`Cmd-Shift-T` / `Ctrl-Alt-Shift-T`) brings back the most recent one.
//!
//! Browser tabs keep their address, title, origin pane, and kind in the
//! [`Store`]; terminal tabs keep their workspace, custom label, and working
//! directory so `tab.create` opens a fresh shell where the closed one stood.
use super::{Location, Scope, Store, Tab, TabId};
use serde_json::{Value, json};
use spaceport_client::protocol::{ClientShellPane, ClientShellSnapshot, ClientShellTab};

const MAX_CLOSED_TABS: usize = 32;
const MAX_CLOSED_WORKSPACES: usize = 64;

/// What a closed Herdr terminal tab needs to open a replacement shell with
/// `tab.create`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosedHerdrTab {
    pub tab_id: String,
    pub workspace_id: String,
    pub label: Option<String>,
    pub cwd: Option<String>,
}

impl ClosedHerdrTab {
    pub fn capture(snapshot: &ClientShellSnapshot, tab_id: &str) -> Option<Self> {
        let tab = snapshot.tabs.iter().find(|tab| tab.tab_id == tab_id)?;
        Some(Self::from_tab(snapshot, tab))
    }

    pub fn from_tab(snapshot: &ClientShellSnapshot, tab: &ClientShellTab) -> Self {
        let label = tab
            .custom_label
            .then(|| tab.label.trim().to_owned())
            .filter(|label| !label.is_empty());
        let pane_cwd = |pane: &ClientShellPane| {
            pane.foreground_cwd
                .as_deref()
                .filter(|path| !path.is_empty())
                .or_else(|| pane.cwd.as_deref().filter(|path| !path.is_empty()))
                .map(str::to_owned)
        };
        let cwd = snapshot
            .panes
            .iter()
            .find(|pane| {
                pane.tab_id == tab.tab_id && pane.workspace_id == tab.workspace_id && pane.focused
            })
            .and_then(pane_cwd)
            .or_else(|| {
                snapshot
                    .panes
                    .iter()
                    .filter(|pane| {
                        pane.tab_id == tab.tab_id && pane.workspace_id == tab.workspace_id
                    })
                    .find_map(pane_cwd)
            });
        Self {
            tab_id: tab.tab_id.clone(),
            workspace_id: tab.workspace_id.clone(),
            label,
            cwd,
        }
    }

    pub fn create_params(&self) -> Value {
        let mut params = json!({
            "workspace_id": self.workspace_id,
            "focus": true,
        });
        if let Some(cwd) = &self.cwd {
            params["cwd"] = Value::String(cwd.clone());
        }
        if let Some(label) = &self.label {
            params["label"] = Value::String(label.clone());
        }
        params
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClosedTab {
    Page(Tab),
    Herdr(ClosedHerdrTab),
}

const MAX_RECORDED_HERDR: usize = 256;

#[derive(Default)]
pub(super) struct ClosedTabs {
    workspaces: Vec<((Scope, String), Vec<ClosedTab>)>,
    recorded_herdr: Vec<((Scope, String), Vec<String>)>,
}

impl ClosedTabs {
    fn workspace_mut(&mut self, key: (Scope, String)) -> &mut Vec<ClosedTab> {
        if let Some(index) = self
            .workspaces
            .iter()
            .position(|(existing, _)| *existing == key)
        {
            let entry = self.workspaces.remove(index);
            self.workspaces.push(entry);
        } else {
            if self.workspaces.len() >= MAX_CLOSED_WORKSPACES {
                self.workspaces.remove(0);
            }
            self.workspaces.push((key, Vec::new()));
        }
        let last = self.workspaces.len() - 1;
        &mut self.workspaces[last].1
    }

    fn recorded_herdr_mut(&mut self, key: (Scope, String)) -> &mut Vec<String> {
        if let Some(index) = self
            .recorded_herdr
            .iter()
            .position(|(existing, _)| *existing == key)
        {
            let entry = self.recorded_herdr.remove(index);
            self.recorded_herdr.push(entry);
        } else {
            if self.recorded_herdr.len() >= MAX_CLOSED_WORKSPACES {
                self.recorded_herdr.remove(0);
            }
            self.recorded_herdr.push((key, Vec::new()));
        }
        let last = self.recorded_herdr.len() - 1;
        &mut self.recorded_herdr[last].1
    }

    pub(super) fn push_page(&mut self, tab: Tab) {
        let key = (tab.scope.clone(), tab.workspace_id.clone());
        let entries = self.workspace_mut(key);
        entries.retain(|entry| match entry {
            ClosedTab::Page(earlier) => earlier.id != tab.id,
            ClosedTab::Herdr(_) => true,
        });
        if entries.len() >= MAX_CLOSED_TABS {
            entries.remove(0);
        }
        entries.push(ClosedTab::Page(tab));
    }

    pub(super) fn push_herdr(&mut self, scope: Scope, tab: ClosedHerdrTab) {
        let key = (scope, tab.workspace_id.clone());
        let recorded = self.recorded_herdr_mut(key.clone());
        if recorded.contains(&tab.tab_id) {
            return;
        }
        if recorded.len() >= MAX_RECORDED_HERDR {
            recorded.remove(0);
        }
        recorded.push(tab.tab_id.clone());
        self.push_herdr_entry(key, tab);
    }

    /// Puts back a terminal tab whose reopen could not be sent. It was
    /// recorded when it closed, so the cross-window guard in [`Self::push_herdr`]
    /// would otherwise drop it.
    pub(super) fn restore_herdr(&mut self, scope: Scope, tab: ClosedHerdrTab) {
        let key = (scope, tab.workspace_id.clone());
        let recorded = self.recorded_herdr_mut(key.clone());
        if !recorded.contains(&tab.tab_id) {
            if recorded.len() >= MAX_RECORDED_HERDR {
                recorded.remove(0);
            }
            recorded.push(tab.tab_id.clone());
        }
        self.push_herdr_entry(key, tab);
    }

    fn push_herdr_entry(&mut self, key: (Scope, String), tab: ClosedHerdrTab) {
        let entries = self.workspace_mut(key);
        if entries
            .iter()
            .any(|entry| matches!(entry, ClosedTab::Herdr(earlier) if earlier.tab_id == tab.tab_id))
        {
            return;
        }
        if entries.len() >= MAX_CLOSED_TABS {
            entries.remove(0);
        }
        entries.push(ClosedTab::Herdr(tab));
    }

    pub(super) fn peek(&self, scope: &Scope, workspace_id: &str) -> Option<&ClosedTab> {
        self.workspaces
            .iter()
            .find(|((saved_scope, saved_workspace), _)| {
                saved_scope == scope && saved_workspace == workspace_id
            })
            .and_then(|(_, entries)| entries.last())
    }

    pub(super) fn pop(&mut self, scope: &Scope, workspace_id: &str) -> Option<ClosedTab> {
        let index = self
            .workspaces
            .iter()
            .position(|((saved_scope, saved_workspace), _)| {
                saved_scope == scope && saved_workspace == workspace_id
            })?;
        let popped = self.workspaces[index].1.pop();
        if self.workspaces[index].1.is_empty() {
            self.workspaces.remove(index);
        }
        popped
    }

    pub(super) fn forget_workspaces(&mut self, scope: &Scope, closed: &[String]) {
        self.workspaces
            .retain(|((saved, workspace), _)| saved != scope || !closed.contains(workspace));
        self.recorded_herdr
            .retain(|((saved, workspace), _)| saved != scope || !closed.contains(workspace));
    }

    pub(super) fn has_workspaces(&self, scope: &Scope, closed: &[String]) -> bool {
        self.workspaces
            .iter()
            .any(|((saved, workspace), _)| saved == scope && closed.contains(workspace))
    }
}

impl Store {
    pub fn record_closed_page(&mut self, tab: Tab) {
        self.closed.push_page(tab);
    }

    pub fn record_closed_herdr_tab(&mut self, scope: Scope, tab: ClosedHerdrTab) {
        self.closed.push_herdr(scope, tab);
    }

    pub fn restore_closed_herdr_tab(&mut self, scope: Scope, tab: ClosedHerdrTab) {
        self.closed.restore_herdr(scope, tab);
    }

    pub fn peek_closed(&self, scope: &Scope, workspace_id: &str) -> Option<&ClosedTab> {
        self.closed.peek(scope, workspace_id)
    }

    pub fn pop_closed(&mut self, scope: &Scope, workspace_id: &str) -> Option<ClosedTab> {
        self.closed.pop(scope, workspace_id)
    }

    /// Reopens a closed browser tab in its workspace, reusing a singleton tab
    /// when the workspace already has one open.
    pub fn reopen_page(&mut self, tab: Tab) -> Option<TabId> {
        if let Some(existing) = match &tab.location {
            Some(Location::Devices) => self
                .in_workspace(&tab.scope, &tab.workspace_id)
                .find(|open| matches!(open.location, Some(Location::Devices)))
                .map(|open| open.id),
            _ => None,
        } {
            return Some(existing);
        }
        self.restore_tab(tab)
    }
}

#[cfg(test)]
mod tests;
