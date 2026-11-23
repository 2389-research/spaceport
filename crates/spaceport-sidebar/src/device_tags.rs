//! Desktop-local tags on devices, such as `personal` and `work`: the tags each
//! device carries, the sections they group a list into, and the file they are
//! saved in. The app keeps the copy every window reads and decides when it is
//! saved.
//! Saved endpoint IDs keep existing grouping until Core IDs can be mapped exactly.

mod error;
mod rules;

pub use error::Error;
pub use rules::{normalized, same};

use crate::Result;
use rules::ComputerTags;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// Offered while no device has a tag yet.
const STARTERS: [&str; 2] = ["personal", "work"];
/// The file the tags are saved in, under the app's state directory.
pub const FILE: &str = "device-tags.json";

/// Every device's tags, by endpoint ID.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tags {
    records: BTreeMap<String, ComputerTags>,
}

impl Tags {
    pub fn of(&self, id: &str) -> &[String] {
        self.records
            .get(id)
            .map_or(&[], |record| record.tags.as_slice())
    }

    /// Whether no device's tags have been recorded at all, even to clear
    /// them.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Whether any device has a tag: until one does, the sidebar is as it
    /// always was.
    pub fn any(&self) -> bool {
        self.records.values().any(|record| !record.tags.is_empty())
    }

    /// Every tag in use, alphabetically, each once.
    pub fn all(&self) -> Vec<&str> {
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
    pub fn has(&self, id: &str, tag: &str) -> bool {
        self.of(id).iter().any(|own| same(own, tag))
    }

    /// The section the device lists under: `filter` while it has that tag,
    /// else its first tag, so a device is listed once. `None` is Untagged.
    pub fn section<'a>(&'a self, id: &str, filter: Option<&'a str>) -> Option<&'a str> {
        filter
            .filter(|tag| self.has(id, tag))
            .or_else(|| self.of(id).first().map(String::as_str))
    }

    /// Where the device's section sorts: alphabetically, Untagged last.
    pub fn rank(&self, id: &str, filter: Option<&str>, all: &[&str]) -> usize {
        self.section(id, filter)
            .and_then(|section| all.iter().position(|tag| same(tag, section)))
            .unwrap_or(all.len())
    }

    /// The device's tags once moved to `tag`, which becomes its section; the
    /// section it left is dropped and any others stay. `None` clears them.
    pub fn moved_to(&self, id: &str, tag: Option<&str>) -> Vec<String> {
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
    pub fn toggled(&self, id: &str, tag: &str) -> Vec<String> {
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
    pub fn tagged(records: &[(&str, &[&str])]) -> Self {
        let mut tags = Self::default();
        for (id, own) in records {
            tags.edit(id, own.iter().map(|tag| (*tag).to_owned()).collect(), 10);
        }
        tags
    }

    /// Records an edit made here at `now_ms`. Returns whether it changed
    /// anything.
    pub fn edit(&mut self, id: &str, tags: Vec<String>, now_ms: u64) -> bool {
        let edited = ComputerTags::new(tags, now_ms);
        if self.of(id) == edited.tags {
            return false;
        }
        self.records.insert(id.to_owned(), edited);
        true
    }

    /// This saved copy with `early`'s edits, made before it was read, over
    /// whatever they are newer than.
    pub fn with_edits(mut self, early: &Self) -> Self {
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

/// The copy saved in `file`; none when missing or damaged. Blocking.
pub fn read(file: &Path) -> Tags {
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
pub fn write(file: &Path, tags: &Tags) -> Result<()> {
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
    std::fs::rename(&partial, file).map_err(|source| crate::Error::from(failed(source)))
}

/// The tags in `tags` that are not `used`, for a picker to offer.
pub fn offered(tags: &Tags) -> Vec<String> {
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
mod tests;
