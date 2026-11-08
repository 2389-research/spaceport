//! Shows Core's watched Spaceport groups and edits one group from a dialog.
//! The UI reads the shared observation and sends blocking CAS work to a worker.

use super::{
    Start, State as HubState, observation,
    workspace_document::{VersionedDocument, edit},
};
use crate::{SheprdWindow, dialog_input::DialogInput, i18n::fl, menu::Page, window::Flash};
use gpui::{prelude::*, *};
use std::sync::{
    Arc, Mutex, MutexGuard,
    mpsc::{self, Receiver, TryRecvError},
};
use uuid::Uuid;

#[derive(Default)]
pub(super) struct State {
    pending: Option<Receiver<edit::Result<()>>>,
}

#[derive(Default)]
struct EditGuard {
    shared: Arc<Mutex<GuardState>>,
}

impl Global for EditGuard {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum GuardState {
    #[default]
    Idle,
    // Any in-flight edit can rebase, so its eventual write version is unknown.
    Pending(u64),
    // Only a newer observed document version can fence this CAS attempt.
    Uncertain(u64),
}

impl EditGuard {
    fn blocks(cx: &App, version: u64) -> bool {
        cx.try_global::<Self>()
            .is_some_and(|guard| Self::blocked(*Self::lock(&guard.shared), version))
    }

    fn blocked(state: GuardState, version: u64) -> bool {
        match state {
            GuardState::Idle => false,
            GuardState::Pending(_) => true,
            GuardState::Uncertain(attempted) => version <= attempted,
        }
    }

    fn begin(cx: &mut App, version: u64) -> Option<Arc<Mutex<GuardState>>> {
        if !cx.has_global::<Self>() {
            cx.set_global(Self::default());
        }
        let shared = Arc::clone(&cx.global::<Self>().shared);
        let mut state = Self::lock(&shared);
        if Self::blocked(*state, version) {
            return None;
        }
        *state = GuardState::Pending(version);
        drop(state);
        Some(shared)
    }

    fn finish(shared: &Arc<Mutex<GuardState>>, version: u64, outcome: &edit::Result<()>) {
        let mut state = Self::lock(shared);
        if *state != GuardState::Pending(version) {
            return;
        }
        *state = match outcome {
            Err(edit::Error::WriteUnknown { version, .. }) => GuardState::Uncertain(*version),
            _ => GuardState::Idle,
        };
    }

    fn clear(shared: &Arc<Mutex<GuardState>>, version: u64) {
        let mut state = Self::lock(shared);
        if *state == GuardState::Pending(version) {
            *state = GuardState::Idle;
        }
    }

    fn lock(shared: &Arc<Mutex<GuardState>>) -> MutexGuard<'_, GuardState> {
        match shared.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

enum Access<'a> {
    Missing,
    Error(&'a observation::Error),
    Ready(&'a VersionedDocument),
}

impl<'a> Access<'a> {
    fn of(projection: Option<&'a observation::Projection>) -> Self {
        match projection {
            Some(projection) => {
                if let Some(error) = projection.error() {
                    Self::Error(error)
                } else if projection.fresh() {
                    projection.document().map_or(Self::Missing, Self::Ready)
                } else {
                    Self::Missing
                }
            }
            _ => Self::Missing,
        }
    }
}

impl SheprdWindow {
    fn core_workspace_access<'a>(&self, cx: &'a App) -> Access<'a> {
        Access::of(
            cx.try_global::<observation::Observation>()
                .map(|observation| observation.projection()),
        )
    }

    pub(crate) fn open_core_workspace_dialog(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.open_menu(window, cx) {
            return;
        }
        self.menu.page = Some(Page::CoreWorkspaceCreate);
        let mut input = DialogInput::new(String::new());
        input.label = Some(fl!("workspace-dialog-name").into());
        self.menu.input = Some(input);
        cx.notify();
    }

