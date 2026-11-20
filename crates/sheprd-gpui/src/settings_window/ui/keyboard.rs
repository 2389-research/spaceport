// ABOUTME: Gives enabled Settings controls keyboard focus and a visible focus ring.
// ABOUTME: Reuses GPUI's keyboard click handling and theme contrast policy.
use crate::config::Theme;
use gpui::{StatefulInteractiveElement, prelude::*};

pub(in crate::settings_window) fn focusable<E: StatefulInteractiveElement>(
    theme: &Theme,
    control: E,
    enabled: bool,
) -> E {
    with_ring(control, enabled, theme.ink(theme.primary()))
}

pub(super) fn focusable_on<E: StatefulInteractiveElement>(
    theme: &Theme,
    control: E,
    background: u32,
) -> E {
    let ring = crate::contrast::ink_on_chrome(
        theme.primary(),
        [background; 3],
        theme.contrast.mark_ratio(),
    );
    with_ring(control, true, ring)
}

fn with_ring<E: StatefulInteractiveElement>(control: E, enabled: bool, color: u32) -> E {
    if !enabled {
        return control;
    }
    crate::accessibility::focusable(control, color)
}

/// Settings owns these fields, so their focus joins this window's tab order.
pub(in crate::settings_window) fn text_input(
    cx: &mut Context<crate::search_input::SearchInput>,
) -> crate::search_input::SearchInput {
    let mut input = crate::search_input::SearchInput::new(cx);
    input.focus = input.focus.clone().tab_stop(true);
    input
}
