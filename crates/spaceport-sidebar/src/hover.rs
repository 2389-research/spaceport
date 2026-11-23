//! What a resting pointer is claiming in the list: the row it entered and how
//! long it has held it, and the menu its dwell opened. The rest and the menu it
//! opened are separate so a small pointer jitter inside the slop does not
//! cancel a pending open.
//!
//! These are the claims only. Whether the feature is on, whether a menu may
//! open over the current connection, and whether the pointer is still inside
//! the popup are the window's to read and to decide.

use gpui::{Pixels, Point};
use std::time::Instant;

pub struct HoverRest {
    pub workspace: String,
    pub position: Point<Pixels>,
    /// Where the list stood when the row was entered. Scrolling slides other
    /// rows under a still pointer, so a changed offset voids this dwell.
    pub scroll: Point<Pixels>,
    pub since: Instant,
    /// The pointer has moved since it entered the row. Dismissing a menu leaves
    /// the pointer where it was, so without this the menu would reopen under it.
    pub moved: bool,
    /// The row was clicked while the pointer was on it. A click already chose
    /// what to do with the row, so its menu waits until the pointer leaves.
    pub pressed: bool,
}

/// A menu the pointer opened by resting. It closes again as soon as the pointer
/// moves anywhere but into it, so a menu nobody asked for needs no click to go.
pub struct HoverMenu {
    /// Where the pointer stood when the menu opened.
    pub position: Point<Pixels>,
    /// The pointer is over the popup. Only its own hover reports this: the popup
    /// may be snapped away from the pointer that opened it.
    pub inside: bool,
}
