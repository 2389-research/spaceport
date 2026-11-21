//! Saved editor group layouts: how each workspace's view was split, which
//! tab each group showed, and the group in use, kept across restarts. The
//! app keeps one per workspace of each daemon; a window restores it the
//! first time it shows that workspace, and saves it whenever it changes.
use super::{Scope, groups::SavedLayout};
use crate::{Error, Result};
use gpui::{App, Global};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

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
}

#[derive(Default)]
pub struct Layouts {
    layouts: HashMap<Key, SavedLayout>,
    writer: Option<sheprd_state_file::Writer<Saved>>,
    quitting: bool,
}

impl Global for Layouts {}

fn parse(bytes: &[u8]) -> Result<HashMap<Key, SavedLayout>> {
    let saved: Saved = serde_json::from_slice(bytes)?;
    if saved.layouts.len() > MAX_LAYOUTS
        || !saved.layouts.iter().all(|entry| {
            entry.scope.valid_workspace(&entry.workspace)
                && entry.layout.valid()
                && (!matches!(entry.scope, Scope::Core(_)) || entry.layout.pages_only())
        })
    {
        return Err(Error::InvalidGroupLayouts);
    }
    Ok(saved
        .layouts
        .into_iter()
        .map(|entry| ((entry.scope, entry.workspace), entry.layout))
        .collect())
}

impl Layouts {
    /// Called before starting GPUI, like the browser tabs, with the file the
    /// app keeps its layouts in. A missing or damaged file starts every
    /// workspace unsplit.
    pub fn load(path: Option<PathBuf>) -> Self {
        let layouts = path
            .as_deref()
            .map(|path| {
                sheprd_state_file::read(path, MAX_FILE_BYTES)
                    .map_err(Error::from)
                    .and_then(|bytes| bytes.as_deref().map_or(Ok(HashMap::new()), parse))
            })
            .transpose()
            .unwrap_or_else(|error| {
                tracing::warn!(%error, "Cannot restore editor groups");
                None
            })
            .unwrap_or_default();
        let writer =
            path.and_then(
                |path| match sheprd_state_file::Writer::start("editor-groups", path) {
                    Ok(writer) => Some(writer),
                    Err(error) => {
                        tracing::warn!(%error, "Cannot start editor group worker");
                        None
                    }
                },
            );
        Self {
            layouts,
            writer,
            quitting: false,
        }
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
