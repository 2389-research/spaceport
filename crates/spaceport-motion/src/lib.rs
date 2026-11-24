//! How long things that come and go in the window take to arrive and leave,
//! and the clock they are measured by. Each motion is a function of when it
//! started and the time now, so a frame never depends on the one before it.
//!
//! QA > Slow Animations runs every animation in the window at a tenth of its
//! speed, to watch how it moves. Animations read the time from [`now`] rather
//! than [`Instant::now`], and GPUI's own animations take their length through
//! [`duration`]. While the switch is on, `now` advances a tenth as fast from
//! the moment it was turned on; turning it off returns to the real time, so
//! whatever is mid-way finishes at once.
//!
//! The switch is process-wide, like the app's menu bar that holds it, and
//! reaches places that draw without an app context.

use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

/// How many times slower animations run while the switch is on.
const FACTOR: u32 = 10;

/// How long something takes to grow in.
pub const ENTER: Duration = Duration::from_millis(180);
/// How long something takes to shrink out; leaving is a little quicker.
pub const LEAVE: Duration = Duration::from_millis(150);

/// The most frames a second a repeating mark asks for. A turning or pulsing
/// mark reads as smooth at this rate, and it is what a working agent costs:
/// every frame of one mark rebuilds its whole window's scene, measured at
/// 4.0 ms in a 1200x780 window with a 40-workspace sidebar, whatever the
/// number of marks on screen. That is 8% of a core while any agent works,
/// against 12% at 30 fps and none at all once nothing moves. One home, so a
/// mark drawn in a row and one drawn in the window's layer turn at one rate.
pub const MARK_MAX_FPS: f32 = 20.;

/// When slow motion was turned on, while it is.
static SINCE: Mutex<Option<Instant>> = Mutex::new(None);

fn since() -> Option<Instant> {
    SINCE.lock().map_or(None, |since| *since)
}

/// Turns slow motion on or off.
pub fn set_slow(slow: bool) {
    if let Ok(mut since) = SINCE.lock() {
        *since = slow.then(Instant::now);
    }
}

/// Whether animations run slowly.
pub fn is_slow() -> bool {
    since().is_some()
}

/// The time animations read: the real time, or while slow motion is on,
/// time that has passed a tenth as fast since it was turned on.
pub fn now() -> Instant {
    at(Instant::now(), since())
}

/// `length` as an animation GPUI times itself should last.
pub fn duration(length: Duration) -> Duration {
    stretch(length, is_slow())
}

/// How far a motion that began at `since` has come at `now`, eased out, or
/// `None` once it is done.
pub fn progress(since: Instant, now: Instant, length: Duration) -> Option<f32> {
    let k = now.saturating_duration_since(since).as_secs_f32() / length.as_secs_f32();
    (k < 1.).then(|| 1. - (1. - k.clamp(0., 1.)).powi(3))
}

fn at(real: Instant, since: Option<Instant>) -> Instant {
    match since {
        Some(since) => since + real.saturating_duration_since(since) / FACTOR,
        None => real,
    }
}

fn stretch(length: Duration, slow: bool) -> Duration {
    if slow { length * FACTOR } else { length }
}

/// Something that opens and closes, such as a panel: how much of it shows.
#[derive(Clone, Copy, Debug, Default)]
pub struct Toggle {
    open: bool,
    /// When it last opened or closed; `None` before either, so a panel
    /// that is open when first drawn is simply there.
    since: Option<Instant>,
}

impl Toggle {
    /// Opens or closes it at `now`. The first report sets where it is
    /// without moving, as a view first drawn shows what it has.
    pub fn set(&mut self, open: bool, now: Instant, first: bool) {
        if first {
            *self = Self { open, since: None };
            return;
        }
        if open != self.open {
            // Reversing mid-way starts from where it is, not from an end.
            let shown = self.shown(now);
            let length = if open { ENTER } else { LEAVE };
            let done = if open { shown } else { 1. - shown };
            // Inverts the ease: the time at which it would have come this far.
            let back = length.mul_f32(1. - (1. - done.clamp(0., 1.)).cbrt());
            self.open = open;
            self.since = Some(now.checked_sub(back).unwrap_or(now));
        }
    }

    /// Whether it is open or opening, rather than closed or closing.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// How much of it shows at `now`, from 0 closed to 1 open.
    pub fn shown(&self, now: Instant) -> f32 {
        let length = if self.open { ENTER } else { LEAVE };
        let k = self
            .since
            .and_then(|since| progress(since, now, length))
            .unwrap_or(1.);
        if self.open { k } else { 1. - k }
    }

    /// Whether it is still opening or closing.
    pub fn moving(&self, now: Instant) -> bool {
        let length = if self.open { ENTER } else { LEAVE };
        self.since
            .is_some_and(|since| progress(since, now, length).is_some())
    }
}

#[cfg(test)]
mod tests;
