//! The sidebar sliding out when it is hidden and back in when it is shown: how
//! far it shows, and whether it is still moving. A sidebar that collapses to
//! the rail slides off before the rail takes its place.
//!
//! The terminal beside it is resized once, when the sidebar is shown or once it
//! has slid off, never as it moves; that, and laying the window out for the
//! frame, stay in the app.

use spaceport_motion::{Toggle, now};
use std::time::Instant;

/// What one frame of the slide comes to.
pub struct Advance {
    pub redraw: bool,
    pub moving: bool,
}

/// The sidebar's slide, from where it is toward where it is wanted.
pub struct SidebarSlide {
    toggle: Toggle,
    /// Set for the frame being drawn: the sidebar is hidden but still
    /// sliding off, so the window keeps its expanded layout.
    closing: bool,
    /// Whether the last frame drew it moving, so the frame after it ends
    /// lays the sidebar out settled.
    moving: bool,
    /// Unit tests lay the sidebar out where a toggle leaves it unless one
    /// asks to see it move.
    #[cfg(any(test, feature = "test-support"))]
    pub live: bool,
}

impl Default for SidebarSlide {
    fn default() -> Self {
        let mut toggle = Toggle::default();
        toggle.set(true, now(), true);
        Self {
            toggle,
            closing: false,
            moving: false,
            #[cfg(any(test, feature = "test-support"))]
            live: false,
        }
    }
}

impl SidebarSlide {
    /// Starts the sidebar sliding toward `visible` at `now`.
    pub fn set(&mut self, visible: bool, at: Instant) {
        #[cfg(any(test, feature = "test-support"))]
        let still = !self.live;
        #[cfg(not(any(test, feature = "test-support")))]
        let still = false;
        self.toggle.set(visible, at, still);
    }

    /// Moves the slide on to `now`, toward `visible`, and says whether the
    /// sidebar has to redraw for it and whether it still moves.
    pub fn advance(&mut self, visible: bool, at: Instant) -> Advance {
        if self.toggle.is_open() != visible {
            self.toggle.set(visible, at, true);
        }
        let moving = self.toggle.moving(at);
        self.closing = moving && !self.toggle.is_open();
        // Also on the frame that settles it.
        let redraw = moving || self.moving;
        self.moving = moving;
        Advance { redraw, moving }
    }

    /// Whether the window keeps the expanded sidebar's layout for the frame
    /// being drawn although the sidebar is hidden.
    pub fn closing(&self) -> bool {
        self.closing
    }

    /// Leaves it settled where it is open or closed, as a sidebar first drawn
    /// is: what a toggle ends at once the slide has run.
    #[cfg(any(test, feature = "test-support"))]
    pub fn settle(&mut self) {
        self.toggle.set(self.toggle.is_open(), now(), true);
    }

    /// How much of the sliding sidebar shows at `now`, `None` while it is
    /// still.
    pub fn shown_at(&self, at: Instant) -> Option<f32> {
        let toggle = &self.toggle;
        toggle.moving(at).then(|| toggle.shown(at))
    }
}
