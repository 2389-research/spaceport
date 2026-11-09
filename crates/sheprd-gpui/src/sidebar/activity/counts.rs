//! The bar's Overview count bubble: what needs the user on the last tick.

use super::ActivityPanel;
use crate::SheprdWindow;
use chrono::{DateTime, Utc};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Counts {
    needs: Count,
}

/// How many items a bubble counts, and when the newest of them changed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Count {
    items: usize,
    latest: Option<DateTime<Utc>>,
}

impl Counts {
    fn count(self, panel: ActivityPanel) -> Option<Count> {
        match panel {
            ActivityPanel::Overview => Some(self.needs),
            ActivityPanel::Workspaces | ActivityPanel::Usage => None,
            ActivityPanel::Automations => None,
        }
    }

    /// The bar's count for `panel`, if it shows one.
    pub(super) fn of(self, panel: ActivityPanel) -> Option<usize> {
        self.count(panel).map(|count| count.items)
    }

    /// When the newest item `panel` counts changed, when known.
    pub(super) fn latest(self, panel: ActivityPanel) -> Option<DateTime<Utc>> {
        self.count(panel)?.latest
    }
}

impl SheprdWindow {
    pub(super) fn activity_counts(&self) -> Counts {
        Counts {
            needs: Count {
                items: self.overview_needing(),
                latest: self.overview_needing_since().map(DateTime::from),
            },
            ..Counts::default()
        }
    }
}
