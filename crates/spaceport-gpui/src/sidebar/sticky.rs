//! The host header pinned above the spaces list while that host's workspaces
//! scroll under it, as section headers stay in native lists, so a scrolled
//! list still says which machine its rows belong to. Which header pins is
//! `spaceport-sidebar`'s; settling the scroll it is read from is the window's.

pub(super) use spaceport_sidebar::sticky::{HostHeader, cover, pinned};

/// Clamps `scroll` to the extent its last frame measured, which is what draws.
/// The wheel moves an offset without bounds and GPUI clamps it only in
/// prepaint, after render pinned headers and judged visibility from it: a list
/// that fits would pin a header it never scrolls past. Call before reading it.
pub(super) fn settle(scroll: &gpui::ScrollHandle) {
    let offset = scroll.offset();
    let clamped = offset
        .min(&gpui::Point::default())
        .max(&-scroll.max_offset());
    if clamped != offset {
        scroll.set_offset(clamped);
    }
}
