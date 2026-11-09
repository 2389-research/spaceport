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

fn url(value: &str) -> Location {
    Location::Web {
        url: WebUrl::try_from(value).unwrap(),
    }
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

#[cfg(any(target_os = "macos", windows))]
mod removed_visual_notes;

/// Needs a build that shows pages: elsewhere a new tab opens nothing.
#[cfg(any(target_os = "macos", windows))]
mod embedded;

/// A page's own icon on its tab; only builds that show pages get one.
#[cfg(any(target_os = "macos", windows))]
mod favicon;

/// The pill on a divider between groups, under real pointer events.
mod divider_grip;

/// Editor groups. Blank browser tabs need no native page, so none is
/// created, and a split alone needs no page at all.
mod groups;

/// Middle-clicking a strip's tab closes it.
mod middle_click;

/// Pages a control request opens, only in a workspace the window shows.
mod requests;

/// Notes a control request sends, only to a pane the window shows.
mod requested_notes;

mod system_open;
/// Browser tabs in a group's strip: reordering and growing in.
mod tab_strip;

/// Next Tab, Previous Tab, and tabs by number step through pages too. Needs
/// a build that shows pages: elsewhere a new tab opens nothing.
#[cfg(any(target_os = "macos", windows))]
mod tab_steps;

/// Move Tab Left and Right move a page among the pages. Needs a build that
/// shows pages: elsewhere a new tab opens nothing.
#[cfg(any(target_os = "macos", windows))]
mod tab_moves;

/// Pages hide under the status bar's tooltips where they draw above the
/// window, which only Windows does.
#[cfg(windows)]
mod status_tooltip;

/// Pages hide under toasts and the file-transfer card where they draw above
/// the window, which only Windows does.
#[cfg(windows)]
mod overlays;

/// What closing a workspace does to its browser tabs.
mod workspace_close;

/// The font size keys zoom a browser tab's page. Only builds that show
/// pages open a browser tab.
#[cfg(any(target_os = "macos", windows))]
mod zoom;

/// The toolbar under `en-XB`. A blank tab opens nothing where builds show
/// no pages.
#[cfg(any(target_os = "macos", windows))]
mod rtl;

/// The `en-XA` leak test. A blank tab opens nothing where builds show no
/// pages.
#[cfg(any(target_os = "macos", windows))]
mod en_xa;

/// Terminal padding beside a page; only builds that show pages open one.
#[cfg(any(target_os = "macos", windows))]
mod terminal_padding;
