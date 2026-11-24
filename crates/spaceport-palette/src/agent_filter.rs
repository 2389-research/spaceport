//! Go To's agent status filter, as Herdr's navigator offers with b/w/i/d/a.
//!
//! The status always comes from the daemon snapshot an entry was built from;
//! a terminal without an agent has none and only passes [`AgentFilter::All`].
//! Choosing one from a keystroke is the app's, since a keystroke is GPUI's.

use crate::{
    entry::Entry,
    search::{self, Hit},
};
use spaceport_client::protocol::AgentStatus;
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AgentFilter {
    #[default]
    All,
    Blocked,
    Working,
    Idle,
    Done,
}

impl AgentFilter {
    pub const ALL: [Self; 5] = [
        Self::All,
        Self::Blocked,
        Self::Working,
        Self::Idle,
        Self::Done,
    ];

    /// The chip's name in test selectors, the same in every language.
    pub fn name(self) -> &'static str {
        match self {
            Self::All => "all-agents",
            Self::Blocked => "blocked",
            Self::Working => "working",
            Self::Idle => "idle",
            Self::Done => "done",
        }
    }

    /// Herdr's navigator letter, typed with Alt so the search keeps every
    /// plain letter.
    pub fn key(self) -> &'static str {
        match self {
            Self::All => "a",
            Self::Blocked => "b",
            Self::Working => "w",
            Self::Idle => "i",
            Self::Done => "d",
        }
    }

    /// Whether a row whose agent reports `status` stays listed; `None` is a
    /// row without an agent.
    pub fn accepts(self, status: Option<AgentStatus>) -> bool {
        let wanted = match self {
            Self::All => return true,
            Self::Blocked => AgentStatus::Blocked,
            Self::Working => AgentStatus::Working,
            Self::Idle => AgentStatus::Idle,
            Self::Done => AgentStatus::Done,
        };
        status == Some(wanted)
    }

    /// Ranks `candidates` as [`search::rank`] does. While a status is chosen,
    /// the candidates are only matching agents, so each one's workspace and
    /// tab are added back as its headings, whether or not the query matches
    /// them, and a workspace ranks with the best of its agents.
    pub fn rank(
        self,
        entries: &[Entry],
        candidates: Vec<usize>,
        query: &search::Query,
        matcher: &mut nucleo_matcher::Matcher,
    ) -> Vec<Hit> {
        let hits = search::rank(entries, candidates, query, matcher);
        if self == Self::All {
            return hits;
        }
        headed(entries, hits)
    }
}

/// `hits` grouped under their workspace, workspaces in the order of their
/// best hit and rows within one in entry order, so every agent stays nested
/// under its headings.
pub(crate) fn headed(entries: &[Entry], hits: Vec<Hit>) -> Vec<Hit> {
    let ancestors = |index: usize| {
        std::iter::successors(entries[index].parent, |parent| entries[*parent].parent)
    };
    let root = |index: usize| ancestors(index).last().unwrap_or(index);
    let mut groups = HashMap::new();
    let mut rows = BTreeMap::new();
    for hit in hits {
        let next = groups.len();
        groups.entry(root(hit.index)).or_insert(next);
        for heading in ancestors(hit.index) {
            rows.entry(heading)
                .or_insert_with(search::Highlights::default);
        }
        rows.insert(hit.index, hit.highlights);
    }
    let mut rows: Vec<_> = rows
        .into_iter()
        .map(|(index, highlights)| {
            let group = groups.get(&root(index)).copied().unwrap_or(usize::MAX);
            (group, index, highlights)
        })
        .collect();
    rows.sort_by_key(|(group, index, _)| (*group, *index));
    rows.into_iter()
        .map(|(_, index, highlights)| Hit {
            index,
            depth: ancestors(index).count(),
            highlights,
        })
        .collect()
}
