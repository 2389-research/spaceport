//! The section list's and the body's scrollbars: the crate draws the thumbs,
//! this window holds the scroll state they read.
use super::SettingsWindow;
use gpui::{prelude::*, *};
pub(in crate::settings_window) use spaceport_settings_window::scrollbar::Pane;

impl SettingsWindow {
    fn pane_scroll(&self, pane: Pane) -> &ScrollHandle {
        match pane {
            Pane::Sections => &self.navigation_scroll,
            Pane::Body => &self.body_scroll,
        }
    }

    fn grab_thumb(&mut self, pane: Pane, y: Pixels) {
        self.thumb_grab = spaceport_settings_window::scrollbar::grab(self.pane_scroll(pane), y);
    }

    /// Moves `pane`'s thumb with the pointer at `y`; whether it scrolled.
    pub(super) fn drag_thumb(&mut self, pane: Pane, y: Pixels) -> bool {
        spaceport_settings_window::scrollbar::drag(self.pane_scroll(pane), self.thumb_grab, y)
    }

    /// `content`, which scrolls with `pane`'s handle, with the thumb over its
    /// trailing edge.
    pub(super) fn scroll_area(
        &self,
        pane: Pane,
        content: impl IntoElement,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let view = cx.entity().downgrade();
        let grabbing = view.clone();
        spaceport_settings_window::scrollbar::scroll_area(
            pane,
            &self.theme,
            self.pane_scroll(pane),
            content,
            move |pane, y, _, cx| {
                let _ = grabbing.update(cx, |this, _| this.grab_thumb(pane, y));
            },
            move |pane, y, _, cx| {
                let _ = view.update(cx, |this, cx| {
                    if this.drag_thumb(pane, y) {
                        cx.notify();
                    }
                });
            },
        )
    }
}
