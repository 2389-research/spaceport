//! Sliding the sidebar out to the left when it is hidden and back in when it is
//! shown, within its own room: the terminal beside it is resized once, when the
//! sidebar is shown or once it has slid off, never as it moves. With the
//! activity bar, the bar stays and the panel beside it slides. How far the
//! sidebar shows and whether it still moves are `spaceport-sidebar`'s.

use crate::SpaceportWindow;
use crate::motion;
use gpui::{Context, Window};

impl SpaceportWindow {
    /// Moves the slide on for the frame being drawn. A sidebar shown or
    /// hidden some other way than a toggle, such as startup's collapsed
    /// setting, is simply there.
    pub(crate) fn tick_sidebar_slide(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let advance = self
            .sidebar_slide
            .advance(self.sidebar_visible, motion::now());
        // The sidebar is a cached view: it redraws only when told.
        if advance.redraw {
            self.sidebar_view.update(cx, |_, cx| cx.notify());
        }
        if advance.moving {
            window.request_animation_frame();
        }
    }

    /// How much of the sliding sidebar shows, `None` while it is still.
    pub(crate) fn sidebar_slide_shown(&self) -> Option<f32> {
        self.sidebar_slide_shown_at(motion::now())
    }

    /// [`Self::sidebar_slide_shown`] at `now`.
    fn sidebar_slide_shown_at(&self, now: std::time::Instant) -> Option<f32> {
        self.sidebar_slide.shown_at(now)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests;
