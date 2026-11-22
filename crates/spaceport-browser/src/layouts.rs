// ABOUTME: Restores and saves each workspace's editor group layout.
// ABOUTME: Reserves tab IDs from every saved page reference across restarts.
//! Saved editor group layouts: how each workspace's view was split, which
//! tab each group showed, and the group in use, kept across restarts. The
//! app keeps one per workspace of each daemon; a window restores it the
//! first time it shows that workspace, and saves it whenever it changes.
use super::{
    Scope,
    groups::SavedLayout,
    store::{SavedEntry, identities},
};
use crate::{Error, Result};
use gpui::{App, Global};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

const MAX_LAYOUTS: usize = 256;
const MAX_FILE_BYTES: u64 = 1024 * 1024;

/// The workspace a layout belongs to.
pub type Key = (Scope, String);

#[derive(Serialize, Deserialize)]
struct Entry {
    scope: Scope,
    workspace: String,
    layout: SavedLayout,
}

#[derive(Serialize, Deserialize)]
struct Saved {
    layouts: Vec<Entry>,
    #[serde(default)]
    next_tab_id: u64,
}

#[derive(Default, Deserialize)]
struct Entries {
    layouts: Vec<SavedEntry<Entry>>,
}

impl Entries {
    // Take the floor before validation or duplicate workspace keys discard entries.
    fn next_tab_id(&self) -> u64 {
        let ids = self
            .layouts
            .iter()
            .filter_map(|entry| match entry {
                SavedEntry::Parsed(entry) => Some(&entry.layout),
                SavedEntry::Rejected(_) => None,
            })
            .flat_map(SavedLayout::pages)
            .map(u64::from);
        identities::reserve(0, ids)
    }
}

impl Entry {
    fn valid(&self) -> bool {
        self.scope.valid_workspace(&self.workspace)
            && self.layout.valid()
            && (!matches!(self.scope, Scope::Core(_)) || self.layout.pages_only())
    }

    fn into_pair(self) -> (Key, SavedLayout) {
        ((self.scope, self.workspace), self.layout)
    }
}

#[derive(Default)]
pub struct Layouts {
    layouts: HashMap<Key, SavedLayout>,
    next_tab_id: u64,
    writer: Option<spaceport_state_file::Writer<Saved>>,
    quitting: bool,
}

impl Global for Layouts {}

impl Saved {
    fn parse(bytes: &[u8]) -> Result<Self> {
        let saved: Self = serde_json::from_slice(bytes)?;
        if saved.layouts.len() > MAX_LAYOUTS || !saved.layouts.iter().all(Entry::valid) {
            return Err(Error::InvalidGroupLayouts);
        }
        Ok(saved)
    }

    fn into_layouts(self) -> HashMap<Key, SavedLayout> {
        self.layouts.into_iter().map(Entry::into_pair).collect()
    }
}

#[cfg(test)]
fn parse(bytes: &[u8]) -> Result<HashMap<Key, SavedLayout>> {
    Ok(Saved::parse(bytes)?.into_layouts())
}

/// The tab store checks layout references before handing out its first ID.
pub(super) fn reserved_next(path: &Path) -> Result<u64> {
    let bytes = spaceport_state_file::read(path, MAX_FILE_BYTES)?;
    let Some(bytes) = bytes else { return Ok(0) };
    let next = identities::layouts(&bytes)?;
    let entries: Entries = identities::rows(&bytes)?;
    Ok(next.max(entries.next_tab_id()))
}

impl Layouts {
    /// Restores layouts before GPUI starts. A missing file starts workspaces unsplit.
    pub fn load(path: Option<PathBuf>) -> Self {
        let mut layouts = Self::default();
        let Some(path) = path else { return layouts };
        if let Err(error) = layouts.restore(&path) {
            tracing::warn!(%error, "Cannot restore editor groups; saving is blocked");
            return layouts;
        }
        match spaceport_state_file::Writer::start("editor-groups", path) {
            Ok(writer) => layouts.writer = Some(writer),
            Err(error) => tracing::warn!(%error, "Cannot start editor group worker"),
        }
        layouts
    }

