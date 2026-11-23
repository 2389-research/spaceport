//! Reordering devices by dragging their headers. A press lifts a header as it
//! lifts a workspace row; while it is lifted the list folds to its headers,
//! so a device with many rows moves as one line among the others, and the
//! headers it passes slide aside to open the gap it lands in. Releasing there
//! moves the device for this client, Local included.
//!
//! The press, its lift, and the plan a folded list is drawn from are here.
//! Which headers list, what the viewport laid out, and the endpoint a release
//! moves stay in the app.

use gpui::{Pixels, Point, ScrollHandle, Task};
use spaceport_reorder::{Beside, SLIDE, Slide, slot_for};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    time::Instant,
};

/// A press on a device's header that may become a reorder.
pub struct DeviceDrag {
    pub endpoint: String,
    pub origin: Point<Pixels>,
    pub pointer: Point<Pixels>,
    /// How far below the header's top the press landed. The list folds when
    /// the header lifts, moving it, so the header follows the pointer from
    /// that spot rather than from where it was pressed.
    pub grab: Pixels,
    pub lifted: bool,
    /// The gap under the pointer and the move it makes, `None` over the
    /// carried header's own place.
    pub target: Option<(usize, Beside<String>)>,
    /// Whether a folded frame has been laid out: until then the list's
    /// geometry is the unfolded one and cannot place the headers.
    measured: Cell<bool>,
    /// Each header's slide, by endpoint ID. Render reads and advances them.
    slides: RefCell<HashMap<String, Slide>>,
    _timer: Task<()>,
}

impl DeviceDrag {
    /// A press on `endpoint`'s header, `grab` below its top, that `lift`
    /// raises once it has rested.
    pub fn press(endpoint: String, position: Point<Pixels>, grab: Pixels, lift: Task<()>) -> Self {
        Self {
            endpoint,
            origin: position,
            pointer: position,
            grab,
            lifted: false,
            target: None,
            measured: Cell::new(false),
            slides: RefCell::default(),
            _timer: lift,
        }
    }

    pub fn carries(&self, endpoint: &str) -> bool {
        self.lifted && self.endpoint == endpoint
    }

    /// Whether a folded frame has been laid out yet, counting this one.
    pub fn mark_measured(&self) -> bool {
        self.measured.replace(true)
    }

    /// Where the header paints, in the window, when it is the carried one: it
    /// rests where the folded list put it and paints where the pointer holds
    /// it. Leaves the slide hovering where it last was, so a later drop
    /// slides from the pointer.
    pub fn pin(&self, endpoint: &str, at: f32, now: Instant) {
        self.slides.borrow_mut().insert(
            endpoint.to_owned(),
            Slide {
                from: at,
                to: at,
                start: now,
            },
        );
    }

    /// Where the carried header last painted, which a redraw adds the
    /// pointer's drift to.
    pub fn pinned(&self, endpoint: &str) -> f32 {
        self.slides
            .borrow()
            .get(endpoint)
            .map_or(0., |slide| slide.to)
    }

    /// Where a header paints on its way to shift `to`, and whether it is
    /// still moving.
    pub fn slide(&self, endpoint: &str, to: f32, now: Instant) -> (f32, bool) {
        let mut slides = self.slides.borrow_mut();
        let slide = slides.entry(endpoint.to_owned()).or_insert(Slide {
            from: 0.,
            to: 0.,
            start: now,
        });
        *slide = slide.toward(to, now, SLIDE);
        (slide.at(now, SLIDE), slide.progress(now, SLIDE) < 1.)
    }
}

/// What the pointer handler needs from the folded list render laid out:
/// each movable header's unit, row, and shift, and the devices in order.
#[derive(Clone, Debug, Default)]
pub struct DeviceDrop {
    pub rows: Vec<(usize, usize, Pixels)>,
    pub ids: Vec<String>,
    pub dragged: usize,
}

/// The folded list's headers while a device is lifted.
pub struct DevicePlan {
    /// Each listed header's shift by endpoint index, and whether it is the
    /// carried one.
    pub shifts: HashMap<usize, (Pixels, bool)>,
    pub drop: DeviceDrop,
    /// Whether a header is still sliding, or the fold still needs measuring.
    pub sliding: bool,
}

impl DevicePlan {
    pub fn shift(&self, endpoint: usize) -> (Pixels, bool) {
        self.shifts.get(&endpoint).copied().unwrap_or_default()
    }
}

/// Where row `row` of the spaces list sits in the window. The handle reports
/// where layout put it, before the list's scroll offset moves it.
pub fn window_top(scroll: &ScrollHandle, row: usize) -> Option<Pixels> {
    Some(scroll.bounds_for_item(row)?.top() + scroll.offset().y)
}

/// The gap a header carried with its top at `card_top`, in window
/// coordinates, points to against the geometry the folded list last laid out.
fn slot(drop: &DeviceDrop, scroll: &ScrollHandle, card_top: Pixels) -> Option<usize> {
    let mut spans = vec![None; drop.ids.len()];
    // Headers are measured where they rest, not where the preview slid them.
    for &(unit, row, shift) in &drop.rows {
        let (Some(top), Some(bounds)) = (window_top(scroll, row), scroll.bounds_for_item(row))
        else {
            continue;
        };
        let top = f32::from(top - shift);
        spans[unit] = Some((top, top + f32::from(bounds.size.height)));
    }
    let (top, bottom) = (*spans.get(drop.dragged)?)?;
    let card = f32::from(card_top);
    Some(slot_for(&spans, drop.dragged, (card, card + bottom - top)))
}

/// The move dropping into `slot` makes: before the device that holds it, or
/// after the last one. `None` for the gaps around the carried device.
fn landing(ids: &[String], dragged: usize, slot: usize) -> Option<Beside<String>> {
    if slot == dragged || slot == dragged + 1 {
        return None;
    }
    match ids.get(slot) {
        Some(id) => Some(Beside::Before(id.clone())),
        None => ids.last().cloned().map(Beside::After),
    }
}

/// The gap the pointer at `card_top` points to, and the move it would make.
pub fn gap(
    drop: &DeviceDrop,
    scroll: &ScrollHandle,
    card_top: Pixels,
) -> Option<(usize, Beside<String>)> {
    let slot = slot(drop, scroll, card_top)?;
    Some((slot, landing(&drop.ids, drop.dragged, slot)?))
}

#[cfg(test)]
mod tests;
