// ABOUTME: Projects saved native tabs into each window workspace.
// ABOUTME: Hides disabled Core views without deleting their stored references.
//! What the window itself does with the browser: the browser it holds, the
//! store and the scope its daemon files tabs under, and the ticks that keep
//! them in step with the daemon. The tabs and their groups are
//! `spaceport_browser`'s.
use super::Layout;
#[cfg(test)]
use crate::i18n::fl;
use crate::{SpaceportWindow, i18n::Localize, window::Flash};
use gpui::{prelude::*, *};
#[cfg(test)]
use spaceport_browser::Location;
use spaceport_browser::{Browser, Layouts, Pick, Scope, Store, TabId, WebUrl};
use std::collections::HashSet;

mod render;

/// A window's browser, restoring the layouts the app saved for each workspace
/// as the window first shows it. Built before the window itself, so the saved
/// layouts it reads come from the app's global rather than from a window.
pub(crate) fn new_browser(cx: &mut App) -> Browser {
    Browser::new(Layouts::snapshot(cx))
}

/// Names the daemon behind an endpoint the way browser tabs remember it.
pub(crate) fn scope(endpoint: &crate::endpoint::Endpoint) -> Scope {
    match endpoint.connection.target.socket_path() {
        Ok(path) => Scope::local(&path),
        Err(_) => Scope::endpoint(&endpoint.id),
    }
}

/// Page titles run long; a tab shows the start of one.
pub(crate) fn tab_label(title: &str) -> SharedString {
    spaceport_truncate::tab_title(title).into()
}

pub(crate) fn store(cx: &App) -> Option<&Store> {
    cx.try_global::<Store>()
}

/// Explains a rejected opening through the store's shared admission check.
pub(crate) fn tab_open_failure(cx: &App, capacity: impl Into<SharedString>) -> Flash {
    match store(cx).and_then(|store| store.check_new_tab_id().err()) {
        Some(error) => Flash::warning(error.localize()),
        None => Flash::warning(capacity),
    }
}

/// What [`SpaceportWindow::open_workspace_page`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PageOpened {
    /// In the system browser.
    SystemBrowser,
    /// Not opened: the window has no such host.
    Unreachable,
}

impl SpaceportWindow {
    pub(crate) fn browser_key(&self) -> Option<(Scope, String)> {
        self.browser.browser_key(&self.browser_hosts(None))
    }

    pub(crate) fn active_browser_key(&self, cx: &App) -> Option<(Scope, String)> {
        match self.selected_core_group() {
            Some(key) => self.core_group_available(key, cx).then(|| key.parts()),
            None => self.browser_key(),
        }
    }

    /// The focused workspace's browser tabs, in the order they opened.
    #[cfg(test)]
    pub(crate) fn browser_tab_ids(&self, cx: &App) -> Vec<TabId> {
        self.browser.browser_tab_ids(&self.browser_hosts(Some(cx)))
    }

    pub(super) fn forget_browser_tabs(&mut self, gone: impl FnMut(TabId) -> bool) {
        let (browser, _) = self.browser_parts(None);
        browser.forget_browser_tabs(gone);
    }

    /// Opens native content in the focused workspace for shared layout tests.
    #[cfg(test)]
    pub(crate) fn open_native_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((scope, workspace)) = self.browser_key() else {
            self.show_flash(Flash::warning(fl!("browser-open-workspace-first")), cx);
            return;
        };
        let location = Some(Location::Devices);
        match Store::update(cx, |store| store.open(scope, &workspace, location, None)) {
            Some(id) => {
                self.poll_devices_overview(cx);
                self.show_browser_tab(id, window, cx);
            }
            None => self.show_flash(tab_open_failure(cx, fl!("browser-too-many-tabs")), cx),
        }
    }

    /// Opens a service or repository URL in the system handler without changing
    /// the workspace's tabs. The endpoint must still name this window's host.
    pub(crate) fn open_workspace_page(
        &mut self,
        endpoint: &str,
        _workspace: &str,
        url: WebUrl,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> PageOpened {
        if !self.endpoints.iter().any(|entry| entry.id == endpoint) {
            return PageOpened::Unreachable;
        }
        cx.open_url(url.as_str());
        PageOpened::SystemBrowser
    }

    /// Drops tabs other windows closed and forgets the tabs of workspaces
    /// the daemon closed. Runs on every window tick.
    pub(crate) fn poll_browser(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(store) = store(cx) {
            let gone: HashSet<TabId> = self
                .browser
                .layouts
                .values()
                .flat_map(Layout::tabs)
                .filter_map(|pick| match pick {
                    Pick::Page(id) => Some(*id),
                    Pick::Herdr(_) => None,
                })
                .filter(|id| store.get(*id).is_none())
                .collect();
            if !gone.is_empty() {
                self.forget_browser_tabs(|id| gone.contains(&id));
                cx.notify();
            }
        }
        self.observe_closed_herdr_tabs(cx);
        self.forget_closed_workspaces(cx);
        self.settle_groups(window, cx);
        self.advance_close_run(window, cx);
    }

    fn forget_closed_workspaces(&mut self, cx: &mut Context<Self>) {
        let Some(snapshot) = self.live.snapshot.as_ref() else {
            return;
        };
        let scope = scope(&self.endpoints[self.selected_endpoint]);
        let unchanged = self
            .browser
            .workspaces
            .as_ref()
            .is_some_and(|(seen_scope, boot, seen)| {
                seen_scope == &scope
                    && boot == &snapshot.boot_id
                    && seen.len() == snapshot.workspaces.len()
                    && snapshot
                        .workspaces
                        .iter()
                        .all(|workspace| seen.contains(&workspace.workspace_id))
            });
        if unchanged {
            return;
        }
        let current: HashSet<String> = snapshot
            .workspaces
            .iter()
            .map(|workspace| workspace.workspace_id.clone())
            .collect();
        let previous = self.browser.workspaces.replace((
            scope.clone(),
            snapshot.boot_id.clone(),
            current.clone(),
        ));
        // A restarted daemon or another session proves nothing was closed.
        let Some((_, _, seen)) = previous
            .filter(|(seen_scope, boot, _)| seen_scope == &scope && boot == &snapshot.boot_id)
        else {
            return;
        };
        let closed: Vec<String> = seen.difference(&current).cloned().collect();
        if closed.is_empty() {
            return;
        }
        self.browser
            .layouts
            .retain(|(saved, workspace), _| saved != &scope || !closed.contains(workspace));
        self.browser.hidden.forget_workspaces(&scope, &closed);
        Layouts::forget_workspaces(cx, &scope, &closed);
        if !store(cx).is_some_and(|store| store.has_workspaces(&scope, &closed)) {
            return;
        }
        Store::update(cx, |store| store.forget_workspaces(&scope, &closed));
        cx.notify();
    }
}

#[cfg(test)]
mod tests;
