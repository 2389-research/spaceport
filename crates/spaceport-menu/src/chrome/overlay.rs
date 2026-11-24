//! The layer over the window: what keeps everything beneath the popup inert
//! while it is open.

use super::{Hosts, Layer};
use gpui::prelude::*;
use gpui::*;
use spaceport_widgets::direction::Logical;

/// The layer that holds the panel, keeps the keyboard, and dismisses the menu
/// on a click or a right click outside it. The window's own keys, its
/// dismissal, and the focus it takes back are the app's, on this element.
pub fn overlay<K>(
    hosts: &Hosts<'_, K>,
    layer: &Layer,
    role: Role,
    title: SharedString,
    placed: AnyElement,
) -> Stateful<Div> {
    div()
        .id("menu-overlay")
        .debug_selector(|| "menu-overlay".into())
        // Focus stays here while the popup is open, so this is the menu or
        // dialog a screen reader announces.
        .role(role)
        .aria_label(title)
        .absolute()
        .top_0()
        .inset_s(px(0.))
        .bottom_0()
        .inset_e(px(0.))
        .when(layer.dims, |overlay| {
            overlay
                .row()
                .items_center()
                .justify_center()
                .bg(hosts.backdrop)
        })
        .occlude()
        .child(placed)
}
