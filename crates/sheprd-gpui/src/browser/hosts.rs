//! What the browser may read of its window for one call.
//!
//! [`sheprd_browser::Browser`] owns everything a browser tab needs and changes
//! it in place. The window owns the endpoints, the shown snapshot, the tab
//! store and the clock, so every call that reads one of those takes a
//! `BrowserHosts` instead of the window: it is the whole of the seam the
//! browser has to the app, and building it here is the window's part.

use super::scope;
use super::{Browser, BrowserHosts};
use crate::{SheprdWindow, motion};
use gpui::App;

impl SheprdWindow {
    /// What one call to the browser may read of this window. `cx` is the
    /// window context of the caller, which only names the store and the
    /// workspace shown; a call without one reads neither.
    pub(super) fn browser_hosts<'a>(&'a self, cx: Option<&'a App>) -> BrowserHosts<'a> {
        BrowserHosts {
            core_enabled: self.core_enabled(),
            core_group: self.selected_core_group(),
            now: motion::now(),
            scope: self.endpoints.get(self.selected_endpoint).map(scope),
            snapshot: self.live.snapshot.as_deref(),
            tab_min_width: self.tab_layout().min_width(),
            tabs: cx.and_then(super::view::store),
            workspace: cx.and_then(|cx| self.active_browser_key(cx)),
        }
    }

    /// The window's browser and what it may read of the window in one call.
    /// Both are taken field by field, so the borrow checker knows the browser
    /// and the hosts are different parts of the window.
    pub(super) fn browser_parts<'a>(
        &'a mut self,
        cx: Option<&'a App>,
    ) -> (&'a mut Browser, BrowserHosts<'a>) {
        let now = motion::now();
        let scope = self.endpoints.get(self.selected_endpoint).map(scope);
        let tab_min_width = self.tab_layout().min_width();
        let workspace = cx.and_then(|cx| self.active_browser_key(cx));
        let tabs = cx.and_then(super::view::store);
        let SheprdWindow {
            browser,
            core_terminals,
            live,
            ..
        } = self;
        let hosts = BrowserHosts {
            core_enabled: core_terminals.hub.enabled(),
            core_group: core_terminals.selected_group(),
            now,
            scope,
            snapshot: live.snapshot.as_deref(),
            tab_min_width,
            tabs,
            workspace,
        };
        (browser, hosts)
    }
}
