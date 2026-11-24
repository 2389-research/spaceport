//! The menu layer coming and going. A popover fades in while it slides a few
//! pixels away from what opened it; a dialog rises into place while the window
//! dims behind it, and closing it lifts the dimming back off. Moving between
//! pages of one popup, such as Add Device's sources or a menu's rename field,
//! keeps it still: only a popup that is new to the eye moves.
//!
//! All Devices instead rises out of the sidebar footer that opened it and
//! sinks back into it when it closes. It is drawn from what was taken out of
//! the menu before the reset, which closing hands to [`Appear::keep`]: the
//! crate holds that value for the length of the leave without naming what it
//! is, so the app keeps the fields a page reads.
//!
//! Distance is `f32` pixels and an anchor is a pair of them, so this is the
//! window's own state machine without GPUI, timed by `spaceport-motion`.

use crate::page::Page;
use spaceport_motion::{ENTER, LEAVE, Toggle, progress};
use std::{cell::Cell, rc::Rc, time::Instant};

/// How far a popover slides toward its place as it appears.
pub(crate) const POPOVER_SLIDE: f32 = 6.;
/// How far a dialog rises into place as it appears.
pub(crate) const DIALOG_RISE: f32 = 10.;

/// What kind of popup the window shows, as far as its motion cares.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shown {
    /// A popover beside what opened it, at that anchor in window
    /// coordinates; one opened somewhere else is new.
    Popover { x: f32, y: f32 },
    /// A dialog over the dimmed window.
    Dialog,
}

/// Whether `page` animates out after it closes: All Devices into the footer.
pub fn leaves(page: Page) -> bool {
    matches!(page, Page::Devices)
}

/// How far a popup `shown` of the way in still has to go: a popover toward
/// its place beside its anchor, a dialog up into the middle.
pub fn offset(dims: bool, shown: f32) -> f32 {
    let distance = if dims { DIALOG_RISE } else { POPOVER_SLIDE };
    (1. - shown) * distance
}

/// Which popup the last frame drew, which page, and what is still leaving.
/// `K` is what a closing page reads, which the app takes out of its own state
/// and hands over with [`Appear::keep`].
#[derive(Default)]
pub struct Appear<K = ()> {
    /// What the last frame drew, `None` while no popup was open.
    shown: Option<Shown>,
    /// The page the last frame drew.
    page: Option<Page>,
    /// When the popup showing now began to appear. Public so a test can hold a
    /// motion at its start instead of waiting it out.
    pub since: Option<Instant>,
    /// A page going away after it closed, from when it did. Public for the
    /// same reason, and for a test that starts a leave of its own.
    pub leaving: Option<(Page, Instant)>,
    /// What the page that closed reads, taken out as it closed.
    pub kept: Option<(Page, K)>,
    /// All Devices' height as last laid out, which its rise travels.
    pub rise_height: Rc<Cell<Option<f32>>>,
    backdrop: Toggle,
}

impl<K> Appear<K> {
    /// Takes the popup this frame draws and the page it draws. `still` takes
    /// the change without a motion: unit tests lay menus out where they settle
    /// unless one asks to see them move.
    pub fn observe(&mut self, shown: Option<Shown>, page: Option<Page>, now: Instant, still: bool) {
        let last = std::mem::replace(&mut self.page, page);
        if shown == self.shown {
            return;
        }
        // A session form closing is All Devices' page with its form over it:
        // it goes at once like any other form, rather than the picker sinking
        // into the footer in the form's place.
        let form = self.shown == Some(Shown::Dialog) && last == Some(Page::Devices);
        self.leaving = match (shown, last) {
            (None, Some(page)) if !still && !form && leaves(page) => Some((page, now)),
            _ => None,
        };
        // Only the page that just closed keeps what it read; anything else
        // drops it, cancelling nothing more since closing did that.
        if self.leaving.map(|(page, _)| page) != self.kept.as_ref().map(|(page, _)| *page) {
            self.kept = None;
        }
        // All Devices stays drawn beneath a session form it opened, so closing
        // the form uncovers it where it was rather than showing it anew; only
        // the dimming lifts.
        let uncovered = page == Some(Page::Devices)
            && last == page
            && self.shown == Some(Shown::Dialog)
            && matches!(shown, Some(Shown::Popover { .. }));
        self.since = shown
            .is_some()
            .then_some(now)
            .filter(|_| !still && !uncovered);
        self.backdrop.set(shown == Some(Shown::Dialog), now, still);
        self.shown = shown;
    }

    /// How much of the open popup shows at `now`, from 0 to 1.
    pub fn panel(&self, now: Instant) -> f32 {
        self.since
            .and_then(|since| progress(since, now, ENTER))
            .unwrap_or(1.)
    }

    /// Keeps what the closing `page` reads, for it to be drawn as it goes.
    pub fn keep(&mut self, page: Page, kept: K) {
        self.kept = Some((page, kept));
    }

    /// How much of the dimming behind a dialog shows at `now`, from 0 to 1. It
    /// keeps fading for a moment after the dialog has gone.
    pub fn backdrop(&self, now: Instant) -> f32 {
        self.backdrop.shown(now)
    }

    /// The page sinking away after it closed, and how much of it is left.
    pub fn leaving(&self, now: Instant) -> Option<(Page, f32)> {
        let (page, since) = self.leaving?;
        progress(since, now, LEAVE).map(|k| (page, 1. - k))
    }

    /// Lets go of a closed page once it has gone, with what it read.
    pub fn settle(&mut self, now: Instant) {
        if self.leaving.is_some() && self.leaving(now).is_none() {
            self.leaving = None;
            self.kept = None;
        }
    }

    /// Whether anything is still moving, so the window keeps drawing frames.
    pub fn moving(&self, now: Instant) -> bool {
        self.panel(now) < 1. || self.backdrop.moving(now) || self.leaving(now).is_some()
    }
}
