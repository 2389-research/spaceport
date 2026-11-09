//! Carrying a tab out of the window opens another view of its exact endpoint
//! or Core group. The tab's runtime stays where it is; only the two windows'
//! visible tab lists change. Closing the new window gives the tab back.

use super::{SheprdWindow, tab_drag::TabDrag};
use crate::{
    NavigationTarget,
    app::Placement,
    browser::{CoreGroupKey, Pick, Scope, Store},
    window_state::WindowState,
};
use gpui::{Bounds, Context, Subscription, WeakEntity, Window};
use sheprd_client::ConnectTarget;

/// A tab carried out of a window, and where the window that shows it opens.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TornTab {
    pub(super) route: TornRoute,
    pub(crate) pick: Pick,
    /// The workspace the tab belongs to, as the window it left keys it.
    pub(crate) workspace: (Scope, String),
    /// The workspace's other tabs when it was torn off, which the new
    /// window hides.
    pub(crate) others: Vec<Pick>,
    /// The window the tab left, which lists it again once the new one
    /// closes.
    pub(crate) from: WeakEntity<SheprdWindow>,
    pub(crate) placement: Placement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TornRoute {
    /// The document observation must still name this group when the new
    /// window accepts the tab; reconnecting makes an old handoff stale.
    Core { key: CoreGroupKey, epoch: u64 },
    Legacy {
        target: ConnectTarget,
        endpoint: String,
        session: ConnectTarget,
    },
}

/// The tabs torn off in a test, which opens no windows.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct TornOff(pub(crate) Vec<TornTab>);

#[cfg(test)]
impl gpui::Global for TornOff {}

impl SheprdWindow {
    /// Opens the tab `drag` carried out of the window in a new window,
    /// placed as if this one had been carried along with it.
    pub(super) fn tear_off_tab(
        &mut self,
        drag: &TabDrag,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(workspace) = self.active_browser_key(cx) else {
            return;
        };
        let route = if let Some(key) = self.selected_core_group() {
            let (Pick::Page(id), Some(epoch)) = (&drag.pick, Self::core_workspace_epoch(cx)) else {
                return;
            };
            if cx
                .try_global::<Store>()
                .and_then(|store| store.get(*id))
                .and_then(|tab| tab.core_group())
                .as_ref()
                != Some(key)
            {
                return;
            }
            TornRoute::Core {
                key: key.clone(),
                epoch,
            }
        } else {
            TornRoute::Legacy {
                target: self.endpoints[0].connection.target.clone(),
                endpoint: self.endpoints[self.selected_endpoint].id.clone(),
                session: self.endpoints[self.selected_endpoint]
                    .connection
                    .target
                    .clone(),
            }
        };
        let frame = window.bounds();
        let bounds = Bounds::new(frame.origin + (drag.pointer - drag.origin), frame.size);
        let torn = TornTab {
            route,
            pick: drag.pick.clone(),
            others: self
                .open_workspace_tabs(cx)
                .into_iter()
                .filter(|pick| *pick != drag.pick)
                .collect(),
            workspace: workspace.clone(),
            from: cx.entity().downgrade(),
            placement: WindowState::beside(bounds, window, cx),
        };
        self.leave_tab(&drag.pick, cx);
        // A tab the group could not move on from stays listed while shown.
        self.browser
            .hidden
            .hide(workspace.clone(), [drag.pick.clone()]);
        #[cfg(test)]
        cx.default_global::<TornOff>().0.push(torn);
        #[cfg(not(test))]
        {
            let (from, pick) = (torn.from.clone(), torn.pick.clone());
            let route = torn.route.clone();
            let placement = torn.placement;
            let failed = move |cx: &mut gpui::App| {
                // A window already closed has nothing to list it in.
                let _ = from.update(cx, |this, cx| this.untear(&workspace, &pick, cx));
            };
            match route {
                TornRoute::Legacy { target, .. } => crate::app::open_additional_window_then(
                    target,
                    Some(placement),
                    move |view, window, cx| {
                        view.arrive(torn, window, cx);
                    },
                    failed,
                    cx,
                ),
                TornRoute::Core { key, .. } => crate::app::open_additional_core_window_then(
                    key,
                    Some(placement),
                    move |view, window, cx| view.arrive(torn, window, cx),
                    failed,
                    cx,
                    #[cfg(feature = "integration-test")]
                    false,
                ),
            }
        }
    }

