//! Groups open and close with motion: a split's new group opens from the
//! right edge while the group it came from gives up the room, and a closed
//! group folds away to the right while its neighbour takes the room back.
//! The layout holds each group's settled share of the row; this only bends
//! the shares drawn while something moves.
use super::GroupId;
use sheprd_motion::{ENTER, LEAVE, progress};
use std::time::Instant;

/// A closed group as drawn now: where it stood and the share of the row it
/// still takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fold {
    pub index: usize,
    pub share: f32,
}

/// A split's new group, opening.
#[derive(Clone, Copy, Debug)]
struct Opening {
    group: GroupId,
    /// The group it split from, which gives up the room as it opens.
    from: GroupId,
    since: Instant,
}

/// A closed group, folding away where it stood.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Folding {
    /// Its place in the row when it closed.
    index: usize,
    share: f32,
    /// The neighbour its room went to.
    into: GroupId,
    since: Instant,
}

impl Folding {
    /// The share of the row it still takes at `now`.
    fn share(&self, now: Instant) -> f32 {
        self.share * progress(self.since, now, LEAVE).map_or(0., |k| 1. - k)
    }
}

pub struct GroupMotion {
    opening: Vec<Opening>,
    folding: Vec<Folding>,
    /// Off in tests, which split and click at once and so need groups
    /// settled; the tests of the motion itself turn it on.
    enabled: bool,
    /// How far after the latest motion began tests fix it, since their
    /// frames are too slow to catch it mid-way on their own.
    #[cfg(any(test, feature = "test-support"))]
    frozen: Option<std::time::Duration>,
}

impl Default for GroupMotion {
    fn default() -> Self {
        Self {
            opening: Vec::new(),
            folding: Vec::new(),
            enabled: !cfg!(any(test, feature = "test-support")),
            #[cfg(any(test, feature = "test-support"))]
            frozen: None,
        }
    }
}

impl GroupMotion {
    /// The time motion is drawn at: `now`, unless a test fixed it.
    fn at(&self, now: Instant) -> Instant {
        #[cfg(any(test, feature = "test-support"))]
        if let Some(offset) = self.frozen {
            let latest = self
                .opening
                .iter()
                .map(|opening| opening.since)
                .chain(self.folding.iter().map(|folding| folding.since))
                .max();
            if let Some(since) = latest {
                return since + offset;
            }
        }
        now
    }

    /// `group` opened from `from` at `now`.
    pub fn open(&mut self, group: GroupId, from: GroupId, now: Instant) {
        if !self.enabled {
            return;
        }
        self.opening.push(Opening {
            group,
            from,
            since: now,
        });
    }

    /// `group`, at `index` with `share` of the row, closed at `now` and
    /// gave its room to `into`.
    pub fn fold(&mut self, group: GroupId, index: usize, share: f32, into: GroupId, now: Instant) {
        if !self.enabled {
            return;
        }
        // A group closed while still opening folds from where it had got to.
        let opened = self.opened(group, now).unwrap_or(1.);
        self.opening.retain(|opening| opening.group != group);
        self.folding.push(Folding {
            index,
            share: share * opened,
            into,
            since: now,
        });
    }

    /// How far `group` has opened at `now`, or `None` once it is open.
    pub fn opened(&self, group: GroupId, now: Instant) -> Option<f32> {
        let now = self.at(now);
        let opening = self.opening.iter().find(|opening| opening.group == group)?;
        progress(opening.since, now, ENTER)
    }

    /// The share of the row `group` takes at `now`, given its settled
    /// `share` and the settled shares of the groups opening beside it.
    pub fn share(
        &self,
        group: GroupId,
        share: f32,
        settled: impl Fn(GroupId) -> f32,
        now: Instant,
    ) -> f32 {
        let now = self.at(now);
        let mut drawn = share;
        for opening in &self.opening {
            let Some(k) = progress(opening.since, now, ENTER) else {
                continue;
            };
            if opening.group == group {
                drawn -= settled(group) * (1. - k);
            }
            if opening.from == group {
                drawn += settled(opening.group) * (1. - k);
            }
        }
        for folding in &self.folding {
            if folding.into == group {
                drawn -= folding.share(now);
            }
        }
        drawn.max(0.)
    }

    /// The closed groups still folding away at `now`, with the share of the
    /// row each still takes.
    pub fn folding(&self, now: Instant) -> Vec<Fold> {
        let now = self.at(now);
        self.folding
            .iter()
            .map(|folding| Fold {
                index: folding.index,
                share: folding.share(now),
            })
            .filter(|fold| fold.share > 0.)
            .collect()
    }

    /// Whether a group is still opening or folding, so the window draws
    /// another frame. Finished motion is forgotten.
    pub fn animating(&mut self, now: Instant) -> bool {
        let now = self.at(now);
        self.opening
            .retain(|opening| progress(opening.since, now, ENTER).is_some());
        self.folding
            .retain(|folding| progress(folding.since, now, LEAVE).is_some());
        !self.opening.is_empty() || !self.folding.is_empty()
    }

    /// Holds all motion where it starts, for tests whose frames are too
    /// slow to catch it mid-way.
    /// Turns motion on, for the tests of the motion itself.
    #[cfg(any(test, feature = "test-support"))]
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Fixes motion `offset` after the latest one began, for tests. Freeze
    /// before the split or close: the offset follows motion that starts
    /// later, so a slow frame in between cannot settle it first.
    #[cfg(any(test, feature = "test-support"))]
    pub fn freeze(&mut self, offset: std::time::Duration) {
        self.frozen = Some(offset);
    }
}

#[cfg(test)]
mod tests;