    fn restore(&mut self, path: &Path) -> Result<()> {
        let Some(bytes) = spaceport_state_file::read(path, MAX_FILE_BYTES)? else {
            return Ok(());
        };
        self.layouts = match Saved::parse(&bytes) {
            Ok(saved) => {
                let ids = saved
                    .layouts
                    .iter()
                    .flat_map(|entry| entry.layout.pages())
                    .map(u64::from);
                self.next_tab_id = identities::reserve(saved.next_tab_id, ids);
                saved.into_layouts()
            }
            Err(error) => {
                tracing::warn!(%error, "Cannot fully restore editor groups");
                spaceport_state_file::preserve_rejected(path, &bytes)?;
                let file: Entries = identities::rows(&bytes)?;
                self.next_tab_id = file.next_tab_id();
                file.layouts
                    .into_iter()
                    .take(MAX_LAYOUTS)
                    .filter_map(SavedEntry::parsed)
                    .filter(Entry::valid)
                    .map(Entry::into_pair)
                    .collect()
            }
        };
        self.next_tab_id = self.next_tab_id.max(identities::layouts(&bytes)?);
        Ok(())
    }

    pub fn install(self, cx: &mut App) {
        cx.set_global(self);
        cx.on_app_quit(|cx| {
            let layouts = cx.global_mut::<Self>();
            layouts.quitting = true;
            let writer = layouts.writer.take();
            cx.background_executor().spawn(async move {
                if let Some(writer) = writer {
                    writer.finish();
                }
            })
        })
        .detach();
    }

    /// Every saved layout, for a new window to restore from.
    pub fn snapshot(cx: &App) -> HashMap<Key, SavedLayout> {
        cx.try_global::<Self>()
            .map(|layouts| layouts.layouts.clone())
            .unwrap_or_default()
    }

    /// Records `layout` for `key`; `None` forgets it. Writes only a change,
    /// and reads before claiming the global so an unchanged layout wakes
    /// nothing.
    pub fn record(cx: &mut App, key: &Key, layout: Option<SavedLayout>) {
        let unchanged = cx.try_global::<Self>().map_or(layout.is_none(), |layouts| {
            layouts.layouts.get(key) == layout.as_ref()
        });
        if unchanged {
            return;
        }
        if !cx.has_global::<Self>() {
            cx.set_global(Self::default());
        }
        let layouts = cx.global_mut::<Self>();
        match layout {
            Some(layout)
                if layouts.layouts.len() < MAX_LAYOUTS || layouts.layouts.contains_key(key) =>
            {
                layouts.next_tab_id =
                    identities::reserve(layouts.next_tab_id, layout.pages().map(u64::from));
                layouts.layouts.insert(key.clone(), layout);
            }
            Some(_) => return,
            None => {
                layouts.layouts.remove(key);
            }
        }
        layouts.save();
    }

    /// Forgets the layouts of workspaces the daemon closed.
    pub fn forget_workspaces(cx: &mut App, scope: &Scope, closed: &[String]) {
        let Some(layouts) = cx.try_global::<Self>() else {
            return;
        };
        if !layouts
            .layouts
            .keys()
            .any(|(saved, workspace)| saved == scope && closed.contains(workspace))
        {
            return;
        }
        let layouts = cx.global_mut::<Self>();
        layouts
            .layouts
            .retain(|(saved, workspace), _| saved != scope || !closed.contains(workspace));
        layouts.save();
    }

    fn save(&self) {
        if self.quitting {
            return;
        }
        if let Some(writer) = &self.writer {
            writer.save(Saved {
                next_tab_id: self.next_tab_id,
                layouts: self
                    .layouts
                    .iter()
                    .map(|((scope, workspace), layout)| Entry {
                        scope: scope.clone(),
                        workspace: workspace.clone(),
                        layout: layout.clone(),
                    })
                    .collect(),
            });
        }
    }
}

#[cfg(test)]
mod tests;
