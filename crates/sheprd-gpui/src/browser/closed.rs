//! The window's part of reopening recently closed tabs: remembering a Herdr
//! tab's close, noticing the ones the daemon closed on its own, and reopening
//! either kind in the workspace it came from. [`super::ClosedTabs`] is the
//! store's.
use super::{
    ClosedHerdrTab, ClosedTab, Pick, Store,
    view::{scope, store},
};
use crate::i18n::fl;
use crate::{SheprdWindow, window::Flash};
use gpui::{Context, Window};
use sheprd_client::Method;
use std::collections::HashSet;

impl SheprdWindow {
    /// Remembers a Herdr tab whose close was just sent to the daemon so it can
    /// be reopened right away and in the order it was closed.
    pub(crate) fn record_closed_herdr_tab(&mut self, tab: ClosedHerdrTab, cx: &mut Context<Self>) {
        let scope = scope(&self.endpoints[self.selected_endpoint]);
        if let Some((seen_scope, _, _, tabs)) = &mut self.browser.herdr_tabs
            && *seen_scope == scope
        {
            tabs.retain(|seen| seen.tab_id != tab.tab_id);
        }
        Store::update(cx, |store| store.record_closed_herdr_tab(scope, tab));
    }

    /// Notices Herdr tabs that disappeared from the selected daemon's snapshot
    /// while their workspace stayed open, such as when a shell exits.
    pub(super) fn observe_closed_herdr_tabs(&mut self, cx: &mut Context<Self>) {
        let Some(snapshot) = self.live.snapshot.as_ref() else {
            return;
        };
        let scope = scope(&self.endpoints[self.selected_endpoint]);
        let unchanged =
            self.browser
                .herdr_tabs
                .as_ref()
                .is_some_and(|(seen_scope, boot, revision, _)| {
                    seen_scope == &scope
                        && boot == &snapshot.boot_id
                        && *revision == snapshot.revision
                });
        if unchanged {
            return;
        }
        let current: Vec<ClosedHerdrTab> = snapshot
            .tabs
            .iter()
            .map(|tab| ClosedHerdrTab::from_tab(snapshot, tab))
            .collect();
        let previous = self.browser.herdr_tabs.replace((
            scope.clone(),
            snapshot.boot_id.clone(),
            snapshot.revision,
            current,
        ));
        let Some((_, _, _, seen)) = previous
            .filter(|(seen_scope, boot, _, _)| seen_scope == &scope && boot == &snapshot.boot_id)
        else {
            return;
        };
        let open_tabs: HashSet<&str> = snapshot
            .tabs
            .iter()
            .map(|tab| tab.tab_id.as_str())
            .collect();
        let open_workspaces: HashSet<&str> = snapshot
            .workspaces
            .iter()
            .map(|workspace| workspace.workspace_id.as_str())
            .collect();
        let closed: Vec<ClosedHerdrTab> = seen
            .into_iter()
            .filter(|tab| {
                !open_tabs.contains(tab.tab_id.as_str())
                    && open_workspaces.contains(tab.workspace_id.as_str())
            })
            .collect();
        if closed.is_empty() {
            return;
        }
        Store::update(cx, |store| {
            for tab in closed {
                store.record_closed_herdr_tab(scope.clone(), tab);
            }
        });
    }

    /// Reopens the most recently closed tab of the focused workspace, whether
    /// it was a browser tab or a Herdr terminal tab.
    pub(crate) fn reopen_closed_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((scope, workspace)) = self.active_browser_key(cx) else {
            return;
        };
        let Some(closed) = store(cx)
            .and_then(|store| store.peek_closed(&scope, &workspace))
            .cloned()
        else {
            return;
        };
        match closed {
            ClosedTab::Page(_) => {
                let Some(ClosedTab::Page(page)) =
                    Store::update(cx, |store| store.pop_closed(&scope, &workspace))
                else {
                    return;
                };
                let Some(id) = Store::update(cx, |store| store.reopen_page(page.clone())) else {
                    Store::update(cx, |store| store.record_closed_page(page));
                    self.show_flash(Flash::warning(fl!("browser-too-many-tabs")), cx);
                    return;
                };
                self.poll_devices_overview(cx);
                match self.ensure_layout().map(|layout| layout.active()) {
                    Some(group) => self.move_tab_to_group(Pick::Page(id), group, window, cx),
                    None => self.show_browser_tab(id, window, cx),
                }
                if let Some(page) = self.devices_overview.pages.get(&id) {
                    window.focus(&page.focus, cx);
                }
            }
            ClosedTab::Herdr(_) => {
                if self.live.activation.fenced()
                    || !self.endpoints[self.selected_endpoint].surface_requested()
                    || self.endpoints[self.selected_endpoint]
                        .connection
                        .handle
                        .is_none()
                    || !self.live.snapshot.as_ref().is_some_and(|snapshot| {
                        snapshot
                            .workspaces
                            .iter()
                            .any(|candidate| candidate.workspace_id == workspace)
                    })
                {
                    return;
                }
                let Some(ClosedTab::Herdr(tab)) =
                    Store::update(cx, |store| store.pop_closed(&scope, &workspace))
                else {
                    return;
                };
                let previous_group = self.browser.new_tab_group.clone();
                if let Some(group) = self.active_group() {
                    self.expect_new_tab_in(group);
                }
                let params = tab.create_params();
                let sent =
                    self.request_focus_change(Method::TabCreate.as_str(), None, |handle, boot| {
                        handle.request(boot, Method::TabCreate, params)
                    });
                if !sent {
                    self.browser.new_tab_group = previous_group;
                    Store::update(cx, |store| store.restore_closed_herdr_tab(scope, tab));
                    return;
                }
                self.discard_composition(cx);
                window.focus(&self.focus, cx);
                cx.notify();
            }
        }
    }
}