    pub(crate) fn submit_core_workspace_dialog(&mut self, cx: &mut Context<Self>) {
        if self.menu.page != Some(Page::CoreWorkspaceCreate)
            || self.core_terminals.workspace_ui.pending.is_some()
        {
            return;
        }
        let Some(input) = self
            .menu
            .input
            .as_ref()
            .filter(|input| input.marked.is_none() && !input.text.trim().is_empty())
        else {
            return;
        };
        let label = input.text.trim().to_owned();
        let base = match self.core_workspace_access(cx) {
            Access::Ready(base) => base,
            Access::Error(_) | Access::Missing => return,
        };
        let version = base.version;
        let base = base.clone();
        let Some(backend) = self
            .core_terminals
            .hub
            .backend()
            .filter(|backend| !backend.core().is_lost())
            .cloned()
        else {
            return;
        };
        let (sender, receiver) = mpsc::sync_channel(1);
        let Some(guard) = EditGuard::begin(cx, version) else {
            return;
        };
        let worker_guard = Arc::clone(&guard);
        let edit = edit::Edit::Create {
            id: Uuid::new_v4(),
            label,
        };
        match std::thread::Builder::new()
            .name("core-workspace-edit".into())
            .spawn(move || {
                let outcome = edit::apply(backend.core(), base, &edit).map(|_| ());
                EditGuard::finish(&worker_guard, version, &outcome);
                let _ = sender.send(outcome);
            }) {
            Ok(_) => {
                self.core_terminals.workspace_ui.pending = Some(receiver);
                self.menu.error = None;
            }
            Err(error) => {
                EditGuard::clear(&guard, version);
                self.menu.error = Some(fl!(
                    "core-workspace-worker-error",
                    reason = bounded(&error.to_string())
                ))
            }
        }
        cx.notify();
    }

