// ABOUTME: Paints Core close consent without enabling or joining a runtime.
// ABOUTME: Owns one synthetic tab for the isolated native confirmation check.
use super::*;
use crate::{
    core_terminals::panes::{Axis, Panes},
    menu::Page,
};

const TERMINAL: &str = "native-close-confirmation-fixture";

impl SheprdWindow {
    pub(crate) fn prepare_core_close_confirmation_check(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<TabId> {
        if self.core_enabled() || self.core_terminals.hub.backend().is_some() {
            return None;
        }
        let (scope, workspace) = self.active_browser_key(cx)?;
        let terminal = TerminalId::new(TERMINAL).ok()?;
        let mut panes = Panes::single(terminal.clone());
        panes
            .split(
                &terminal,
                Axis::Row,
                TerminalId::new("native-close-neighbor-fixture").ok()?,
            )
            .ok()?;
        let tab = Store::update(cx, |store| {
            store.open(
                scope,
                &workspace,
                Some(Location::CoreTerminal {
                    panes,
                    observe: false,
                }),
                None,
            )
        })?;
        self.open_core_pane_close(tab, terminal, window, cx);
        Some(tab)
    }

    pub(crate) fn core_close_confirmation_untouched(&self, tab: TabId) -> bool {
        self.menu.page == Some(Page::ConfirmClose)
            && self.menu.close.as_ref().is_some_and(|close| close.error.is_none()
                && matches!(&close.target, Target::Core(target) if target.tab == tab && target.terminal.as_str() == TERMINAL))
    }

    pub(crate) fn remove_core_close_confirmation_check(&self, tab: TabId, cx: &mut App) -> bool {
        Store::update(cx, |store| {
            if !store.get(tab).is_some_and(|tab| {
                matches!(&tab.location,
                Some(Location::CoreTerminal {panes, ..}) if panes.first().as_str() == TERMINAL)
            }) {
                return false;
            }
            store.close(tab).is_some()
        })
    }
}
