// ABOUTME: Shares close consent and preferences across Herdr and Core panes.
// ABOUTME: Keeps captured targets fenced until a fresh confirmation.
// i18n: migrated
use crate::i18n::fl;
use crate::{
    Error, Result, SheprdWindow,
    config::preferences::Preference,
    controls::{self, Command},
    menu::Page,
};
use gpui::{prelude::*, *};
use serde_json::{Value, json};
use sheprd_client::{
    Method,
    protocol::{AgentStatus, ClientShellSnapshot},
};

mod batch;
mod core_pane;
mod sequence;
mod workspace;

pub(crate) use sequence::CloseRun;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CloseConfirmation {
    target: Target,
    label: String,
    /// The agents closing it would interrupt, as the dialog names them.
    interrupts: Option<String>,
    confirm_selected: bool,
    /// Pane closes only: confirming also saves `confirm_close_pane = false`.
    do_not_ask_again: bool,
    error: Option<String>,
}

pub(crate) struct Paint {
    title: String,
    note: String,
    confirm: String,
    label: String,
    interrupts: Option<String>,
    confirm_selected: bool,
    do_not_ask_again: bool,
    offers_do_not_ask_again: bool,
    error: Option<String>,
    long_labels: bool,
    offset: Point<Pixels>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Target {
    // keep-sorted start
    Batch(batch::Batch),
    Core(core_pane::Pane),
    Herdr(HerdrTarget),
    Workspace(workspace::Target),
    // keep-sorted end
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HerdrTarget {
    boot: String,
    workspace: String,
    tab: String,
    pane: Option<String>,
}

impl CloseConfirmation {
    pub(super) fn capture_pane(snapshot: &ClientShellSnapshot, id: &str) -> Option<Self> {
        let pane = snapshot.panes.iter().find(|pane| pane.pane_id == id)?;
        let mut close = Self::capture_tab(snapshot, &pane.tab_id)?;
        let Target::Herdr(target) = &mut close.target else {
            return None;
        };
        if target.workspace != pane.workspace_id {
            return None;
        }
        target.pane = Some(pane.pane_id.clone());
        close.label = pane.label.clone().unwrap_or_else(|| pane.pane_id.clone());
        close.interrupts = close.interrupted(snapshot);
        close.request(snapshot).ok()?;
        Some(close)
    }

    pub(super) fn capture_tab(snapshot: &ClientShellSnapshot, id: &str) -> Option<Self> {
        let tab = snapshot.tabs.iter().find(|tab| tab.tab_id == id)?;
        let mut close = Self {
            target: Target::Herdr(HerdrTarget {
                boot: snapshot.boot_id.clone(),
                workspace: tab.workspace_id.clone(),
                tab: tab.tab_id.clone(),
                pane: None,
            }),
            label: tab.label.clone(),
            interrupts: None,
            confirm_selected: false,
            do_not_ask_again: false,
            error: None,
        };
        close.interrupts = close.interrupted(snapshot);
        Some(close)
    }

    fn capture(command: Command, snapshot: &ClientShellSnapshot) -> Option<Self> {
        if !matches!(command, Command::ClosePane | Command::CloseTab) {
            return None;
        }
        controls::request(command, snapshot)?;
        if command == Command::ClosePane {
            Self::capture_pane(snapshot, snapshot.focused_pane_id.as_deref()?)
        } else {
            Self::capture_tab(snapshot, snapshot.focused_tab_id.as_deref()?)
        }
    }

    fn is_pane(&self) -> bool {
        match &self.target {
            Target::Herdr(target) => target.pane.is_some(),
            Target::Core(_) => true,
            Target::Batch(_) | Target::Workspace(_) => false,
        }
    }

    /// Names the agents in this pane or tab that are working or waiting on
    /// the user, so the dialog says what closing it stops; none when nothing
    /// is in flight.
    fn interrupted(&self, snapshot: &ClientShellSnapshot) -> Option<String> {
        let Target::Herdr(target) = &self.target else {
            return None;
        };
        let busy: Vec<_> = snapshot
            .agents
            .iter()
            .filter(|agent| {
                agent.tab_id == target.tab
                    && target
                        .pane
                        .as_ref()
                        .is_none_or(|pane| agent.pane_id == *pane)
                    && matches!(
                        agent.agent_status,
                        AgentStatus::Working | AgentStatus::Blocked
                    )
            })
            .collect();
        match busy.as_slice() {
            [] => None,
            [agent] => {
                let name = crate::sidebar::agent_name(agent);
                Some(if agent.agent_status == AgentStatus::Blocked {
                    fl!("close-modal-agent-waiting", agent = name)
                } else {
                    fl!("close-modal-agent-working", agent = name)
                })
            }
            agents => Some(fl!("close-modal-agents-busy", count = agents.len())),
        }
    }

    /// Whether closing this target could interrupt an agent mid-task: one
    /// working, or blocked on a prompt. Idle, done, and unknown agents, and
    /// tabs without any, have nothing in flight to lose. Pane closes ask
    /// unless `confirm_close_pane` is off, so this only matters for tabs.
    fn interrupts_agent(&self, snapshot: &ClientShellSnapshot) -> bool {
        let Target::Herdr(target) = &self.target else {
            return false;
        };
        let busy = |status| matches!(status, AgentStatus::Working | AgentStatus::Blocked);
        // The tab's aggregate status may rank a finished agent above a
        // working one, so each agent in the tab is checked as well.
        snapshot
            .tabs
            .iter()
            .any(|tab| tab.tab_id == target.tab && busy(tab.agent_status))
            || snapshot
                .agents
                .iter()
                .any(|agent| agent.tab_id == target.tab && busy(agent.agent_status))
    }

    fn request(&self, snapshot: &ClientShellSnapshot) -> Result<(Method, Value)> {
        let Target::Herdr(target) = &self.target else {
            return Err(Error::StaleCloseTarget);
        };
        target.request(snapshot)
    }
}

impl HerdrTarget {
    fn request(&self, snapshot: &ClientShellSnapshot) -> Result<(Method, Value)> {
        let target = self;
        if snapshot.boot_id != target.boot
            || !snapshot
                .workspaces
                .iter()
                .any(|workspace| workspace.workspace_id == target.workspace)
            || !snapshot
                .tabs
                .iter()
                .any(|tab| tab.tab_id == target.tab && tab.workspace_id == target.workspace)
            || target.pane.as_ref().is_some_and(|id| {
                !snapshot.panes.iter().any(|pane| {
                    &pane.pane_id == id
                        && pane.tab_id == target.tab
                        && pane.workspace_id == target.workspace
                })
            })
        {
            return Err(Error::StaleCloseTarget);
        }
        Ok(if let Some(id) = &target.pane {
            (Method::PaneClose, json!({"pane_id": id}))
        } else {
            (Method::TabClose, json!({"tab_id": target.tab}))
        })
    }
}

impl SheprdWindow {
    pub(super) fn open_tab_close(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let close = self
            .live
            .snapshot
            .as_ref()
            .and_then(|snapshot| CloseConfirmation::capture_tab(snapshot, id));
        self.show_close(close, window, cx);
    }

    pub(super) fn open_close_confirmation(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let close = self
            .live
            .snapshot
            .as_ref()
            .and_then(|snapshot| CloseConfirmation::capture(command, snapshot));
        self.show_close(close, window, cx);
    }

    fn show_close(
        &mut self,
        close: Option<CloseConfirmation>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(close) = close else {
            return;
        };
        if self.open_menu(window, cx) {
            self.present_close(close, window, cx);
        }
    }

    /// Shows the confirmation in a menu the caller has already opened and
    /// fenced, or closes straight away when confirmation is turned off for the
    /// target kind or a tab has no agent mid-task. The immediate close still
    /// goes through `confirm_close`, so its connection and target checks hold
    /// and a refusal stays visible in the dialog.
    pub(super) fn present_close(
        &mut self,
        close: CloseConfirmation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let immediate = if matches!(&close.target, Target::Batch(_) | Target::Workspace(_)) {
            false
        } else if close.is_pane() {
            !self.config.confirm_close_pane
        } else {
            !self.config.confirm_close_tab
                || !self
                    .live
                    .snapshot
                    .as_ref()
                    .is_some_and(|snapshot| close.interrupts_agent(snapshot))
        };
        self.menu.close = Some(close);
        self.menu.page = Some(Page::ConfirmClose);
        if immediate {
            self.confirm_close(window, cx);
        } else {
            cx.notify();
        }
    }

    fn confirm_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(edit) = self.send_close(window, cx) {
            self.save_preference(move || crate::config::Config::save_preference(edit), cx);
        }
    }

    /// Whether the open dialog offers "Do not ask again": only for a pane, and
    /// only while pane closes still ask. A refused immediate close falls back
    /// to the dialog with the option already off, so there is nothing to save.
    fn offers_do_not_ask_again(&self) -> bool {
        self.config.confirm_close_pane
            && self
                .menu
                .close
                .as_ref()
                .is_some_and(CloseConfirmation::is_pane)
    }

    /// Sends the close, returning the preference "Do not ask again" asks to
    /// persist once the close actually went out; a refusal persists nothing.
    pub(super) fn send_close(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Preference> {
        let close = self.menu.close.as_ref()?;
        let target = match &close.target {
            Target::Batch(target) => {
                self.begin_close_batch(target.clone(), window, cx);
                return None;
            }
            Target::Workspace(target) => {
                self.send_workspace_close(target.clone(), window, cx);
                return None;
            }
            Target::Core(target) => return self.send_core_pane_close(target.clone(), window, cx),
            Target::Herdr(target) => target,
        };
        let target = target.clone();
        let stop_asking = close.do_not_ask_again;
        let result = self.queue_herdr_close(&target, cx);
        match result {
            Ok(()) => {
                self.dismiss_menu(window, cx);
                stop_asking.then_some(Preference::ConfirmClosePane(false))
            }
            Err(error) => {
                if let Some(close) = &mut self.menu.close {
                    close.error = Some(error.to_string());
                }
                cx.notify();
                None
            }
        }
    }

    fn queue_herdr_close(&mut self, target: &HerdrTarget, cx: &mut Context<Self>) -> Result<()> {
        let (method, params) = (|| {
            if !self.menu_target_current() || !self.input_ready() {
                return Err(Error::StaleConnection);
            }
            if target.pane.is_some()
                && self
                    .live
                    .surface
                    .as_ref()
                    .is_some_and(|s| s.popup.is_some())
            {
                return Err(Error::ConnectionNotReady);
            }
            let snapshot = self.live.snapshot.as_ref().ok_or(Error::NotConnected)?;
            target.request(snapshot)
        })()?;
        // Closing a tab, or the last pane of one, closes the tab.
        let closing_tab = self.live.snapshot.as_ref().and_then(|snapshot| {
            let last_pane = snapshot
                .panes
                .iter()
                .filter(|p| p.tab_id == target.tab && p.workspace_id == target.workspace)
                .count()
                <= 1;
            (target.pane.is_none() || last_pane)
                .then(|| crate::browser::ClosedHerdrTab::capture(snapshot, &target.tab))
                .flatten()
        });
        let sent = self.request_focus_change(method.as_str(), None, |handle, boot| {
            handle.request(boot, method, params)
        });
        if !sent {
            return Err(Error::ConnectionNotReady);
        }
        if let Some(closed) = closing_tab {
            self.record_closed_herdr_tab(closed, cx);
        }
        Ok(())
    }

    pub(super) fn close_confirmation_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        window.prevent_default();
        match event.keystroke.key.as_str() {
            "enter" if event.is_held => {}
            "escape" => self.dismiss_menu(window, cx),
            "tab" | "left" | "right" => {
                if let Some(close) = &mut self.menu.close {
                    close.confirm_selected = !close.confirm_selected;
                }
                cx.notify();
            }
            "space" => self.toggle_do_not_ask_again(cx),
            "enter" => {
                if self
                    .menu
                    .close
                    .as_ref()
                    .is_some_and(|close| close.confirm_selected)
                {
                    self.confirm_close(window, cx);
                } else {
                    self.dismiss_menu(window, cx);
                }
            }
            _ => {}
        }
    }

    fn toggle_do_not_ask_again(&mut self, cx: &mut Context<Self>) {
        if !self.offers_do_not_ask_again() {
            return;
        }
        if let Some(close) = &mut self.menu.close {
            close.do_not_ask_again = !close.do_not_ask_again;
            cx.notify();
        }
    }

    pub(crate) fn close_confirmation_paint(&self) -> Option<Paint> {
        let close = self.menu.close.as_ref()?;
        let (title, note, confirm) = if let Target::Batch(target) = &close.target {
            (
                fl!("close-modal-batch-title", count = target.items.len()),
                fl!(
                    "close-modal-batch-note",
                    panes = target.pane_count(),
                    views = target.view_count()
                ),
                fl!("close-modal-close-batch", count = target.items.len()),
            )
        } else if let Target::Workspace(target) = &close.target {
            (
                if target.workspace_count() > 1 {
                    fl!("close-modal-workspace-group-title")
                } else {
                    fl!("close-modal-workspace-title")
                },
                fl!(
                    "close-modal-workspace-note",
                    workspaces = target.workspace_count(),
                    panes = target.pane_count()
                ),
                if target.workspace_count() > 1 {
                    fl!("close-modal-close-workspace-group")
                } else {
                    fl!("close-modal-close-workspace")
                },
            )
        } else if close.is_pane() {
            (
                fl!("close-modal-pane-title"),
                fl!("close-modal-pane-note"),
                fl!("close-modal-close-pane"),
            )
        } else {
            (
                fl!("close-modal-tab-title"),
                fl!("close-modal-tab-note"),
                fl!("close-modal-close-tab"),
            )
        };
        Some(Paint {
            title,
            note,
            confirm,
            label: close.label.clone(),
            interrupts: close.interrupts.clone(),
            confirm_selected: close.confirm_selected,
            do_not_ask_again: close.do_not_ask_again,
            offers_do_not_ask_again: self.offers_do_not_ask_again(),
            error: close.error.clone(),
            long_labels: matches!(&close.target, Target::Batch(_) | Target::Workspace(_)),
            offset: self.menu.close_targets_scroll.offset(),
        })
    }

    pub(super) fn render_close_confirmation(&self, cx: &mut Context<Self>) -> Div {
        let Some(paint) = self.close_confirmation_paint() else {
            return div();
        };
        self.render_close_content(&paint, true, cx)
    }

    pub(crate) fn render_close_paint(&self, paint: &Paint, cx: &mut Context<Self>) -> Div {
        self.render_close_content(paint, false, cx)
    }

    fn render_close_content(&self, close: &Paint, active: bool, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        div()
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .text_size(px(self.config.ui.size * 1.35))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(close.title.clone()),
            )
            .child(
                div()
                    .id("close-targets")
                    .debug_selector(|| "close-targets".into())
                    .when(close.long_labels, |labels| {
                        labels
                            .max_h(px(160.))
                            .when(active, |labels| {
                                labels
                                    .overflow_y_scroll()
                                    .track_scroll(&self.menu.close_targets_scroll)
                            })
                            .when(!active, |labels| labels.overflow_hidden())
                    })
                    .child(
                        div()
                            .debug_selector(|| "close-target-text".into())
                            .relative()
                            .when(!active, |label| label.top(close.offset.y))
                            .child(close.label.clone()),
                    ),
            )
            .when_some(close.interrupts.clone(), |panel, interrupts| {
                panel.child(
                    div()
                        .debug_selector(|| "close-interrupts".into())
                        .text_color(crate::menu::danger(theme))
                        .child(interrupts),
                )
            })
            .child(
                div()
                    .text_color(rgb(theme.subtext()))
                    .child(close.note.clone()),
            )
            .when_some(close.error.clone(), |panel, error| {
                panel.child(div().bg(rgb(theme.active)).p(px(8.)).child(error))
            })
            .when(close.offers_do_not_ask_again, |panel| {
                panel.child(
                    div()
                        .id("close-do-not-ask")
                        .debug_selector(|| "close-do-not-ask".into())
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .cursor_pointer()
                        .text_color(rgb(theme.muted))
                        .child(crate::toggles::checkbox(
                            theme,
                            self.config.ui.size + 2.,
                            close.do_not_ask_again,
                        ))
                        .child(fl!("close-modal-do-not-ask"))
                        .when(active, |row| {
                            row.on_click(
                                cx.listener(|this, _, _, cx| this.toggle_do_not_ask_again(cx)),
                            )
                        }),
                )
            })
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(
                        close_button("close-cancel", fl!("close-modal-cancel"), active)
                            .when(active && !close.confirm_selected, |button| {
                                button.aria_active_descendant()
                            })
                            .debug_selector(|| "close-cancel".into())
                            .px(px(12.))
                            .py(px(6.))
                            .rounded(px(crate::config::corners::CONTROL))
                            .border_1()
                            .border_color(rgb(if close.confirm_selected {
                                theme.active
                            } else {
                                theme.foreground
                            }))
                            .cursor_pointer()
                            .when(active, |button| {
                                button.hover(|s| s.bg(rgb(theme.active))).on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.dismiss_menu(window, cx)
                                    }),
                                )
                            }),
                    )
                    .child(
                        close_button("close-confirm", close.confirm.clone(), active)
                            .when(active && close.confirm_selected, |button| {
                                button.aria_active_descendant()
                            })
                            .debug_selector(|| "close-confirm".into())
                            .px(px(12.))
                            .py(px(6.))
                            .rounded(px(crate::config::corners::CONTROL))
                            .border_1()
                            .border_color(rgb(if close.confirm_selected {
                                theme.foreground
                            } else {
                                theme.active
                            }))
                            .bg(rgb(theme.active))
                            .text_color(crate::menu::danger(theme))
                            .cursor_pointer()
                            .when(active, |button| {
                                button.on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.confirm_close(window, cx)
                                    }),
                                )
                            }),
                    ),
            )
    }
}

fn close_button(id: &'static str, label: String, active: bool) -> Stateful<Div> {
    div()
        .id(id)
        .when(active, |button| {
            button.role(Role::Button).aria_label(label.clone())
        })
        .child(label)
}

#[cfg(test)]
mod tests;
