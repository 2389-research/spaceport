// ABOUTME: Reverses popup entry and departure from each panel's current visibility.
// ABOUTME: Paints closing content without retaining active popup owners.
//! The menu layer coming and going. A popover fades in while it slides a few
//! pixels away from what opened it; a dialog rises into place while the
//! window dims behind it, and closing it lifts the dimming back off. Moving
//! between pages of one popup, such as Add Device's sources or a menu's
//! rename field, keeps it still: only a popup that is new to the eye moves.
//!
//! All Devices instead rises out of the sidebar footer that opened it and
//! sinks back into it when it closes. It is drawn again from what
//! [`kept::Kept`] took out before the reset as it goes.

mod kept;

pub(super) use kept::Kept;

// i18n: migrated
use super::Page;
#[cfg(test)]
use crate::motion::{ENTER, LEAVE};
use crate::{
    SheprdWindow,
    motion::{self, Toggle},
};
use gpui::{AnyElement, Context, Pixels, Point, Rgba, Window, div, prelude::*, px, rgba};
use std::time::Instant;

/// How far a popover slides toward its place as it appears.
const POPOVER_SLIDE: f32 = 6.;
/// How far a dialog rises into place as it appears.
const DIALOG_RISE: f32 = 10.;

/// How dark a dialog dims the window when fully shown, out of 255.
const DIM: f32 = 176.;

/// The dimming over the window behind a dialog, `shown` of the way in.
pub(super) fn backdrop(background: u32, shown: f32) -> Rgba {
    let alpha = (DIM * shown.clamp(0., 1.)).round() as u32;
    rgba((background << 8) | alpha)
}

/// What kind of popup the window shows, as far as its motion cares.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Shown {
    /// A popover beside what opened it; one opened somewhere else is new.
    Popover(Point<Pixels>),
    /// A dialog over the dimmed window.
    Dialog,
}

struct Leaving {
    page: Page,
    shape: Shown,
    motion: Toggle,
}

#[derive(Default)]
pub(crate) struct Appear {
    /// What the last frame drew, `None` while no popup was open.
    shown: Option<Shown>,
    /// The page the last frame drew.
    page: Option<Page>,
    /// The open popup’s reversible entry and departure.
    panel_motion: Toggle,
    /// A page going away after it closed, from when it did.
    leaving: Option<Leaving>,
    /// What the page that closed reads, taken out as it closed.
    kept: Option<(Page, Kept)>,
    /// All Devices' height as last laid out, which its rise travels.
    pub(super) rise_height: std::rc::Rc<std::cell::Cell<Option<Pixels>>>,
    backdrop: Toggle,
    /// Unit tests lay menus out where they settle unless one asks to see
    /// them move, so geometry checks need not wait out the motion.
    #[cfg(test)]
    pub(crate) live: bool,
}

impl Appear {
    fn observe(&mut self, shown: Option<Shown>, page: Option<Page>, now: Instant) {
        let last = std::mem::replace(&mut self.page, page);
        if shown == self.shown {
            return;
        }
        #[cfg(test)]
        let still = !self.live;
        #[cfg(not(test))]
        let still = false;
        // All Devices stays drawn beneath a session form it opened, so
        // closing the form uncovers it where it was rather than showing it
        // anew; only the dimming lifts.
        let uncovered = page == Some(Page::Devices)
            && last == page
            && self.shown == Some(Shown::Dialog)
            && matches!(shown, Some(Shown::Popover(_)));
        if let Some(shape) = shown {
            let reversing = self
                .leaving
                .as_ref()
                .is_some_and(|leaving| Some(leaving.page) == page && leaving.shape == shape);
            if reversing {
                if let Some(leaving) = self.leaving.take() {
                    self.panel_motion = leaving.motion;
                }
            } else {
                self.panel_motion.set(uncovered || still, now, true);
            }
            self.panel_motion.set(true, now, uncovered || still);
            self.leaving = None;
            self.kept = None;
        } else {
            let motion = self.panel_motion;
            self.leaving = match (last, self.shown) {
                (Some(page), Some(shape)) if !still => {
                    let mut motion = motion;
                    motion.set(false, now, false);
                    Some(Leaving {
                        page,
                        shape,
                        motion,
                    })
                }
                _ => None,
            };
            if self.leaving.as_ref().map(|leaving| leaving.page)
                != self.kept.as_ref().map(|(page, _)| *page)
            {
                self.kept = None;
            }
            self.panel_motion.set(false, now, still);
        }
        self.backdrop.set(shown == Some(Shown::Dialog), now, still);
        self.shown = shown;
    }