    pub(crate) fn poll_core_workspace_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(pending) = self.core_terminals.workspace_ui.pending.as_ref() else {
            return;
        };
        let outcome = match pending.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                self.core_terminals.workspace_ui.pending = None;
                self.core_workspace_edit_error(fl!("core-workspace-worker-lost"), cx);
                return;
            }
        };
        self.core_terminals.workspace_ui.pending = None;
        match outcome {
            Ok(()) if self.menu.page == Some(Page::CoreWorkspaceCreate) => {
                self.dismiss_menu(window, cx)
            }
            Ok(()) => cx.notify(),
            Err(error) => {
                let unknown = matches!(&error, edit::Error::WriteUnknown { .. });
                self.core_workspace_edit_error(
                    if unknown {
                        fl!(
                            "core-workspace-unknown",
                            reason = bounded(&error.to_string())
                        )
                    } else {
                        bounded(&error.to_string())
                    },
                    cx,
                );
            }
        }
    }

    fn core_workspace_edit_error(&mut self, message: String, cx: &mut Context<Self>) {
        if self.menu.page == Some(Page::CoreWorkspaceCreate) {
            self.menu.error = Some(message);
            cx.notify();
        } else {
            self.show_flash(Flash::warning(message), cx);
        }
    }

    pub(crate) fn core_workspace_rows(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = &self.theme;
        let mut section = div()
            .id("core-workspaces")
            .debug_selector(|| "core-workspaces".into())
            .flex()
            .flex_col()
            .mt_3()
            .px_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(fl!("core-workspace-section")),
                    )
                    .child(
                        div()
                            .id("core-workspace-new")
                            .role(Role::Button)
                            .aria_label(fl!("core-workspace-new"))
                            .cursor_pointer()
                            .text_color(rgb(theme.foreground))
                            .child(fl!("core-workspace-new"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_core_workspace_dialog(window, cx);
                                this.core_terminals.hub.connect(Start::IfMissing);
                            })),
                    ),
            );
        match self.core_workspace_access(cx) {
            Access::Ready(document) if document.value.workspaces().is_empty() => {
                section = section.child(
                    div()
                        .text_color(rgb(theme.subtext()))
                        .child(fl!("core-workspace-empty")),
                );
            }
            Access::Ready(document) => {
                for workspace in document.value.workspaces().iter().take(50) {
                    let label = workspace.label.clone();
                    section = section.child(
                        div()
                            .id(SharedString::from(format!(
                                "core-workspace-{}",
                                workspace.id
                            )))
                            .role(Role::ListItem)
                            .py_1()
                            .truncate()
                            .text_color(rgb(if workspace.archived {
                                theme.subtext()
                            } else {
                                theme.foreground
                            }))
                            .child(crate::sidebar::label_text(&label)),
                    );
                }
                let remaining = document.value.workspaces().len().saturating_sub(50);
                if remaining > 0 {
                    section = section.child(
                        div()
                            .text_color(rgb(theme.subtext()))
                            .child(fl!("core-workspace-more", count = remaining)),
                    );
                }
            }
            Access::Error(error) => {
                section = section.child(div().text_color(rgb(theme.subtext())).child(fl!(
                    "core-workspace-error",
                    reason = bounded(&error.to_string())
                )))
            }
            Access::Missing => {
                let state = self.core_terminals.hub.state();
                let joinable = matches!(state, HubState::Idle | HubState::Unavailable(_));
                let message = match state {
                    HubState::Idle => fl!("core-workspace-connect"),
                    HubState::Connecting(_) | HubState::Ready(_) => fl!("core-workspace-loading"),
                    HubState::Unavailable(error) => fl!(
                        "core-workspace-join-error",
                        reason = bounded(&error.to_string())
                    ),
                };
                section = section.child(
                    div()
                        .id("core-workspace-status")
                        .text_color(rgb(theme.subtext()))
                        .truncate()
                        .when(joinable, |row| {
                            row.role(Role::Button)
                                .aria_label(message.clone())
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.core_terminals.hub.connect(Start::Join);
                                    cx.notify();
                                }))
                        })
                        .child(message),
                );
            }
        }
        section.into_any_element()
    }

    pub(crate) fn render_core_workspace_dialog(&self, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let status = if self.core_terminals.workspace_ui.pending.is_some() {
            Some(fl!("core-workspace-creating"))
        } else {
            match self.core_workspace_access(cx) {
                Access::Error(error) => Some(fl!(
                    "core-workspace-error",
                    reason = bounded(&error.to_string())
                )),
                Access::Missing => Some(fl!("core-workspace-stale")),
                Access::Ready(document) if EditGuard::blocks(cx, document.version) => {
                    Some(fl!("core-workspace-uncertain"))
                }
                Access::Ready(_)
                    if !self
                        .core_terminals
                        .hub
                        .backend()
                        .is_some_and(|backend| !backend.core().is_lost()) =>
                {
                    Some(fl!("core-workspace-connect"))
                }
                Access::Ready(_) => None,
            }
        };
        let armed = status.is_none()
            && self
                .menu
                .input
                .as_ref()
                .is_some_and(|input| !input.text.trim().is_empty() && input.marked.is_none());
        let mut body = div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(fl!("menu-page-new-workspace")),
            )
            .child(self.render_dialog_input(cx));
        if let Some(error) = &self.menu.error {
            body = body.child(
                div()
                    .text_color(crate::menu::danger(theme))
                    .child(error.clone()),
            );
        } else if let Some(status) = status {
            body = body.child(div().text_color(rgb(theme.subtext())).child(status));
        }
        div().child(body).child(
            div()
                .flex()
                .justify_end()
                .gap_2()
                .p_4()
                .child(
                    div()
                        .id("core-workspace-cancel")
                        .role(Role::Button)
                        .aria_label(fl!("workspace-dialog-cancel"))
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _, window, cx| this.dismiss_menu(window, cx)))
                        .child(fl!("workspace-dialog-cancel")),
                )
                .child(
                    div()
                        .id("core-workspace-create")
                        .role(Role::Button)
                        .aria_label(fl!("workspace-dialog-create"))
                        .when(armed, |button| {
                            button.cursor_pointer().on_click(
                                cx.listener(|this, _, _, cx| this.submit_core_workspace_dialog(cx)),
                            )
                        })
                        .child(fl!("workspace-dialog-create")),
                ),
        )
    }
}

fn bounded(message: &str) -> String {
    message.chars().take(300).collect()
}

#[cfg(test)]
mod tests;
