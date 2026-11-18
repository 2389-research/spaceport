// ABOUTME: Captures complete Close Others consent without closing any view or terminal.
// ABOUTME: Rechecks exact ownership against current and pending metadata before each effect.
use super::{CloseConfirmation, Target};
use crate::{
    Error, Result, SheprdWindow,
    browser::{Pick, Scope, Store, Tab},
};
use gpui::App;
use sheprd_client::protocol::ClientShellSnapshot;
use std::collections::{BTreeSet, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Batch {
    pub(super) owner: (Scope, String),
    connection: (u64, Option<u64>),
    pub(super) items: Vec<Item>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Item {
    Herdr {
        close: CloseConfirmation,
        panes: BTreeSet<String>,
    },
    View(Tab),
}

impl Item {
    pub(super) fn tab(&self) -> Option<&str> {
        match self {
            Self::Herdr { close, .. } => match &close.target {
                Target::Herdr(target) => Some(&target.tab),
                _ => None,
            },
            Self::View(_) => None,
        }
    }
}

impl Batch {
    pub(super) fn capture(view: &SheprdWindow, tabs: Vec<Pick>, cx: &App) -> Result<Self> {
        let owner = view.active_browser_key(cx).ok_or(Error::StaleCloseTarget)?;
        let mut seen = HashSet::new();
        let mut items = Vec::new();
        for pick in tabs {
            if !seen.insert(pick.clone()) {
                return Err(Error::StaleCloseTarget);
            }
            items.push(match pick {
                Pick::Page(id) => {
                    let tab = cx
                        .try_global::<Store>()
                        .and_then(|store| store.get(id))
                        .filter(|tab| (&tab.scope, &tab.workspace_id) == (&owner.0, &owner.1))
                        .ok_or(Error::StaleCloseTarget)?;
                    Item::View(tab.clone())
                }
                Pick::Herdr(id) => {
                    let snapshot = view.live.snapshot.as_deref().ok_or(Error::NotConnected)?;
                    let close = CloseConfirmation::capture_tab(snapshot, &id)
                        .ok_or(Error::StaleCloseTarget)?;
                    let Target::Herdr(target) = &close.target else {
                        return Err(Error::StaleCloseTarget);
                    };
                    if view.selected_core_group().is_some() || target.workspace != owner.1 {
                        return Err(Error::StaleCloseTarget);
                    }
                    let panes = snapshot
                        .panes
                        .iter()
                        .filter(|pane| pane.tab_id == id && pane.workspace_id == owner.1)
                        .map(|pane| pane.pane_id.clone())
                        .collect();
                    Item::Herdr { close, panes }
                }
            });
        }
        Ok(Self {
            owner,
            connection: (
                view.selection_epoch,
                view.endpoints
                    .get(view.selected_endpoint)
                    .map(|endpoint| endpoint.generation),
            ),
            items,
        })
    }

    pub(super) fn labels(&self) -> String {
        self.items
            .iter()
            .map(|item| match item {
                Item::Herdr { close, .. } => close.label.clone(),
                Item::View(tab) => tab.label(),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(super) fn pane_count(&self) -> usize {
        self.items
            .iter()
            .map(|item| match item {
                Item::Herdr { panes, .. } => panes.len(),
                Item::View(_) => 0,
            })
            .sum()
    }

    pub(super) fn view_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, Item::View(_)))
            .count()
    }

    pub(super) fn validate_owner(&self, view: &SheprdWindow, cx: &App) -> Result<()> {
        if view.active_browser_key(cx).as_ref() != Some(&self.owner)
            || self.connection
                != (
                    view.selection_epoch,
                    view.endpoints
                        .get(view.selected_endpoint)
                        .map(|endpoint| endpoint.generation),
                )
        {
            return Err(Error::StaleConnection);
        }
        let Some(boot) = self.items.iter().find_map(|item| match item {
            Item::Herdr { close, .. } => match &close.target {
                Target::Herdr(target) => Some(&target.boot),
                _ => None,
            },
            Item::View(_) => None,
        }) else {
            return Ok(());
        };
        if !view.live.status.is_connected() || view.selected_core_group().is_some() {
            return Err(Error::NotConnected);
        }
        if &view
            .live
            .latest_snapshot()
            .ok_or(Error::NotConnected)?
            .boot_id
            != boot
        {
            return Err(Error::StaleCloseTarget);
        }
        let inbox = view.endpoints[view.selected_endpoint]
            .connection
            .inbox
            .try_lock()
            .map_err(|_| Error::ConnectionNotReady)?;
        if !inbox.status.is_connected() {
            return Err(Error::NotConnected);
        }
        if &inbox.latest_snapshot().ok_or(Error::NotConnected)?.boot_id != boot {
            return Err(Error::StaleCloseTarget);
        }
        Ok(())
    }

    pub(super) fn validate(&self, view: &SheprdWindow, items: &[Item], cx: &App) -> Result<()> {
        self.validate_owner(view, cx)?;
        for item in items {
            if let Item::View(tab) = item
                && cx.try_global::<Store>().and_then(|store| store.get(tab.id)) != Some(tab)
            {
                return Err(Error::StaleCloseTarget);
            }
        }
        if items.iter().all(|item| matches!(item, Item::View(_))) {
            return Ok(());
        }
        if !view.live.status.is_connected() || view.selected_core_group().is_some() {
            return Err(Error::NotConnected);
        }
        Self::validate_snapshot(
            items,
            view.live.latest_snapshot().ok_or(Error::NotConnected)?,
        )?;
        let inbox = view.endpoints[view.selected_endpoint]
            .connection
            .inbox
            .try_lock()
            .map_err(|_| Error::ConnectionNotReady)?;
        if !inbox.status.is_connected() {
            return Err(Error::NotConnected);
        }
        Self::validate_snapshot(items, inbox.latest_snapshot().ok_or(Error::NotConnected)?)
    }

    fn validate_snapshot(items: &[Item], snapshot: &ClientShellSnapshot) -> Result<()> {
        for item in items {
            let Item::Herdr { close, panes } = item else {
                continue;
            };
            close.request(snapshot)?;
            let Target::Herdr(target) = &close.target else {
                return Err(Error::StaleCloseTarget);
            };
            let current: BTreeSet<_> = snapshot
                .panes
                .iter()
                .filter(|pane| pane.tab_id == target.tab && pane.workspace_id == target.workspace)
                .map(|pane| pane.pane_id.clone())
                .collect();
            if &current != panes
                || !snapshot
                    .tabs
                    .iter()
                    .any(|tab| tab.tab_id == target.tab && tab.label == close.label)
            {
                return Err(Error::StaleCloseTarget);
            }
        }
        Ok(())
    }
}
