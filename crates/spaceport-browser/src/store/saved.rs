// ABOUTME: Reads the saved tab file that any earlier build may have written.
// ABOUTME: Checks the file's bounds, then yields only the tabs that restore.
//! The schema of `browser-tabs.json`. A tab of a kind an earlier build
//! showed reads as `Retired`: it is dropped, but its ID stays reserved. A
//! kind no build wrote refuses the file, so tabs a newer build saved are
//! never dropped and written over.
use super::{Location, MAX_TABS, MAX_TITLE_CHARS, Saved, SavedEntry, Scope, Tab, TabId};
use crate::{Error, Result};
use serde::Deserialize;
use spaceport_panes::Panes;

#[derive(Deserialize)]
struct SavedFile {
    tabs: Vec<SavedTab>,
    #[serde(default)]
    next: u64,
}

#[derive(Deserialize)]
struct SavedTab {
    id: TabId,
    scope: Scope,
    workspace_id: String,
    location: Option<SavedLocation>,
    title: String,
    #[serde(default)]
    origin: Option<String>,
    /// Tabs saved before panels existed are all `Group` tabs.
    #[serde(default)]
    place: SavedPlace,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SavedLocation {
    Devices,
    /// Tabs saved before splits named one `terminal`.
    CoreTerminal {
        #[serde(alias = "terminal")]
        panes: Panes,
        #[serde(default)]
        observe: bool,
    },
    /// A page or editor tab of an earlier build; its payload is ignored.
    #[serde(
        rename = "web",
        alias = "local",
        alias = "review",
        alias = "orchestrator",
        alias = "code"
    )]
    Retired {},
}

/// Where a tab was shown. Only group tabs restore; the others were editor
/// panels of an earlier build.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum SavedPlace {
    #[default]
    Group,
    Retired,
}

impl<'de> Deserialize<'de> for SavedPlace {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        match name.as_str() {
            "group" => Ok(Self::Group),
            "code" | "code_group" | "px0" => Ok(Self::Retired),
            _ => Err(serde::de::Error::unknown_variant(
                &name,
                &["group", "code", "code_group", "px0"],
            )),
        }
    }
}

impl SavedTab {
    fn valid(&self) -> bool {
        self.scope.valid_workspace(&self.workspace_id)
            && self.title.chars().count() <= MAX_TITLE_CHARS
            && self
                .origin
                .as_ref()
                .is_none_or(|pane| !pane.is_empty() && pane.len() <= 256)
    }

    /// The live tab, or `None` for a kind the app no longer shows, or a
    /// blank tab outside a Core group.
    fn restore(self) -> Option<Tab> {
        if self.place == SavedPlace::Retired {
            return None;
        }
        let location = match (self.location, &self.scope) {
            (Some(SavedLocation::Devices), _) => Some(Location::Devices),
            (Some(SavedLocation::CoreTerminal { panes, observe }), _) => {
                Some(Location::CoreTerminal { panes, observe })
            }
            // A Core tab still waiting for its terminal reference.
            (None, Scope::Core(_)) => None,
            (None, Scope::Legacy(_)) | (Some(SavedLocation::Retired {}), _) => return None,
        };
        Some(Tab {
            id: self.id,
            scope: self.scope,
            workspace_id: self.workspace_id,
            location,
            title: self.title,
            origin: self.origin,
        })
    }
}

/// The tabs of a saved file that restore, and the next free ID.
pub(super) fn parse(bytes: &[u8]) -> Result<Saved> {
    let file: SavedFile = serde_json::from_slice(bytes)?;
    if file.tabs.len() > MAX_TABS || !file.tabs.iter().all(SavedTab::valid) {
        return Err(Error::InvalidBrowserTabs);
    }
    // Saved group picks may still name a discarded tab. Never assign its ID
    // to a new tab, or that old pick could show unrelated content.
    let next = file
        .tabs
        .iter()
        .map(|tab| tab.id.0.saturating_add(1))
        .max()
        .unwrap_or(0)
        .max(file.next);
    Ok(Saved {
        tabs: file
            .tabs
            .into_iter()
            .filter_map(SavedTab::restore)
            .collect(),
        next,
    })
}

/// Restores independent entries that still pass this build's schema checks.
pub(super) fn recover(bytes: &[u8]) -> Saved {
    #[derive(Default, Deserialize)]
    struct Entries {
        tabs: Vec<SavedEntry<SavedTab>>,
    }
    let Ok(file) = super::identities::rows::<Entries>(bytes) else {
        return Saved::default();
    };
    let mut saved = Saved::default();
    for (index, entry) in file.tabs.into_iter().enumerate() {
        if let Some(tab) = entry.parsed() {
            saved.next = saved.next.max(tab.id.0.saturating_add(1));
            if index < MAX_TABS && tab.valid() {
                saved.tabs.extend(tab.restore());
            }
        }
    }
    saved
}

#[cfg(test)]
mod tests;
