// ABOUTME: Builds visible native tab entries and their interaction callbacks.
// ABOUTME: Keeps saved Core tabs hidden while the app session disables Core.
//! Drawing browser tabs: their entries in a group's strip, and the page with
//! its toolbar where the terminal would be.

use super::{store, tab_label};
use crate::{
    SheprdWindow,
    browser::{
        Location, Tab, TabId,
        groups::{Pick, Slot},
    },
    tab_layout::{Mark, Probe, TabContext, TabFace},
};
use gpui::{prelude::*, *};

/// What a page's tab shows until the page reports an icon of its own.
const GLOBE: &str = "icons/globe.svg";

impl SheprdWindow {
    /// The workspace's browser tabs, after its Herdr tabs in a group's strip.
    pub(crate) fn browser_tab_entries(
        &self,
        slot: Slot,
        shown: Option<TabId>,
        faces: &TabContext<'_>,
        cx: &mut Context<Self>,
    ) -> Vec<(TabId, Stateful<Div>)> {
        let (Some((scope, workspace)), Some(store)) = (self.active_browser_key(cx), store(cx))
        else {
            return Vec::new();
        };
        let tabs: Vec<Tab> = store
            .in_workspace(&scope, &workspace)
            .filter(|tab| self.core_enabled() || !tab.requires_core())
            .filter(|tab| self.group_lists(slot.id, &Pick::Page(tab.id)))
            .cloned()
            .collect();
        let layout = self.tab_layout();
        tabs.into_iter()
            .map(|tab| {
                let id = tab.id;
                // Device tabs use their own marks rather than a page favicon.
                let icon = match &tab.location {
                    Some(Location::Devices) => "icons/devices.svg",
                    Some(Location::CoreTerminal { .. }) => "icons/terminal.svg",
                    _ => GLOBE,
                };
                let mark = Mark::Icon(icon);
                let base = div()
                    .id(SharedString::from(format!("browser-tab-{id}")))
                    .debug_selector(move || slot.selector(&format!("browser-tab-{id}")));
                let face = TabFace {
                    label: tab_label(&tab.label()),
                    mark,
                    zoomed: false,
                    muted: false,
                    probe: Probe::Page(id),
                    // Split, the page closes in its group alone.
                    close: Box::new(cx.listener(move |this, _, window, cx| {
                        this.close_strip_tab(Pick::Page(id), window, cx);
                    })),
                };
                let tab = layout
                    .tab(
                        base,
                        face,
                        self.tab_state(shown == Some(id), slot.id),
                        faces,
                    )
                    .map(|tab| self.grow_tab(tab, slot.id, &Pick::Page(id)))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.show_browser_tab_in(Some(slot.id), id, window, cx);
                    }));
                (id, tab)
            })
            .collect()
    }
}
