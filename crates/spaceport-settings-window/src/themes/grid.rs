//! The card grid's geometry and the palettes it has already drawn: how many
//! cards a width holds, where a step lands, and a bounded cache of prepared
//! palettes, fenced by the listing they were loaded for.

use super::Choice;
use spaceport_settings::Theme;
use std::{collections::VecDeque, ops::Range};

/// How many palettes the grid keeps before the oldest goes: a long catalogue
/// scrolled from end to end must not grow with it.
const CACHE_LIMIT: usize = 64;

/// How many cards fit across a grid `width` wide.
pub fn columns(width: f32) -> usize {
    ((width + 6.) / 174.).floor().clamp(2., 6.) as usize
}

/// Where a step from `index` lands: a row up or down, kept inside `count`.
pub fn navigate(index: usize, count: usize, columns: usize, up: bool) -> usize {
    if up {
        index.saturating_sub(columns)
    } else {
        (index + columns).min(count.saturating_sub(1))
    }
}

/// What the grid has drawn and what it is loading.
#[derive(Debug)]
pub struct Grid {
    /// How many cards fit across the drawn grid.
    pub columns: usize,
    /// Bumped by [`Grid::invalidate`], so palettes loaded for a listing the
    /// grid no longer lists are dropped instead of drawn.
    pub epoch: u64,
    /// Whether a batch was deferred to the next frame's work.
    pub queued: bool,
    /// Whether a batch is loading.
    pub running: bool,
    /// The cards on screen, by their place in the listing.
    pub visible: Range<usize>,
    /// The palettes loaded so far, oldest first; `None` is a theme that could
    /// not load, which is drawn as unavailable rather than as loading again.
    pub palettes: VecDeque<(Choice, Option<Theme>)>,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            columns: 2,
            epoch: 0,
            queued: false,
            running: false,
            visible: 0..0,
            palettes: VecDeque::new(),
        }
    }
}

impl Grid {
    /// Forgets every palette: the listing they were loaded for is gone.
    pub fn invalidate(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.palettes.clear();
    }

    /// Takes a batch loaded for `epoch`, unless the listing changed while it
    /// was loading.
    pub fn finish(&mut self, epoch: u64, palettes: Vec<(Choice, Option<Theme>)>) {
        self.running = false;
        if self.epoch != epoch {
            return;
        }
        for palette in palettes {
            if self.palettes.len() == CACHE_LIMIT {
                self.palettes.pop_front();
            }
            self.palettes.push_back(palette);
        }
    }

    /// The palette loaded for `choice`, or none while it has not loaded.
    pub fn palette(&self, choice: &Choice) -> Option<&Option<Theme>> {
        self.palettes
            .iter()
            .find(|(cached, _)| cached == choice)
            .map(|(_, theme)| theme)
    }

    /// Whether every card on screen has a palette loaded, so no load is owed.
    pub fn covers(&self, listing: &[Choice]) -> bool {
        listing
            .get(self.visible.clone())
            .unwrap_or_default()
            .iter()
            .all(|choice| self.palette(choice).is_some())
    }
}

#[cfg(test)]
mod tests;
