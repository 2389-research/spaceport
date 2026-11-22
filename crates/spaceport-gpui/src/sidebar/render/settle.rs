//! Pinning and revealing the sidebar's rows before the frame paints them.

use super::measure::SidebarMeasure;
use super::rows::SidebarRows;
use crate::SpaceportWindow;
use gpui::{prelude::*, *};

use super::super::sticky;

/// What settling leaves the frame: the header held over the spaces list.
pub(super) struct SidebarSettled {
    /// The copy of a host's header held over the top of the spaces list.
    pub(super) pinned_host: Option<Stateful<Div>>,
}

impl SpaceportWindow {
    /// Settles the frame's rows against what the last frame measured: which
    /// host header pins over the list, which rows must be revealed, and
    /// whether another frame is owed for a slide or a fold. These read and
    /// move scroll offsets, so they run after the rows are built and before
    /// anything is painted.
    pub(super) fn sidebar_settle(
        &self,
        m: &SidebarMeasure,
        rows: &SidebarRows,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SidebarSettled {
        // A header still sliding, or a device fold still measuring, owes the
        // frame after this one.
        if rows.sliding {
            window.request_animation_frame();
        }
        let mut headers = if m.headed {
            self.host_headers(&rows.host_rows)
        } else {
            Vec::new()
        };
        if let Some(row) = self.sidebar_pin_reveal.take()
            && self.uncover_revealed(row, &headers)
        {
            headers = self.host_headers(&rows.host_rows);
        }
        // Rows under the pinned header are hidden, so a reveal must clear it.
        let cover = px(sticky::cover(&headers));
        // Search results stand in for the spaces list, so no nested row paints.
        self.reveal_nested_agent(
            rows.nested_focus.clone().filter(|_| rows.search.is_none()),
            cover,
            window,
        );
        // Folded headers slide under a pinned one, so none is pinned then, nor
        // in the frame that unfolds, whose header places are the folded list's;
        // the frame after pins from the unfolded one.
        let unfolding =
            self.sidebar_folded.replace(m.device_plan.is_some()) && m.device_plan.is_none();
        if unfolding {
            window.request_animation_frame();
        }
        let pinned_host = sticky::pinned(&headers)
            .filter(|_| m.device_plan.is_none() && !unfolding)
            .and_then(|pinned| self.pinned_host_row(pinned, m.look, m.width, cx));
        // Follow the selection, but only once a frame has measured the
        // viewport: the handle resolves the request against the previous
        // frame's bounds, so an unmeasured list would scroll to a meaningless
        // offset. Recording what was revealed keeps later frames from undoing
        // the user's own scrolling.
        for (list, row) in rows.highlighted.iter().enumerate() {
            let Some(row) = *row else { continue };
            if self.sidebar_revealed[list].get() != Some(row)
                && self.sidebar_scroll[list].bounds().size.height > px(0.)
            {
                let scroll = &self.sidebar_scroll[list];
                let cover = if list == 0 { cover } else { px(0.) };
                let visible = scroll.bounds_for_item(row).is_some_and(|bounds| {
                    let offset = scroll.offset().y;
                    bounds.bottom() + offset > scroll.bounds().top() + cover
                        && bounds.top() + offset < scroll.bounds().bottom()
                });
                // Even a partially visible worktree is already seen. GPUI's
                // reveal also moves clipped rows, so only request it off-screen.
                if !visible {
                    scroll.scroll_to_item(row);
                    // The scroll lands in prepaint, after this render pinned a
                    // header for the old offset, and puts a row from above at
                    // the top edge, under the pinned header. Another frame pins
                    // for the new offset and moves the row out from under it.
                    if list == 0 && m.headed {
                        window.request_animation_frame();
                        self.sidebar_pin_reveal.set(Some(row));
                    }
                }
                self.sidebar_revealed[list].set(Some(row));
            }
        }
        SidebarSettled { pinned_host }
    }
}
