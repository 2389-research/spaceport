//! What a group's tab strip holds: the tabs growing into it and shrinking out
//! of it, its sideways scroll and thumb, and the body sliding under a
//! connected strip. The rules live in [`super::tab_appear`] and
//! [`super::tab_scroll`]; this applies them to this window's groups.
use super::{
    Browser, GroupId, Pick, Thumb,
    hosts::BrowserHosts,
    tab_appear::{Leaving, Listed},
};
use gpui::{Pixels, ScrollHandle, Styled, px};
use std::time::Instant;

impl Browser {
    /// Records what `group`'s strip draws; see
    /// [`super::tab_appear::TabAppear::observe`].
    pub fn observe_strip(&mut self, group: GroupId, tabs: Vec<Listed>, now: Instant) {
        self.appear.observe(group, tabs, now);
    }

    /// The tabs shrinking out of `group`'s strip.
    pub fn leaving_tabs(&self, group: GroupId) -> Vec<Leaving> {
        self.appear.leaving(group).to_vec()
    }

    /// Forgets the strips of groups that closed, so their tabs never shrink
    /// out of a strip no longer drawn.
    pub fn forget_gone_strips(&mut self) {
        let layouts = &self.layouts;
        let live = |group| {
            layouts
                .values()
                .any(|layout| layout.slots().any(|slot| slot.id == group))
        };
        self.appear.retain(live);
        self.tab_body.retain(live);
        self.group_bounds
            .borrow_mut()
            .retain(|group, _| live(*group));
        self.emptied.retain(|group| live(*group));
        // The group drawn before any workspace has a layout keeps its strip.
        let fallback = self.fallback_group;
        self.tab_scroll
            .retain(|group| group == fallback || live(group));
    }

    /// What tracks `group`'s strip's sideways scroll.
    pub fn strip_scroll(&mut self, group: GroupId) -> ScrollHandle {
        self.tab_scroll.handle(group)
    }

    /// Brings `group`'s chosen tab, at `index` in its strip, into view when
    /// the choice changes or while the tab grows in. Whether the strip needs
    /// another frame to do it.
    pub fn reveal_tab(
        &mut self,
        group: GroupId,
        pick: &Pick,
        index: usize,
        width: f32,
        hosts: &BrowserHosts<'_>,
    ) -> bool {
        let growing = self.appear.growth(group, pick, hosts.now).is_some();
        self.tab_scroll.reveal(group, pick, index, width, growing)
    }

    /// The thumb `group`'s strip draws, when its tabs outgrow it.
    pub fn strip_thumb(&self, group: GroupId) -> Option<Thumb> {
        self.tab_scroll.thumb(group)
    }

    /// Takes hold of `group`'s strip thumb with the pointer at `x`.
    pub fn grab_strip_thumb(&mut self, group: GroupId, x: Pixels) {
        self.tab_scroll.grab(group, x);
    }

    /// Drags `group`'s strip thumb to the pointer at `x`; whether it moved.
    pub fn drag_strip_thumb(&mut self, group: GroupId, x: Pixels) -> bool {
        self.tab_scroll.drag(group, x)
    }

    /// Whether a tab is still growing or shrinking, so the window draws
    /// another frame.
    pub fn tabs_growing(&self) -> bool {
        self.appear.animating()
    }

    /// Makes groups open and fold, held where their motion starts.
    #[cfg(any(test, feature = "test-support"))]
    pub fn hold_group_motion(&mut self) {
        self.group_motion.enable();
        self.group_motion.freeze(std::time::Duration::ZERO);
    }

    /// Ends every tab's growing and shrinking at once, as a moment later.
    #[cfg(any(test, feature = "test-support"))]
    pub fn settle_tabs(&mut self) {
        self.appear = Default::default();
    }

    /// Narrows and fades `tab` while it grows into the strip; a whole tab
    /// is left as it is.
    pub fn grow_tab<E: Styled>(
        &self,
        tab: E,
        group: GroupId,
        pick: &Pick,
        hosts: &BrowserHosts<'_>,
    ) -> E {
        /// Wider than any label a strip shows, so the last frame of growth
        /// does not visibly clamp a long one.
        const GROWN: f32 = 480.;
        match self.appear.growth(group, pick, hosts.now) {
            Some(k) => tab
                .min_w(px(hosts.tab_min_width * k))
                .max_w(px(GROWN * k))
                .overflow_hidden()
                .opacity(k),
            None => tab,
        }
    }
}
