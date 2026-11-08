//! Shows Core's watched Spaceport groups and edits one group from a dialog.
//! The UI reads the shared observation and sends blocking CAS work to a worker.

mod render;

use super::{
    observation,
    panes::Panes,
    workspace_document::{VersionedDocument, edit},
};
use crate::{
    SheprdWindow,
    browser::{Location, Store},
    dialog_input::DialogInput,
    i18n::fl,
    menu::Page,
    window::Flash,
};
use gpui::{prelude::*, *};
use sheprd_backend::TerminalId;
use std::sync::{
    Arc, Mutex, MutexGuard,
    mpsc::{self, Receiver, TryRecvError},
};
use uuid::Uuid;

#[derive(Default)]
pub(super) struct State {
    dialog_generation: u64,
    pending: Option<Pending>,
    selected: MenuAction,
    selected_group: Option<Uuid>,
    target: Option<DialogTarget>,
}

impl State {
    fn open(&mut self, target: Option<DialogTarget>) {
        self.dialog_generation = self.dialog_generation.wrapping_add(1);
        self.selected = MenuAction::Rename;
        self.target = target;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum MenuAction {
    #[default]
    Rename,
    Archive,
    Delete,
}

impl MenuAction {
    fn step(self, forward: bool) -> Self {
        match (self, forward) {
            (Self::Rename, true) | (Self::Delete, false) => Self::Archive,
            (Self::Archive, true) => Self::Delete,
            (Self::Archive, false) => Self::Rename,
            (Self::Delete, true) => Self::Rename,
            (Self::Rename, false) => Self::Delete,
        }
    }
}

struct Pending {
    dialog_generation: u64,
    dismiss_page: Option<Page>,
    receiver: Receiver<edit::Result<()>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DialogTarget {
    epoch: u64,
    id: Uuid,
    version: u64,
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
    fn core_workspace_epoch(cx: &App) -> Option<u64> {
        cx.try_global::<observation::Observation>()
            .map(|observation| observation.projection().epoch())
    }

    fn core_workspace_access<'a>(&self, cx: &'a App) -> Access<'a> {
        Access::of(
            cx.try_global::<observation::Observation>()
                .map(|observation| observation.projection()),
        )
    }

    fn select_core_workspace(&mut self, id: Uuid, cx: &mut Context<Self>) {
        if !matches!(self.core_workspace_access(cx), Access::Ready(doc) if doc.value.workspaces().iter().any(|row| row.id == id))
        {
            return;
        }
        let selected = &mut self.core_terminals.workspace_ui.selected_group;
        *selected = if *selected == Some(id) {
            None
        } else {
            Some(id)
        };
        cx.notify();
    }

    fn toggle_core_workspace_member(
        &mut self,
        group: Uuid,
        core_id: String,
        terminal_id: String,
        member: bool,
        cx: &mut Context<Self>,
    ) {
        let document = match self.core_workspace_access(cx) {
            Access::Ready(document) => document,
            _ => {
                self.show_flash(Flash::warning(fl!("core-workspace-stale")), cx);
                return;
            }
        };
        let Some(workspace) = document
            .value
            .workspaces()
            .iter()
            .find(|row| row.id == group)
        else {
            return;
        };
        let present = workspace
            .members
            .iter()
            .any(|item| item.core_id == core_id && item.terminal_id == terminal_id);
        if present == member {
            return;
        }
        if member {
            let local = self
                .core_terminals
                .hub
                .backend()
                .map(|backend| &backend.core().hello().core.id);
            let observed = cx
                .try_global::<observation::Observation>()
                .is_some_and(|observation| {
                    observation.projection().fresh()
                        && observation.projection().core_id() == Some(core_id.as_str())
                        && observation
                            .projection()
                            .terminals()
                            .iter()
                            .any(|info| info.id.as_str() == terminal_id)
                });
            if local != Some(&core_id) || terminal_id.contains('/') || !observed {
                self.show_flash(Flash::warning(fl!("core-workspace-stale")), cx);
                return;
            }
        }
        let edit = if member {
            edit::Edit::AddMember {
                id: group,
                core_id,
                terminal_id,
            }
        } else {
            edit::Edit::RemoveMember {
                id: group,
                core_id,
                terminal_id,
            }
        };
        self.submit_core_workspace_edit(edit, cx);
        if let Some(error) = self.menu.error.take() {
            self.show_flash(Flash::warning(error), cx);
        }
    }

    fn open_core_workspace_member(
        &mut self,
        group: Uuid,
        core_id: &str,
        terminal_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(backend) = self.core_terminals.hub.backend() else {
            return;
        };
        if backend.core().is_lost() || backend.core().hello().core.id != core_id {
            return;
        }
        let Access::Ready(document) = self.core_workspace_access(cx) else {
            return;
        };
        if !document.value.workspaces().iter().any(|workspace| {
            workspace.id == group
                && workspace
                    .members
                    .iter()
                    .any(|member| member.core_id == core_id && member.terminal_id == terminal_id)
        }) {
            return;
        }
        let observed = cx
            .try_global::<observation::Observation>()
            .is_some_and(|observation| {
                observation.projection().fresh()
                    && observation.projection().core_id() == Some(core_id)
                    && observation
                        .projection()
                        .terminals()
                        .iter()
                        .any(|info| info.id.as_str() == terminal_id)
            });
        if !observed || terminal_id.contains('/') {
            return;
        }
        let Ok(terminal) = TerminalId::new(terminal_id.to_owned()) else {
            return;
        };
        if self.selected_is_remote() {
            self.show_flash(Flash::warning(fl!("core-terminals-local-only")), cx);
            return;
        }
        let Some((scope, workspace)) = self.browser_key() else {
            self.show_flash(Flash::warning(fl!("core-terminals-open-workspace")), cx);
            return;
        };
        let existing = cx.try_global::<Store>().and_then(|store| {
            store
                .in_workspace(&scope, &workspace)
                .find_map(|tab| match &tab.location {
                    Some(Location::CoreTerminal {
                        panes,
                        observe: false,
                    }) if panes.contains(&terminal) => Some(tab.id),
                    _ => None,
                })
        });
        let tab = existing.or_else(|| {
            Store::update(cx, |store| {
                store.open(
                    scope,
                    &workspace,
                    Some(Location::CoreTerminal {
                        panes: Panes::single(terminal.clone()),
                        observe: false,
                    }),
                    None,
                )
            })
        });
        if let Some(tab) = tab {
            self.sync_core_views(cx);
            self.show_core_pane(tab, &terminal, window, cx);
        } else {
            self.show_flash(Flash::warning(fl!("core-terminals-too-many-tabs")), cx);
        }
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
        self.core_terminals.workspace_ui.open(None);
        let mut input = DialogInput::new(String::new());
        input.label = Some(fl!("workspace-dialog-name").into());
        self.menu.input = Some(input);
        cx.notify();
    }

    pub(crate) fn open_core_workspace_rename(
        &mut self,
        id: Uuid,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (version, label) = match self.core_workspace_access(cx) {
            Access::Ready(document) => {
                match document.value.workspaces().iter().find(|row| row.id == id) {
                    Some(row) => (document.version, row.label.clone()),
                    None => return,
                }
            }
            Access::Error(_) | Access::Missing => return,
        };
        let Some(epoch) = Self::core_workspace_epoch(cx) else {
            return;
        };
        if !self.open_menu(window, cx) {
            return;
        }
        self.menu.page = Some(Page::CoreWorkspaceRename);
        self.core_terminals
            .workspace_ui
            .open(Some(DialogTarget { epoch, id, version }));
        let mut input = DialogInput::new(label);
        input.label = Some(fl!("workspace-dialog-name").into());
        self.menu.input = Some(input);
        cx.notify();
    }

    pub(crate) fn open_core_workspace_delete(
        &mut self,
        id: Uuid,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let version = match self.core_workspace_access(cx) {
            Access::Ready(document)
                if document.value.workspaces().iter().any(|row| row.id == id) =>
            {
                document.version
            }
            _ => return,
        };
        let Some(epoch) = Self::core_workspace_epoch(cx) else {
            return;
        };
        if !self.open_menu(window, cx) {
            return;
        }
        self.menu.page = Some(Page::CoreWorkspaceDelete);
        self.core_terminals
            .workspace_ui
            .open(Some(DialogTarget { epoch, id, version }));
        cx.notify();
    }

    pub(crate) fn open_core_workspace_actions(
        &mut self,
        id: Uuid,
        anchor: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let version = match self.core_workspace_access(cx) {
            Access::Ready(document)
                if document.value.workspaces().iter().any(|row| row.id == id) =>
            {
                document.version
            }
            _ => return,
        };
        let Some(epoch) = Self::core_workspace_epoch(cx) else {
            return;
        };
        if !self.open_menu(window, cx) {
            return;
        }
        self.menu.anchor = anchor;
        self.menu.page = Some(Page::CoreWorkspaceActions);
        self.core_terminals
            .workspace_ui
            .open(Some(DialogTarget { epoch, id, version }));
        cx.notify();
    }

    pub(crate) fn cycle_core_workspace_action(&mut self, forward: bool, cx: &mut Context<Self>) {
        if self.menu.page != Some(Page::CoreWorkspaceActions) {
            return;
        }
        let state = &mut self.core_terminals.workspace_ui;
        state.selected = state.selected.step(forward);
        cx.notify();
    }

    fn hover_core_workspace_action(
        &mut self,
        action: MenuAction,
        hovered: bool,
        cx: &mut Context<Self>,
    ) {
        if hovered && self.menu.page == Some(Page::CoreWorkspaceActions) {
            self.core_terminals.workspace_ui.selected = action;
            cx.notify();
        }
    }

    pub(crate) fn core_workspace_delete_selected(&self) -> bool {
        self.menu.page == Some(Page::CoreWorkspaceActions)
            && self.core_terminals.workspace_ui.selected == MenuAction::Delete
    }

    pub(crate) fn activate_selected_core_workspace_action(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.activate_core_workspace_action(self.core_terminals.workspace_ui.selected, window, cx);
    }

    fn activate_core_workspace_action(
        &mut self,
        action: MenuAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.menu.page != Some(Page::CoreWorkspaceActions) {
            return;
        }
        let Some(target) = self.core_terminals.workspace_ui.target else {
            return;
        };
        let archived = match self.core_workspace_access(cx) {
            Access::Ready(document)
                if document.version == target.version
                    && Self::core_workspace_epoch(cx) == Some(target.epoch) =>
            {
                match document
                    .value
                    .workspaces()
                    .iter()
                    .find(|row| row.id == target.id)
                {
                    Some(row) => row.archived,
                    None => return,
                }
            }
            _ => return,
        };
        match action {
            MenuAction::Rename => self.open_core_workspace_rename(target.id, window, cx),
            MenuAction::Archive => {
                if self.core_terminals.workspace_ui.pending.is_some() {
                    self.menu.error = Some(fl!("core-workspace-editing"));
                    cx.notify();
                    return;
                }
                self.toggle_core_workspace_archive(target.id, archived, cx);
                if self.core_terminals.workspace_ui.pending.is_some() {
                    self.dismiss_menu(window, cx);
                }
            }
            MenuAction::Delete => self.open_core_workspace_delete(target.id, window, cx),
        }
    }

    fn toggle_core_workspace_archive(
        &mut self,
        id: Uuid,
        was_archived: bool,
        cx: &mut Context<Self>,
    ) {
        let current = match self.core_workspace_access(cx) {
            Access::Ready(document) => document
                .value
                .workspaces()
                .iter()
                .find(|row| row.id == id)
                .map(|row| row.archived),
            Access::Error(_) | Access::Missing => None,
        };
        if current != Some(was_archived) {
            self.menu.error = Some(fl!("core-workspace-changed"));
            cx.notify();
            return;
        }
        self.submit_core_workspace_edit(
            edit::Edit::Archive {
                id,
                archived: !was_archived,
            },
            cx,
        );
    }

    pub(crate) fn submit_core_workspace_dialog(&mut self, cx: &mut Context<Self>) {
        if self.core_terminals.workspace_ui.pending.is_some() {
            return;
        }
        let edit = match self.menu.page {
            Some(Page::CoreWorkspaceCreate | Page::CoreWorkspaceRename) => {
                let Some(input) = self
                    .menu
                    .input
                    .as_ref()
                    .filter(|input| input.marked.is_none() && !input.text.trim().is_empty())
                else {
                    return;
                };
                let label = input.text.trim().to_owned();
                if self.menu.page == Some(Page::CoreWorkspaceCreate) {
                    edit::Edit::Create {
                        id: Uuid::new_v4(),
                        label,
                    }
                } else {
                    let Some(target) = self.core_terminals.workspace_ui.target else {
                        return;
                    };
                    edit::Edit::Rename {
                        id: target.id,
                        label,
                    }
                }
            }
            Some(Page::CoreWorkspaceDelete) => {
                let Some(target) = self.core_terminals.workspace_ui.target else {
                    return;
                };
                edit::Edit::Delete { id: target.id }
            }
            _ => return,
        };
        self.submit_core_workspace_edit(edit, cx);
    }

    fn submit_core_workspace_edit(&mut self, edit: edit::Edit, cx: &mut Context<Self>) {
        if self.core_terminals.workspace_ui.pending.is_some() {
            self.menu.error = Some(fl!("core-workspace-editing"));
            cx.notify();
            return;
        }
        let base = match self.core_workspace_access(cx) {
            Access::Ready(base) => base,
            Access::Error(error) => {
                self.menu.error = Some(fl!(
                    "core-workspace-error",
                    reason = bounded(&error.to_string())
                ));
                cx.notify();
                return;
            }
            Access::Missing => {
                self.menu.error = Some(fl!("core-workspace-stale"));
                cx.notify();
                return;
            }
        };
        if let Some(target) = self.core_terminals.workspace_ui.target
            && matches!(edit, edit::Edit::Rename { .. } | edit::Edit::Delete { .. })
            && (target.version != base.version
                || Self::core_workspace_epoch(cx) != Some(target.epoch)
                || !base
                    .value
                    .workspaces()
                    .iter()
                    .any(|row| row.id == target.id))
        {
            self.menu.error = Some(fl!("core-workspace-changed"));
            cx.notify();
            return;
        }
        let version = base.version;
        let base = base.clone();
        let Some(backend) = self
            .core_terminals
            .hub
            .backend()
            .filter(|backend| !backend.core().is_lost())
            .cloned()
        else {
            self.menu.error = Some(fl!("core-workspace-connect"));
            cx.notify();
            return;
        };
        let (sender, receiver) = mpsc::sync_channel(1);
        let Some(guard) = EditGuard::begin(cx, version) else {
            self.menu.error = Some(fl!("core-workspace-uncertain"));
            cx.notify();
            return;
        };
        let dismiss_page = match &edit {
            edit::Edit::AddMember { .. }
            | edit::Edit::Archive { .. }
            | edit::Edit::RemoveMember { .. } => None,
            edit::Edit::Create { .. } => Some(Page::CoreWorkspaceCreate),
            edit::Edit::Delete { .. } => Some(Page::CoreWorkspaceDelete),
            edit::Edit::Rename { .. } => Some(Page::CoreWorkspaceRename),
        };
        let dialog_generation = self.core_terminals.workspace_ui.dialog_generation;
        let worker_guard = Arc::clone(&guard);
        match std::thread::Builder::new()
            .name("core-workspace-edit".into())
            .spawn(move || {
                let outcome = edit::apply(backend.core(), base, &edit).map(|_| ());
                EditGuard::finish(&worker_guard, version, &outcome);
                let _ = sender.send(outcome);
            }) {
            Ok(_) => {
                self.core_terminals.workspace_ui.pending = Some(Pending {
                    dialog_generation,
                    dismiss_page,
                    receiver,
                });
                self.menu.error = None;
            }
            Err(error) => {
                EditGuard::clear(&guard, version);
                self.core_workspace_edit_error(
                    fl!(
                        "core-workspace-worker-error",
                        reason = bounded(&error.to_string())
                    ),
                    dismiss_page,
                    dialog_generation,
                    cx,
                );
            }
        }
        cx.notify();
    }

    pub(crate) fn poll_core_workspace_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(pending) = self.core_terminals.workspace_ui.pending.as_ref() else {
            return;
        };
        let dismiss_page = pending.dismiss_page;
        let dialog_generation = pending.dialog_generation;
        let outcome = match pending.receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                self.core_terminals.workspace_ui.pending = None;
                self.core_workspace_edit_error(
                    fl!("core-workspace-worker-lost"),
                    dismiss_page,
                    dialog_generation,
                    cx,
                );
                return;
            }
        };
        self.core_terminals.workspace_ui.pending = None;
        match outcome {
            Ok(())
                if dismiss_page.is_some()
                    && self.menu.page == dismiss_page
                    && self.core_terminals.workspace_ui.dialog_generation == dialog_generation =>
            {
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
                    dismiss_page,
                    dialog_generation,
                    cx,
                );
            }
        }
    }

    fn core_workspace_edit_error(
        &mut self,
        message: String,
        dismiss_page: Option<Page>,
        dialog_generation: u64,
        cx: &mut Context<Self>,
    ) {
        if dismiss_page.is_some()
            && self.menu.page == dismiss_page
            && self.core_terminals.workspace_ui.dialog_generation == dialog_generation
        {
            self.menu.error = Some(message);
            cx.notify();
        } else {
            self.show_flash(Flash::warning(message), cx);
        }
    }
}

fn bounded(message: &str) -> String {
    message.chars().take(300).collect()
}

#[cfg(test)]
mod tests;
