// ABOUTME: Captures Core split-pane closes for the shared confirmation dialog.
// ABOUTME: Rechecks the selected workspace, saved pane, and exact Core before closing.
use super::{CloseConfirmation, Target};
use crate::{
    SheprdWindow,
    browser::{Location, Scope, Store, TabId},
    config::preferences::Preference,
};
use gpui::{App, Context, FocusHandle, Window};
use sheprd_backend::TerminalId;

#[cfg(feature = "integration-test")]
mod native_checks;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Pane {
    owner: (Scope, String),
    tab: TabId,
    terminal: TerminalId,
    core_id: Option<String>,
}

impl SheprdWindow {
    /// Cancelling returns to the captured pane only while it still exists here.
    pub(crate) fn core_close_focus(&self, cx: &App) -> Option<FocusHandle> {
        let Target::Core(target) = &self.menu.close.as_ref()?.target else {
            return None;
        };
        if self.active_browser_key(cx).as_ref() != Some(&target.owner) {
            return None;
        }
        let tab = cx.try_global::<Store>()?.get(target.tab)?;
        if tab.scope != target.owner.0 || tab.workspace_id != target.owner.1 {
            return None;
        }
        if !matches!(&tab.location, Some(Location::CoreTerminal { panes, .. }) if panes.contains(&target.terminal))
        {
            return None;
        }
        self.core_pane_focus(target.tab, &target.terminal, cx)
    }

    pub(crate) fn open_core_pane_close(
        &mut self,
        tab: TabId,
        terminal: TerminalId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(shown) = cx.try_global::<Store>().and_then(|store| store.get(tab)) else {
            return;
        };
        let target = Pane {
            owner: (shown.scope.clone(), shown.workspace_id.clone()),
            tab,
            terminal: terminal.clone(),
            core_id: shown
                .core_group()
                .map(|group| group.hub_core_id().to_owned())
                .or_else(|| {
                    self.core_terminals
                        .hub
                        .backend()
                        .map(|backend| backend.core().hello().core.id.clone())
                }),
        };
        let close = CloseConfirmation {
            target: Target::Core(target),
            label: terminal.to_string(),
            interrupts: None,
            confirm_selected: false,
            do_not_ask_again: false,
            error: None,
        };
        self.show_close(Some(close), window, cx);
    }

    pub(super) fn send_core_pane_close(
        &mut self,
        target: Pane,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Preference> {
        let current = self.menu_current()
            && self.active_browser_key(cx).as_ref() == Some(&target.owner)
            && cx.try_global::<Store>().and_then(|store| store.get(target.tab)).is_some_and(|tab| {
                tab.scope == target.owner.0
                    && tab.workspace_id == target.owner.1
                    && matches!(&tab.location, Some(Location::CoreTerminal { panes, observe: false })
                        if panes.len() > 1 && panes.contains(&target.terminal))
            })
            && target.core_id.as_ref().is_some_and(|id| self.core_terminals.hub.backend_for(id).is_ok());
        if !current {
            if let Some(close) = &mut self.menu.close {
                close.error = Some(crate::Error::StaleCloseTarget.to_string());
            }
            cx.notify();
            return None;
        }
        if !self.remove_core_pane(target.tab, &target.terminal, window, cx) {
            return None;
        }
        let stop_asking = self
            .menu
            .close
            .as_ref()
            .is_some_and(|close| close.do_not_ask_again);
        self.end_core_terminal(target.terminal);
        self.dismiss_menu(window, cx);
        self.focus_core_terminal(target.tab, window, cx);
        stop_asking.then_some(Preference::ConfirmClosePane(false))
    }
}
