//! What the browser may read of its window for one call.
//!
//! [`Browser`] owns everything a browser tab needs and changes it in place. The
//! window owns the endpoints, the shown snapshot, the tab store and the clock,
//! so every call that reads one of those takes [`BrowserHosts`] instead of the
//! window: it is the whole of the seam the browser has to the app.

use super::{CoreGroupKey, Scope, Store};
use crate::{SheprdWindow, endpoint::Endpoint};
use gpui::App;
use sheprd_client::protocol::ClientShellSnapshot;

/// What one window offers the browser for a single call. The app owns the
/// window and the store; the browser only reads them.
pub(crate) struct BrowserHosts<'a> {
    /// Whether this build shows herdr-core features: tabs that need it are
    /// left out of strips when false.
    pub(crate) core_enabled: bool,
    /// The herdr-core group this window shows, or none for a legacy workspace.
    pub(crate) core_group: Option<&'a CoreGroupKey>,
    /// Every endpoint this window knows, in `SheprdWindow.endpoints` order.
    pub(crate) endpoints: &'a [Endpoint],
    /// Which of `endpoints` this window shows,
    /// `SheprdWindow.selected_endpoint`.
    pub(crate) selected_endpoint: usize,
    /// The newest metadata of the shown endpoint, or none before it connects.
    pub(crate) snapshot: Option<&'a ClientShellSnapshot>,
    /// The window's tab store, read for a caller with a window context; none
    /// without one, and none while no store is installed.
    pub(crate) tabs: Option<&'a Store>,
    /// The workspace this window shows browser tabs for, or none: no workspace
    /// is focused, a herdr-core group is shown instead, or the caller had no
    /// window context.
    pub(crate) workspace: Option<(Scope, String)>,
}

impl SheprdWindow {
    /// What one call to the browser may read of this window. `cx` is the
    /// window context of the caller, which only names the store and the
    /// workspace shown; a call without one reads neither.
    pub(super) fn browser_hosts<'a>(&'a self, cx: Option<&'a App>) -> BrowserHosts<'a> {
        BrowserHosts {
            core_enabled: self.core_enabled(),
            core_group: self.selected_core_group(),
            endpoints: &self.endpoints,
            selected_endpoint: self.selected_endpoint,
            snapshot: self.live.snapshot.as_deref(),
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
    ) -> (&'a mut super::Browser, BrowserHosts<'a>) {
        let selected_endpoint = self.selected_endpoint;
        let workspace = cx.and_then(|cx| self.active_browser_key(cx));
        let tabs = cx.and_then(super::view::store);
        let SheprdWindow {
            browser,
            core_terminals,
            endpoints,
            live,
            ..
        } = self;
        let hosts = BrowserHosts {
            core_enabled: core_terminals.hub.enabled(),
            core_group: core_terminals.selected_group(),
            endpoints,
            selected_endpoint,
            snapshot: live.snapshot.as_deref(),
            tabs,
            workspace,
        };
        (browser, hosts)
    }
}
