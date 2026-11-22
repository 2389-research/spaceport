//! Reordering by dragging, shared by the sidebar's workspace rows and the
//! tab strips: when a press lifts, how the carried item finds its gap among
//! the others, and how the others slide aside to preview the drop. Positions
//! are along one axis, so the same rules serve a column and a row.

use std::time::{Duration, Instant};

/// How long a press rests before what it pressed lifts.
pub const LIFT_DELAY: Duration = Duration::from_millis(250);
/// How far a press may travel before it lifts without waiting.
pub const LIFT_DISTANCE: f32 = 4.;
/// How long a dropped preview waits for the daemon's new order before the
/// list falls back to the order it last received.
pub const SETTLE_TIMEOUT: Duration = Duration::from_secs(2);
/// How long an item takes to slide to its previewed place. A test build that
/// enables this crate's `test-support` feature settles at once: its harness
/// checks where rows go, and a frame clock would only add waits.
pub const SLIDE: Duration = if cfg!(feature = "test-support") {
    Duration::ZERO
} else {
    Duration::from_millis(150)
};

/// Where a dropped item lands: beside another, since a drop past the last
/// item shown still has to land after it rather than at the very end of a
/// longer list the view shows only part of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Beside<T> {
    Before(T),
    After(T),
}

impl<T> Beside<T> {
    /// The same side of what `f` makes of the anchor, if it makes anything.
    pub fn and_then<U>(self, f: impl FnOnce(T) -> Option<U>) -> Option<Beside<U>> {
        Some(match self {
            Self::Before(anchor) => Beside::Before(f(anchor)?),
            Self::After(anchor) => Beside::After(f(anchor)?),
        })
    }
}

/// An item gliding between two shifts, eased out so it arrives gently.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slide {
    pub from: f32,
    pub to: f32,
    pub start: Instant,
}

impl Slide {
    pub fn progress(&self, now: Instant, duration: Duration) -> f32 {
        if duration.is_zero() {
            return 1.;
        }
        (now.saturating_duration_since(self.start).as_secs_f32() / duration.as_secs_f32()).min(1.)
    }

    pub fn at(&self, now: Instant, duration: Duration) -> f32 {
        let eased = 1. - (1. - self.progress(now, duration)).powi(3);
        self.from + (self.to - self.from) * eased
    }

    /// Heads for `to` from wherever the row is now, so a gap that moves
    /// mid-slide turns the row around instead of making it jump.
    pub fn toward(self, to: f32, now: Instant, duration: Duration) -> Self {
        if (self.to - to).abs() <= f32::EPSILON {
            return self;
        }
        Self {
            from: self.at(now, duration),
            to,
            start: now,
        }
    }
}

/// The gap the carried card points to, from every unit's resting span and
/// the card's own. Moving up, the card passes a unit once its top edge is
/// above that unit's middle; moving down, once its bottom edge is below it.
/// Units without a span (hidden rows) never block the card.
pub fn slot_for(spans: &[Option<(f32, f32)>], dragged: usize, card: (f32, f32)) -> usize {
    let center = |unit: usize| spans[unit].map(|(top, bottom)| (top + bottom) / 2.);
    if let Some(unit) = (0..dragged).find(|&unit| center(unit).is_some_and(|c| card.0 < c)) {
        return unit;
    }
    (dragged + 1..spans.len())
        .rev()
        .find(|&unit| center(unit).is_some_and(|c| card.1 > c))
        .map_or(dragged, |unit| unit + 1)
}

/// How far each unit moves to preview dropping `dragged` into `slot`, given
/// every unit's height: the units it passes close its place, and it takes the
/// room they left. A gap that moves nothing moves no row.
pub fn preview(heights: &[f32], dragged: usize, slot: usize) -> Vec<f32> {
    let mut shifts = vec![0.; heights.len()];
    let hole = heights[dragged];
    if slot > dragged + 1 && slot <= heights.len() {
        shifts[dragged + 1..slot].fill(-hole);
        shifts[dragged] = heights[dragged + 1..slot].iter().sum();
    } else if slot < dragged {
        shifts[slot..dragged].fill(hole);
        shifts[dragged] = -heights[slot..dragged].iter().sum::<f32>();
    }
    shifts
}

#[cfg(test)]
mod tests;
