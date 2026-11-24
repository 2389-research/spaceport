//! Where the kit painted each probed element, for the app's native macOS tests,
//! which click a control by the bounds it reports. Built only into a
//! test-enabled app.

use gpui::*;
use spaceport_widgets::direction::Logical;
use std::collections::HashMap;

/// Where each probed element was painted, by id.
#[derive(Default)]
pub struct NativeBounds(HashMap<String, Bounds<Pixels>>);
impl Global for NativeBounds {}

/// Records the bounds of the element it is a child of under `id`, for a native
/// test to click it by. Absolute, so it takes no room in its parent's layout.
pub fn native_probe(id: impl Into<String>) -> impl IntoElement {
    let id = id.into();
    canvas(
        |_, _, _| (),
        move |bounds, _, _, cx| {
            cx.default_global::<NativeBounds>()
                .0
                .insert(id.clone(), bounds);
        },
    )
    .absolute()
    .top_0()
    .inset_s(px(0.))
    .size_full()
}

/// Where the element probed as `id` was last painted.
pub fn bounds(cx: &App, id: &str) -> Option<Bounds<Pixels>> {
    cx.try_global::<NativeBounds>()?.0.get(id).copied()
}

/// Forgets every painted bounds, so the next frame's are read.
pub fn clear(cx: &mut App) {
    cx.default_global::<NativeBounds>().0.clear();
}
