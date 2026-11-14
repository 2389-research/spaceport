#![allow(clippy::unwrap_used)]

use super::{Location, Store, WebUrl, view::scope};
use crate::{
    SheprdWindow,
    sidebar::layout_tests::{fixture_window, full_draw, snapshot},
};
use gpui::{Entity, VisualTestContext};
use std::sync::Arc;

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
}

/// The fixture window, showing workspace `w0` as a connected daemon would.
fn window(cx: &mut gpui::TestAppContext) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut shown = snapshot(40);
        shown.focused_workspace_id = Some("w0".into());
        shown.focused_tab_id = Some("t0".into());
        view.live.snapshot = Some(Arc::new(shown));
        view
    })
}

/// The pill on a divider between groups, under real pointer events.
mod divider_grip;

/// Shared native tab and terminal groups.
mod groups;

/// Middle-clicking a strip's tab closes it.
mod middle_click;

/// Pages a control request opens, only in a workspace the window shows.
mod requests;

/// Explicit URL and file opening through the system handler.
mod system_open;
/// Browser tabs in a group's strip: reordering and growing in.
mod tab_strip;

/// Next Tab, Previous Tab, and tab numbers include native content.
mod tab_steps;

/// Move Tab Left and Right reorder native content tabs.
mod tab_moves;

/// What closing a workspace does to its browser tabs.
mod workspace_close;

/// Native content shares tabs, focus, and close behavior with terminals.
mod native_tabs;
