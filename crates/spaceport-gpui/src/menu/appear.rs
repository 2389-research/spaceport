//! The menu layer coming and going. A popover fades in while it slides a few
//! pixels away from what opened it; a dialog rises into place while the
//! window dims behind it, and closing it lifts the dimming back off. Moving
//! between pages of one popup, such as Add Device's sources or a menu's
//! rename field, keeps it still: only a popup that is new to the eye moves.
//!
//! All Devices instead rises out of the sidebar footer that opened it and
//! sinks back into it when it closes. It is drawn again from what
//! [`kept::Kept`] took out before the reset as it goes.
//!
//! Which popup is shown and how far each motion has come is
//! [`spaceport_menu::appear`]'s, over a page and a pixel distance; what a
//! closing page reads, and the dimming a dialog lays over the window, stay
//! this window's.

// i18n: migrated
mod kept;

pub(super) use kept::Kept;
pub(super) use spaceport_menu::appear::{Shown, leaves};

use super::Page;
use crate::{SpaceportWindow, motion};
use gpui::{AnyElement, Context, Pixels, Point, Rgba, Window, div, prelude::*, px, rgba};
use spaceport_menu::appear::Appear as State;
use std::time::Instant;

/// How dark a dialog dims the window when fully shown, out of 255.
const DIM: f32 = 176.;

/// The dimming over the window behind a dialog, `shown` of the way in.
pub(super) fn backdrop(background: u32, shown: f32) -> Rgba {
    let alpha = (DIM * shown.clamp(0., 1.)).round() as u32;
    rgba((background << 8) | alpha)
}

/// A popover beside `anchor`, as far as its motion cares: one opened
/// somewhere else is new.
pub(super) fn popover(anchor: Point<Pixels>) -> Shown {
    Shown::Popover {
        x: anchor.x.into(),
        y: anchor.y.into(),
    }
}

/// How far a popup `shown` of the way in still has to go.
pub(super) fn offset(dims: bool, shown: f32) -> Pixels {
    px(spaceport_menu::appear::offset(dims, shown))
}

/// The window's menu layer, and the seam unit tests lay menus out by.
#[derive(Default)]
pub(crate) struct Appear {
    pub(in crate::menu) state: State<Kept>,
    /// Unit tests lay menus out where they settle unless one asks to see them
    /// move, so geometry checks need not wait out the motion.
    #[cfg(test)]
    live: bool,
}

impl Appear {
    /// Takes the popup this frame draws and the page it draws.
    fn observe(&mut self, shown: Option<Shown>, page: Option<Page>, now: Instant) {
        #[cfg(test)]
        let still = !self.live;
        #[cfg(not(test))]
        let still = false;
        self.state.observe(shown, page, now, still);
    }
}

impl SpaceportWindow {
    /// Takes the popup this frame draws, and keeps frames coming while it,
    /// or the dimming a closed dialog leaves, still moves.
    pub(crate) fn tick_menu_appear(&mut self, window: &mut Window) {
        let now = motion::now();
        // The application menu bar's menus draw their own way.
        let shown = self
            .menu
            .page
            .filter(|page| *page != Page::Application)
            .map(|_| match self.menu_dims() {
                true => Shown::Dialog,
                false => popover(self.menu.anchor),
            });
        self.menu.appear.observe(shown, self.menu.page, now);
        self.menu.appear.state.settle(now);
        if self.menu.appear.state.moving(now) {
            window.request_animation_frame();
        }
    }
}

impl SpaceportWindow {
    /// How much of a closed dialog's dimming is still lifting at `now`, from
    /// 0 to 1; 0 while a dialog that dims the window itself is open.
    pub(crate) fn menu_fading(&self, now: Instant) -> f32 {
        if self.menu_dims() {
            return 0.;
        }
        self.menu.appear.state.backdrop(now)
    }

    /// What a popup that just closed leaves while it goes: a dialog's
    /// dimming lifting, or All Devices sinking into the footer. Neither takes
    /// input: the window beneath is already the one being used.
    pub(crate) fn render_menu_leaving(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let now = motion::now();
        // Also behind a popover a dialog went back to, such as the session
        // picker under its editor: it draws no dimming of its own.
        let shown = self.menu_fading(now);
        let fade = (shown > 0.).then(|| {
            div()
                .id("menu-fade")
                .debug_selector(|| "menu-fade".into())
                .absolute()
                .inset_0()
                .bg(backdrop(self.theme.background, shown))
                .into_any_element()
        });
        let leaving = self
            .menu
            .appear
            .state
            .leaving(now)
            .filter(|_| self.menu.page.is_none());
        let page = leaving.map(|(page, left)| {
            // Drawn as it was: its own fields and page back in the menu for
            // the length of this one draw, which only reads them.
            let mut kept = self.menu.appear.state.kept.take();
            if let Some((_, kept)) = &mut kept {
                kept.swap(&mut self.menu);
            }
            self.menu.page = Some(page);
            let element = self
                .render_menu_layer(page, Some(left), window, cx)
                .into_any_element();
            self.menu.page = None;
            if let Some((_, kept)) = &mut kept {
                kept.swap(&mut self.menu);
            }
            self.menu.appear.state.kept = kept;
            element
        });
        fade.into_iter().chain(page).collect()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests;
