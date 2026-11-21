//! The window's part of a group's tab strip: the calls its drawing makes,
//! each handed to the browser the window holds. The strip's own state — the
//! tabs growing in and out, the sideways scroll, the sliding body — is
//! `sheprd_browser`'s, in [`sheprd_browser::Browser`].
use super::{GroupId, Leaving, Listed, Pick, Thumb};
use crate::SheprdWindow;
use gpui::{prelude::*, *};
use std::time::Instant;

impl SheprdWindow {
    pub(crate) fn observe_strip(&mut self, group: GroupId, tabs: Vec<Listed>, now: Instant) {
        let (browser, _) = self.browser_parts(None);
        browser.observe_strip(group, tabs, now);
    }

    pub(crate) fn leaving_tabs(&self, group: GroupId) -> Vec<Leaving> {
        self.browser.leaving_tabs(group)
    }

    pub(crate) fn forget_gone_strips(&mut self) {
        let (browser, _) = self.browser_parts(None);
        browser.forget_gone_strips();
    }

    pub(crate) fn strip_scroll(&mut self, group: GroupId) -> ScrollHandle {
        let (browser, _) = self.browser_parts(None);
        browser.strip_scroll(group)
    }

    pub(crate) fn reveal_tab(
        &mut self,
        group: GroupId,
        pick: &Pick,
        index: usize,
        width: f32,
    ) -> bool {
        let (browser, hosts) = self.browser_parts(None);
        browser.reveal_tab(group, pick, index, width, &hosts)
    }

    pub(crate) fn strip_thumb(&self, group: GroupId) -> Option<Thumb> {
        self.browser.strip_thumb(group)
    }

    pub(crate) fn grab_strip_thumb(&mut self, group: GroupId, x: Pixels) {
        let (browser, _) = self.browser_parts(None);
        browser.grab_strip_thumb(group, x);
    }

    pub(crate) fn drag_strip_thumb(&mut self, group: GroupId, x: Pixels) -> bool {
        let (browser, _) = self.browser_parts(None);
        browser.drag_strip_thumb(group, x)
    }

    pub(crate) fn tabs_growing(&self) -> bool {
        self.browser.tabs_growing()
    }

    #[cfg(test)]
    pub(crate) fn hold_group_motion(&mut self) {
        let (browser, _) = self.browser_parts(None);
        browser.hold_group_motion();
    }

    #[cfg(test)]
    pub(crate) fn settle_tabs(&mut self) {
        let (browser, _) = self.browser_parts(None);
        browser.settle_tabs();
    }

    pub(crate) fn grow_tab<E: Styled>(&self, tab: E, group: GroupId, pick: &Pick) -> E {
        self.browser
            .grow_tab(tab, group, pick, &self.browser_hosts(None))
    }
}