    /// Lists `pick` of `workspace` again when the window it was carried to
    /// could not open, since no window closing would give it back.
    pub(super) fn untear(
        &mut self,
        workspace: &(Scope, String),
        pick: &Pick,
        cx: &mut Context<Self>,
    ) {
        self.browser.hidden.show(workspace, pick);
        cx.notify();
    }

    /// Lists again, once a window closes, the tabs this one hid when they
    /// were carried off to it.
    pub(crate) fn give_back_torn_tabs(cx: &mut Context<Self>) -> Subscription {
        let this = cx.entity().downgrade();
        cx.on_window_closed(move |cx, closed| {
            let _ = this.update(cx, |this, cx| {
                if this.browser.hidden.release(closed) {
                    cx.notify();
                }
            });
        })
    }

    /// Accepts a Core tab only in the exact fresh group and observation
    /// epoch. A legacy host may need to wait until its catalog entry loads.
    pub(crate) fn arrive(
        &mut self,
        torn: TornTab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if let TornRoute::Core { key, epoch } = &torn.route {
            let valid = self.selected_core_group() == Some(key)
                && Self::core_workspace_epoch(cx) == Some(*epoch)
                && self.core_group_available(key, cx)
                && matches!(&torn.pick, Pick::Page(id) if cx.try_global::<Store>().and_then(|store| store.get(*id)).and_then(|tab| tab.core_group()).as_ref() == Some(key));
            if !valid {
                let _ = torn
                    .from
                    .update(cx, |from, cx| from.untear(&torn.workspace, &torn.pick, cx));
                window.remove_window();
                return false;
            }
            self.browser.saved.remove(&torn.workspace);
            if let Pick::Page(id) = &torn.pick {
                self.show_browser_tab(*id, window, cx);
            }
            self.browser
                .hidden
                .hide(torn.workspace.clone(), torn.others.iter().cloned());
            let holder = window.window_handle().window_id();
            let _ = torn.from.update(cx, |from, _| {
                from.browser
                    .hidden
                    .hold(&torn.workspace, &torn.pick, holder);
            });
            return true;
        }
        // The workspace's saved split would show the tabs this window
        // hides, so it is dropped before the page is shown in it.
        self.browser.saved.remove(&torn.workspace);
        if let Pick::Page(id) = torn.pick {
            self.show_browser_tab(id, window, cx);
        }
        let holder = window.window_handle().window_id();
        let _ = torn.from.update(cx, |from, _| {
            from.browser
                .hidden
                .hold(&torn.workspace, &torn.pick, holder);
        });
        self.arrival = Some(torn);
        self.land_arrival(false, cx);
        true
    }

    /// Goes where a waiting legacy tab is, once its host is listed:
    /// the session it showed there, then the tab, or a page's workspace,
    /// with the workspace's other tabs hidden and unsplit.
    /// The navigation itself waits for the surface. After the saved hosts
    /// are `listed`, a host still missing is gone, and the tab with it.
    pub(crate) fn land_arrival(&mut self, listed: bool, cx: &mut Context<Self>) {
        let Some(torn) = self.arrival.take() else {
            return;
        };
        let TornRoute::Legacy {
            endpoint, session, ..
        } = &torn.route
        else {
            return;
        };
        if !self
            .endpoints
            .iter()
            .any(|entry| entry.id == *endpoint && entry.enabled)
        {
            if !listed {
                self.arrival = Some(torn);
            }
            return;
        }
        self.browser
            .hidden
            .hide(torn.workspace.clone(), torn.others.iter().cloned());
        if let Some(session) = remote_session(session) {
            self.select_device_session(endpoint, session, cx);
        }
        let target = match &torn.pick {
            Pick::Herdr(tab) => NavigationTarget::Tab(tab.clone()),
            Pick::Page(id) => {
                let Some(tab) = cx.try_global::<Store>().and_then(|store| store.get(*id)) else {
                    return;
                };
                NavigationTarget::Workspace(tab.workspace_id.clone())
            }
        };
        self.navigate_endpoint(endpoint, target.as_deref(), cx);
    }
}

/// The session a remote host's endpoint is attached to, which another
/// window of the same host may not be.
pub(super) fn remote_session(target: &ConnectTarget) -> Option<&str> {
    match target {
        ConnectTarget::Ssh { session, .. } | ConnectTarget::Wsl { session, .. } => Some(session),
        _ => None,
    }
}