    /// How much of the open popup shows at `now`, from 0 to 1.
    pub(super) fn panel(&self, now: Instant) -> f32 {
        self.panel_motion.shown(now)
    }

    /// How far a popup `shown` of the way in still has to go: a popover
    /// toward its place beside its anchor, a dialog up into the middle.
    pub(super) fn offset(dims: bool, shown: f32) -> Pixels {
        let distance = if dims { DIALOG_RISE } else { POPOVER_SLIDE };
        px((1. - shown) * distance)
    }

    /// Keeps what the closing `page` reads, for it to be drawn as it goes.
    pub(super) fn keep(&mut self, page: Page, kept: Kept) {
        self.kept = Some((page, kept));
    }

    /// How much of the dimming behind a dialog shows at `now`, from 0 to 1.
    /// It keeps fading for a moment after the dialog has gone.
    pub(super) fn backdrop(&self, now: Instant) -> f32 {
        self.backdrop.shown(now)
    }

    /// The page sinking away after it closed, and how much of it is left.
    pub(super) fn leaving(&self, now: Instant) -> Option<(Page, f32)> {
        let leaving = self.leaving.as_ref()?;
        let left = leaving.motion.shown(now);
        (left > 0.).then_some((leaving.page, left))
    }

    /// Lets go of a closed page once it has gone, with what it read.
    fn settle(&mut self, now: Instant) {
        if self.leaving(now).is_none() {
            self.leaving = None;
            self.kept = None;
        }
    }

    fn moving(&self, now: Instant) -> bool {
        (self.shown.is_some() && self.panel_motion.moving(now))
            || self.backdrop.moving(now)
            || self.leaving(now).is_some()
    }
}

impl SheprdWindow {
    /// Takes the popup this frame draws, and keeps frames coming while it,
    /// or the dimming a closed dialog leaves, still moves.
    pub(crate) fn tick_menu_appear(&mut self, window: &mut Window) {
        let now = motion::now();
        let shown = self.menu.page.map(|_| match self.menu_dims() {
            true => Shown::Dialog,
            false => Shown::Popover(self.menu.anchor),
        });
        self.menu.appear.observe(shown, self.menu.page, now);
        self.menu.appear.settle(now);
        if self.menu.appear.moving(now) {
            window.request_animation_frame();
        }
    }
}

impl SheprdWindow {
    pub(crate) fn keep_application_menu_paint(&mut self, paint: crate::menus::in_window::Paint) {
        let kept = Kept {
            application: Some(paint),
            ..Kept::default()
        };
        self.menu.appear.keep(Page::Application, kept);
    }

    /// How much of a closed dialog's dimming is still lifting at `now`, from
    /// 0 to 1; 0 while a dialog that dims the window itself is open.
    pub(crate) fn menu_fading(&self, now: Instant) -> f32 {
        if self.menu_dims() {
            return 0.;
        }
        self.menu.appear.backdrop(now)
    }

    /// Draws captured panels and lifting dimming after the active menu closes.
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
            .leaving(now)
            .filter(|_| self.menu.page.is_none());
        let underlay = leaving.and_then(|(page, left)| {
            (page == Page::Devices)
                .then(|| self.menu.appear.kept.as_ref()?.1.devices.as_ref())
                .flatten()
                .filter(|paint| paint.session_modal())
                .map(|paint| {
                    self.render_session_picker_underlay(Some(paint), window, cx)
                        .opacity(left)
                        .into_any_element()
                })
        });
        let page = leaving.map(|(page, left)| {
            // Passive templates use captured values and the explicit page, without restoring active state.
            let kept = self.menu.appear.kept.take();
            let element = if page == Page::Application {
                kept.as_ref()
                    .and_then(|(_, kept)| kept.application.as_ref())
                    .map(|paint| {
                        self.render_application_menu_layer(
                            paint,
                            false,
                            (left, Appear::offset(false, left)),
                            window,
                            cx,
                        )
                        .into_any_element()
                    })
                    .unwrap_or_else(|| div().into_any_element())
            } else {
                self.render_menu_layer(
                    page,
                    Some(left),
                    kept.as_ref().map(|(_, kept)| kept),
                    window,
                    cx,
                )
                .into_any_element()
            };
            self.menu.appear.kept = kept;
            element
        });
        underlay.into_iter().chain(fade).chain(page).collect()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests;
