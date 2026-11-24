//! The device picker's lines: what each line is in paint order, the identity
//! the keys stay on while rows arrive above them, and a fold's own manner.
//! Which device stands unfolded, and how far its sessions are open, stay the
//! window's.

use crate::sessions::{Entry, Row};
use spaceport_motion::{LEAVE, duration, progress};
use std::time::Instant;

/// A line of the picker, in paint order.
pub enum Line {
    All,
    /// A device, by its index in the window's endpoints.
    Device(usize),
    /// A session row or note under the unfolded device.
    Session(Entry),
    AddDevice,
}

impl Line {
    /// Whether the keys may stop here: a note under a device explains it
    /// rather than being something to choose.
    pub fn selectable(&self) -> bool {
        match self {
            Self::Session(entry) => entry.row().is_some(),
            _ => true,
        }
    }
}

/// A device's sessions closing after it folded: drawn under its row,
/// shrinking from where they were, until [`LEAVE`] passes.
pub struct Folding {
    pub device: String,
    pub since: Instant,
    /// Their height when they began to close, in pixels.
    pub height: f32,
    /// How open they were then, from 0 to 1.
    pub from: f32,
}

impl Folding {
    /// How open they still are at `now`, or `None` once closed.
    pub fn shown(&self, now: Instant) -> Option<f32> {
        progress(self.since, now, duration(LEAVE)).map(|k| self.from * (1. - k))
    }
}

/// What a selectable line is, apart from where it sits: the keys stay on it
/// while rows arrive above it.
#[derive(Debug, PartialEq)]
pub enum Pick {
    All,
    Device(String),
    Session(Row),
    AddDevice,
}
