//! Desktop-local tags on devices, such as `personal` and `work`.
//! Saved endpoint IDs keep existing grouping until Core IDs can be mapped exactly.

mod error;
mod rules;

pub use error::Error;
pub(crate) use rules::{normalized, same};

use gpui::{App, BorrowAppContext, Global};
use rules::ComputerTags;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, PoisonError},
    time::{SystemTime, UNIX_EPOCH},
};

const FILE: &str = "device-tags.json";
/// Offered while no device has a tag yet.
pub(crate) const STARTERS: [&str; 2] = ["personal", "work"];

/// This app's copy of every device's tags, by endpoint ID.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct Tags {
    records: BTreeMap<String, ComputerTags>,
}

impl Tags {
    pub(crate) fn of(&self, id: &str) -> &[String] {
        self.records
            .get(id)
            .map_or(&[], |record| record.tags.as_slice())
    }

    /// Whether any device has a tag: until one does, the sidebar is as it
    /// always was.
    pub(crate) fn any(&self) -> bool {
        self.records.values().any(|record| !record.tags.is_empty())
    }

    /// Every tag in use, alphabetically, each once.
    pub(crate) fn all(&self) -> Vec<&str> {
        let mut tags: Vec<&str> = Vec::new();
        for tag in self.records.values().flat_map(|record| &record.tags) {
            if !tags.iter().any(|known| same(known, tag)) {
                tags.push(tag);
            }
        }
        tags.sort_by_cached_key(|tag| tag.to_lowercase());
        tags
    }

    /// Whether the device carries `tag`.
    pub(crate) fn has(&self, id: &str, tag: &str) -> bool {
        self.of(id).iter().any(|own| same(own, tag))
    }

    /// The section the device lists under: `filter` while it has that tag,
    /// else its first tag, so a device is listed once. `None` is Untagged.
    pub(crate) fn section<'a>(&'a self, id: &str, filter: Option<&'a str>) -> Option<&'a str> {
        filter
            .filter(|tag| self.has(id, tag))
            .or_else(|| self.of(id).first().map(String::as_str))
    }

    /// Where the device's section sorts: alphabetically, Untagged last.
    pub(crate) fn rank(&self, id: &str, filter: Option<&str>, all: &[&str]) -> usize {
        self.section(id, filter)
            .and_then(|section| all.iter().position(|tag| same(tag, section)))
            .unwrap_or(all.len())
    }

    /// The device's tags once moved to `tag`, which becomes its section; the
    /// section it left is dropped and any others stay. `None` clears them.
    pub(crate) fn moved_to(&self, id: &str, tag: Option<&str>) -> Vec<String> {
        let Some(tag) = tag else {
            return Vec::new();
        };
        let tag = self.known_spelling(tag);
        let others = self.of(id).iter().skip(1).filter(|own| !same(own, &tag));
        std::iter::once(tag.clone())
            .chain(others.cloned())
            .collect()
    }

    /// The device's tags with `tag` added or taken away.
    pub(crate) fn toggled(&self, id: &str, tag: &str) -> Vec<String> {
        if self.has(id, tag) {
            return self
                .of(id)
                .iter()
                .filter(|own| !same(own, tag))
                .cloned()
                .collect();
        }
        let mut tags = self.of(id).to_vec();
        tags.push(self.known_spelling(tag));
        tags
    }

    /// `tag` as a device already spells it, as two spellings are one tag.
    fn known_spelling(&self, tag: &str) -> String {
        self.all()
            .into_iter()
            .find(|known| same(known, tag))
            .unwrap_or(tag)
            .to_owned()
    }

    /// `records` as edited long ago, for tests of what draws them.
    #[cfg(test)]
    pub(crate) fn tagged(records: &[(&str, &[&str])]) -> Self {
        let mut tags = Self::default();
        for (id, own) in records {
            tags.edit(id, own.iter().map(|tag| (*tag).to_owned()).collect(), 10);
        }
        tags
    }

    /// Records an edit made here at `now_ms`. Returns whether it changed
    /// anything.
    fn edit(&mut self, id: &str, tags: Vec<String>, now_ms: u64) -> bool {
        let edited = ComputerTags::new(tags, now_ms);
        if self.of(id) == edited.tags {
            return false;
        }
        self.records.insert(id.to_owned(), edited);
        true
    }

    /// This saved copy with `early`'s edits, made before it was read, over
    /// whatever they are newer than.
    fn with_edits(mut self, early: &Self) -> Self {
        for (id, edit) in &early.records {
            if self
                .records
                .get(id)
                .is_none_or(|saved| saved.updated_ms <= edit.updated_ms)
            {
                self.records.insert(id.clone(), edit.clone());
            }
        }
        self
    }
}

