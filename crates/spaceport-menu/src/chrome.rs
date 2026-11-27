// ABOUTME: Measures and paints a popup's panel, placement and overlay.
// ABOUTME: Keeps popup input and focus isolated from what lies beneath it.
//! The popup shell the app lays in its window: where the panel sits, how much
//! room each page takes, the surface it paints on, and the overlay that keeps
//! everything beneath it inert. A page's rows are [`crate::rows`]', painted the
//! same way by every page and by the in-window app menu.
//!
//! A page is the window's, so the app builds its body and hands it over as an
//! element; what is here is the chrome around it. The app owns the window and
//! every word, and supplies what only it knows as [`Hosts`] — the page's
//! appear state, the anchor, the window's measurements, the theme — and keeps
//! the keyboard and pointer handlers, which are the window's own.

mod overlay;
mod panel;

pub use overlay::overlay;
pub use panel::{anchor_footer_panel, leaving, panel, placement};

use crate::appear::{Appear, offset};
use crate::page::Page;
use gpui::{Font, Pixels, Point, Rgba, Size, px};
use spaceport_settings::Theme;
use std::time::Instant;

/// The margin a footer panel or a popover keeps from the window's edges.
pub const MENU_MARGIN: f32 = 8.;
/// Below this panel width the Add Device dialog shows its rail of sources as
/// icons, leaving the chosen source's form its room.
pub const ADD_DEVICE_COMPACT_BELOW: f32 = 560.;
/// The Add Device dialog's width when there is room: the rail and a form
/// beside it.
pub const ADD_DEVICE_WIDTH: f32 = 640.;
/// The Add Device dialog's height, the same for every source so switching
/// never resizes it: room for the SSH form and setup prompts. A form taller
/// than that scrolls inside.
pub const ADD_DEVICE_HEIGHT: f32 = 620.;
/// The width the All Devices picker's panel takes.
pub const DEVICES_MENU_WIDTH: f32 = 280.;
/// The gap that panel keeps above the footer control that opened it.
pub const DEVICES_MENU_GAP: f32 = 12.;

/// The font the chrome is set in, as the app configured it. The family and its
/// fallback cascade are applied without weight or style, which nested chrome
/// inherits, as the app's own text font keeps them.
#[derive(Clone, Debug)]
pub struct Text {
    /// The face every label is shaped in.
    pub font: Font,
    /// The distance between two lines of it.
    pub line_height: f32,
    /// Its size.
    pub size: f32,
}

/// The window's own measurements that the chrome lays out against.
#[derive(Clone, Copy, Debug)]
pub struct Metrics {
    /// The height the titlebar and the application menu bar take at the top.
    pub chrome: f32,
    /// The window's viewport, as the frame being drawn was laid out in.
    pub viewport: Size<Pixels>,
}

/// What one window offers the popup chrome for a single call.
pub struct Hosts<'a, K> {
    /// Whether the page is one source of the Add Device dialog.
    pub add_device: bool,
    /// Where the popup's anchor sits, in window coordinates.
    pub anchor: Point<Pixels>,
    /// How the open popup, or the dimming a closed one left, moves.
    pub appear: &'a Appear<K>,
    /// The dimming a dialog lays over the window behind it, at `now`.
    pub backdrop: Rgba,
    /// Whether the host menu lists forward ports rather than its actions,
    /// which widens it.
    pub host_forwards: bool,
    /// How much of a page that just closed still shows, for the one frame
    /// that draws it as it goes.
    pub leaving: Option<f32>,
    /// The window's own measurements.
    pub metrics: Metrics,
    /// `spaceport_motion::now()`, read once per call so every motion in it
    /// agrees.
    pub now: Instant,
    /// Whether the Go To palette has room to show its preview beside it.
    pub palette_previews: bool,
    /// The width the worktree popover measured for its captions, for that
    /// page alone.
    pub popover: Option<Pixels>,
    /// Whether a session form covers the All Devices picker, which turns that
    /// page into a dialog for as long as it is there.
    pub session_modal: bool,
    /// The font the chrome is set in.
    pub text: Text,
    /// The colours the chrome paints with.
    pub theme: &'a Theme,
}

/// What the popup measured once, before it draws: where it hangs, how big the
/// window is, and how far it has come in.
pub struct Layer {
    pub add_device: bool,
    pub compact_rail: bool,
    pub dims: bool,
    pub footer_anchored: bool,
    pub leaving: Option<f32>,
    pub lift: Pixels,
    pub now: Instant,
    pub pointer_anchored: bool,
    pub popover: Option<Pixels>,
    pub rises: bool,
    pub session_modal: bool,
    pub shown: f32,
    pub viewport: Size<Pixels>,
}

/// What the popup measures before it draws `page`.
pub fn layer<K>(hosts: &Hosts<'_, K>, page: Page) -> Layer {
    let text = &hosts.text;
    let viewport = hosts.metrics.viewport;
    let session_modal = hosts.session_modal;
    let now = hosts.now;
    let shown = hosts.leaving.unwrap_or_else(|| hosts.appear.panel(now));
    let dims = dims(page, session_modal);
    Layer {
        add_device: hosts.add_device,
        // Add Device's sources share one wide dialog; a narrow window keeps
        // the rail to icons so the form still has room.
        compact_rail: viewport.width - px(32.) < px(ADD_DEVICE_COMPACT_BELOW * (text.size / 12.)),
        dims,
        footer_anchored: footer_anchored(page, session_modal),
        leaving: hosts.leaving,
        lift: px(offset(dims, shown)),
        now,
        pointer_anchored: page.pointer_anchored(),
        popover: (page == Page::Workspace).then_some(hosts.popover).flatten(),
        rises: page == Page::Devices && !session_modal,
        session_modal,
        shown,
        viewport,
    }
}

/// Whether `page` is a dialog that dims the window behind it, rather than a
/// popover beside what opened it.
pub fn dims(page: Page, session_modal: bool) -> bool {
    !footer_anchored(page, session_modal) && page != Page::Application && !page.pointer_anchored()
}

/// Whether `page` hangs above the window's foot as a panel of the footer that
/// opened it.
fn footer_anchored(page: Page, session_modal: bool) -> bool {
    matches!(page, Page::Menu | Page::Devices) && !session_modal
}

/// The room a footer menu has, and whether it opens above its anchor, as it
/// does from the sidebar footer at the window's foot.
pub(crate) fn footer_room(
    anchor: Point<Pixels>,
    viewport: Size<Pixels>,
    chrome: f32,
) -> (Pixels, bool) {
    let chrome = px(chrome);
    let band = (viewport.height - chrome - px(2. * MENU_MARGIN)).max(px(60.));
    let room = |side: Pixels| side.clamp(px(0.), band).max(px(60.)).min(band);
    let above = room(anchor.y - px(12. + MENU_MARGIN) - chrome);
    let below = room(viewport.height - anchor.y - px(12. + MENU_MARGIN));
    if above >= below {
        (above, true)
    } else {
        (below, false)
    }
}

/// Sets the family and its fallback cascade and nothing else: GPUI's own
/// `font` would also reset weight and style, which nested chrome inherits.
pub(crate) fn text_font<E: gpui::Styled>(mut element: E, text: &Text) -> E {
    let style = element.text_style();
    style.font_family = Some(text.font.family.clone());
    style.font_fallbacks = text.font.fallbacks.clone();
    element
}
