//! Where each row was last painted, by row id, for scrolling to it, and the
//! probe that records it.

use gpui::*;
use std::collections::HashMap;

/// Where each row was last painted, by row id, for scrolling to it.
#[derive(Default)]
pub struct RowBounds(pub HashMap<SharedString, Bounds<Pixels>>);
impl Global for RowBounds {}

/// Records the bounds of the element it is a child of under `id`. Absolute, so
/// it takes no room in the row's layout.
pub fn track_bounds(id: SharedString) -> impl IntoElement {
    canvas(
        move |bounds, _, cx| {
            cx.default_global::<RowBounds>().0.insert(id, bounds);
        },
        |_, _, _, _| {},
    )
    .absolute()
    .size_full()
}

/// Space kept above a row that search scrolls to the top of the body.
pub const SCROLL_MARGIN: f32 = 12.;
