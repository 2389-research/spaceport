//! Tabs grow into a strip when they open and shrink out of it when they
//! close, so the eye follows where one landed or where one went. Each group's
//! strip is tracked on its own, so a tab closed in one group shrinks out of
//! that strip alone. The tabs a strip already has when the window first draws
//! it simply appear.
use super::{GroupId, Pick};
use gpui::SharedString;
use sheprd_motion::{ENTER, LEAVE, progress};
use std::{collections::HashMap, time::Instant};

/// A tab on its way out of a strip, drawn where it stood as it shrinks.
#[derive(Clone, Debug, PartialEq)]
pub struct Leaving {
    pub pick: Pick,
    pub label: SharedString,
    /// Its place in the strip when it closed.
    pub index: usize,
    /// Its width when it closed.
    pub width: f32,
    since: Instant,
}

impl Leaving {
    /// How much of it is left at `now`, from 1 to 0.
    pub fn left(&self, now: Instant) -> f32 {
        progress(self.since, now, LEAVE).map_or(0., |k| 1. - k)
    }
}

/// A tab a strip draws: what it is, what it says, and how wide it is.
pub struct Listed {
    pub pick: Pick,
    pub label: SharedString,
    pub width: f32,
}

#[derive(Default)]
struct Strip {
    tabs: Vec<Listed>,
    growing: HashMap<Pick, Instant>,
    leaving: Vec<Leaving>,
}

#[derive(Default)]
pub struct TabAppear {
    strips: HashMap<GroupId, Strip>,
    /// Set by `hold`: motions that start from then on start held too.
    #[cfg(any(test, feature = "test-support"))]
    held: bool,
}

impl TabAppear {
    /// Records the tabs `group`'s strip draws at `now`. The first time a
    /// strip is seen its tabs are taken as they are; after that, a new tab
    /// grows in and a missing one shrinks out where it stood.
    pub fn observe(&mut self, group: GroupId, tabs: Vec<Listed>, now: Instant) {
        #[cfg(any(test, feature = "test-support"))]
        let start = if self.held { held_start() } else { now };
        #[cfg(not(any(test, feature = "test-support")))]
        let start = now;
        let Some(strip) = self.strips.get_mut(&group) else {
            self.strips.insert(
                group,
                Strip {
                    tabs,
                    ..Default::default()
                },
            );
            return;
        };
        for tab in &tabs {
            if !strip.tabs.iter().any(|old| old.pick == tab.pick) {
                strip.growing.insert(tab.pick.clone(), start);
                // A tab that comes back is no longer leaving.
                strip.leaving.retain(|leaving| leaving.pick != tab.pick);
            }
        }
        for (index, old) in strip.tabs.iter().enumerate() {
            if !tabs.iter().any(|tab| tab.pick == old.pick) {
                strip.leaving.push(Leaving {
                    pick: old.pick.clone(),
                    label: old.label.clone(),
                    index,
                    width: old.width,
                    since: start,
                });
            }
        }
        strip.tabs = tabs;
        strip
            .growing
            .retain(|_, since| progress(*since, now, ENTER).is_some());
        strip
            .leaving
            .retain(|leaving| progress(leaving.since, now, LEAVE).is_some());
    }

    /// How far `tab` has grown into `group`'s strip at `now`, or `None`
    /// once it is whole.
    pub fn growth(&self, group: GroupId, tab: &Pick, now: Instant) -> Option<f32> {
        let since = self.strips.get(&group)?.growing.get(tab)?;
        progress(*since, now, ENTER)
    }

    /// The tabs shrinking out of `group`'s strip.
    pub fn leaving(&self, group: GroupId) -> &[Leaving] {
        self.strips
            .get(&group)
            .map_or(&[], |strip| strip.leaving.as_slice())
    }

    /// Whether a tab is still growing or shrinking, so the window draws
    /// another frame.
    pub fn animating(&self) -> bool {
        self.strips
            .values()
            .any(|strip| !strip.growing.is_empty() || !strip.leaving.is_empty())
    }

    /// Forgets the strips of groups that are gone.
    pub fn retain(&mut self, mut live: impl FnMut(GroupId) -> bool) {
        self.strips.retain(|group, _| live(*group));
    }

    /// Holds every moving tab where it started, and every one that starts
    /// later, for tests whose frames are too slow to catch one mid-way.
    /// Holding a motion only once it has begun would race the frame that
    /// begins it: on a slow machine that frame can outlast the motion.
    #[cfg(any(test, feature = "test-support"))]
    pub fn hold(&mut self) {
        self.held = true;
        let later = held_start();
        for strip in self.strips.values_mut() {
            for since in strip.growing.values_mut() {
                *since = later;
            }
            for leaving in &mut strip.leaving {
                leaving.since = later;
            }
        }
    }
}

/// When a held motion starts: far enough ahead that it never moves.
#[cfg(any(test, feature = "test-support"))]
fn held_start() -> Instant {
    Instant::now() + std::time::Duration::from_secs(3600)
}

#[cfg(test)]
mod tests;
