//! Each window's quick lookup of the shared Core terminal observation.
//! A successful run is shown at once while its subscribed event catches up.

use sheprd_backend::{TerminalId, TerminalInfo};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct Infos {
    fresh: HashSet<TerminalId>,
    known: HashMap<TerminalId, TerminalInfo>,
    revision: u64,
}

impl Infos {
    pub(super) fn get(&self, id: &TerminalId) -> Option<&TerminalInfo> {
        self.known.get(id)
    }

    pub(super) fn insert(&mut self, info: TerminalInfo) -> TerminalInfo {
        self.fresh.insert(info.id.clone());
        self.known.insert(info.id.clone(), info.clone());
        info
    }

    /// Copy a new shared revision. A run result survives one snapshot that
    /// began before Core reported the created terminal.
    pub(super) fn sync(&mut self, revision: u64, terminals: &[TerminalInfo], fresh: bool) -> bool {
        if revision == self.revision {
            return false;
        }
        self.revision = revision;
        let mut known: HashMap<TerminalId, TerminalInfo> = terminals
            .iter()
            .cloned()
            .map(|info| (info.id.clone(), info))
            .collect();
        for id in self.fresh.drain() {
            if fresh
                && let Some(info) = self.known.get(&id)
                && !known.contains_key(&id)
            {
                known.insert(id, info.clone());
            }
        }
        let changed = known != self.known;
        self.known = known;
        changed
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