/// This app's local tags.
#[derive(Default)]
pub(crate) struct DeviceTags {
    tags: Arc<Tags>,
    file: Option<PathBuf>,
    /// Whether the saved copy was read: saving before then could overwrite
    /// tags left by another window from an earlier app run.
    loaded: bool,
    /// The newest copy to write, shared with the writer so a slow write
    /// never puts an older copy back. Only ever held for a moment, as the UI
    /// thread takes it.
    saving: Arc<Mutex<Option<Arc<Tags>>>>,
    /// Held by the writer across a write, so they land one at a time.
    writing: Arc<Mutex<()>>,
}

impl Global for DeviceTags {}

impl DeviceTags {
    /// Reads the saved copy off the UI thread.
    pub(crate) fn install(cx: &mut App) {
        let file = crate::preferences::state_dir().map(|dir| dir.join(FILE));
        cx.set_global(Self {
            file: file.clone(),
            ..Self::default()
        });
        let read = cx
            .background_executor()
            .spawn(async move { file.as_deref().map(read).unwrap_or_default() });
        cx.spawn(async move |cx| {
            let tags = read.await;
            cx.update(|cx| Self::finish_loading(cx, tags));
        })
        .detach();
    }

    /// Takes the saved copy, keeping any edit made while it was being read.
    fn finish_loading(cx: &mut App, saved: Tags) {
        let edited = cx.update_global::<Self, _>(|this, _| {
            let edited = !this.tags.records.is_empty();
            this.tags = Arc::new(saved.with_edits(&this.tags));
            this.loaded = true;
            edited
        });
        if edited {
            Self::save(cx);
        }
    }

    /// The copy to draw from; empty before [`Self::install`], as in tests.
    pub(crate) fn current(cx: &App) -> Arc<Tags> {
        cx.try_global::<Self>()
            .map(|this| this.tags.clone())
            .unwrap_or_default()
    }

    /// Replaces a device's local tags and saves them.
    pub(crate) fn set(cx: &mut App, id: &str, tags: Vec<String>) {
        if !cx.has_global::<Self>() {
            return;
        }
        let changed = cx.update_global::<Self, _>(|this, _| {
            let mut copy = (*this.tags).clone();
            if !copy.edit(id, tags, now_ms()) {
                return false;
            }
            this.tags = Arc::new(copy);
            true
        });
        if changed {
            Self::save(cx);
        }
    }

    fn save(cx: &mut App) {
        let this = cx.global::<Self>();
        let Some(file) = this.file.clone().filter(|_| this.loaded) else {
            return;
        };
        let (saving, writing) = (this.saving.clone(), this.writing.clone());
        *saving.lock().unwrap_or_else(PoisonError::into_inner) = Some(this.tags.clone());
        cx.background_executor()
            .spawn(async move {
                // The newest copy is taken with the write's turn in hand, so
                // the last write holds it.
                let _writing = writing.lock().unwrap_or_else(PoisonError::into_inner);
                let newest = saving.lock().unwrap_or_else(PoisonError::into_inner).take();
                if let Some(tags) = newest
                    && let Err(error) = write(&file, &tags)
                {
                    tracing::warn!(%error, "Could not save device tags");
                }
            })
            .detach();
    }
}

/// The copy saved in `file`; none when missing or damaged. Blocking.
fn read(file: &Path) -> Tags {
    std::fs::read(file)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Tags>(&bytes).ok())
        .map(|saved| Tags {
            records: saved
                .records
                .into_iter()
                .map(|(id, record)| (id, ComputerTags::new(record.tags, record.updated_ms)))
                .collect(),
        })
        .unwrap_or_default()
}

/// Replaces `file` whole, so a crash leaves one copy or the other. Blocking.
fn write(file: &Path, tags: &Tags) -> Result<(), Error> {
    let failed = |source| Error::Save {
        path: file.to_owned(),
        source,
    };
    let bytes = serde_json::to_vec_pretty(tags).map_err(|error| failed(error.into()))?;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).map_err(failed)?;
    }
    let partial = file.with_extension("json.partial");
    std::fs::write(&partial, bytes).map_err(failed)?;
    std::fs::rename(&partial, file).map_err(failed)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

/// The tags in `tags` that are not `used`, for a picker to offer.
pub(crate) fn offered(tags: &Tags) -> Vec<String> {
    let all = tags.all();
    let known: BTreeSet<&str> = if all.is_empty() {
        STARTERS.into_iter().collect()
    } else {
        all.into_iter().collect()
    };
    let mut known: Vec<String> = known.into_iter().map(str::to_owned).collect();
    known.sort_by_cached_key(|tag| tag.to_lowercase());
    known
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
