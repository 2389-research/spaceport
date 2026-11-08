//! Terminals that herdr-core (Herdr 2) runs, shown as tabs in the window's
//! groups beside its Herdr tabs and pages. herdr-core streams raw output, so
//! the app emulates each one and paints it with the same painter as a Herdr
//! pane. A tab splits into panes the app lays out itself, drawn in the style
//! of the window's tab layout; any group can show one too, so terminals also
//! sit side by side across groups.
//!
//! Everything that waits runs off the UI thread: the hub connects (and may
//! start an installed core), each shown pane has a worker that owns its
//! `sheprd_backend::Terminal`, and terminal details are listed on a thread.
//! The window reads their mailboxes on its tick and renders the prepared
//! screens.

mod commands;
mod error;
mod fallback;
mod hub;
mod infos;
mod observation;

pub(crate) mod panes;
mod screen;
mod selection;
mod sidebar;
mod split;
mod tabs;
mod view;
mod worker;
pub(crate) mod workspace_document;
mod workspace_ui;

#[cfg(test)]
pub(crate) fn set_document_for_test_as(
    cx: &mut App,
    core_id: &str,
    doc: sheprd_core_client::wire::Doc,
) {
    observation::set_document_for_test_as(cx, core_id, doc);
}

#[cfg(test)]
pub(crate) fn set_document_stale_for_test(cx: &mut App) {
    observation::set_document_stale_for_test(cx);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;

pub(crate) use error::Error;
pub(crate) use hub::{Hub, Start, State};
pub(crate) use view::{CoreTerminalView, Look};

pub(crate) fn install_observation(cx: &mut App) {
    observation::Observation::install(cx);
}

/// The catalog Core reported on the current connection, never a local SSH list.
pub(crate) fn machine_catalog(cx: &App) -> Option<&[sheprd_core_client::wire::MachineView]> {
    cx.try_global::<observation::Observation>()?
        .projection()
        .machines()
}

pub(crate) enum RemoteDocumentView<'a> {
    Loading,
    Ready(&'a [workspace_document::Workspace]),
    Stale,
    Failed(&'a observation::Error),
}

pub(crate) fn selected_remote_machine(cx: &App) -> Option<String> {
    cx.try_global::<observation::Observation>()?
        .selected_remote()
}

pub(crate) fn select_remote_machine(cx: &mut App, machine_id: &str) -> bool {
    cx.global_mut::<observation::Observation>()
        .select_remote(machine_id)
}

pub(crate) fn remote_document_status<'a>(
    cx: &'a App,
    machine_id: &str,
) -> Option<RemoteDocumentView<'a>> {
    Some(
        match cx
            .try_global::<observation::Observation>()?
            .remote_status(machine_id)?
        {
            observation::RemoteStatus::Loading => RemoteDocumentView::Loading,
            observation::RemoteStatus::Ready(document) => {
                RemoteDocumentView::Ready(document.value.workspaces())
            }
            observation::RemoteStatus::Stale => RemoteDocumentView::Stale,
            observation::RemoteStatus::Failed(error) => RemoteDocumentView::Failed(error),
        },
    )
}

pub(crate) fn machine_catalog_error(cx: &App) -> Option<&observation::Error> {
    cx.try_global::<observation::Observation>()?
        .projection()
        .machine_error()
}

pub(crate) fn machine_catalog_started(cx: &App) -> bool {
    cx.try_global::<observation::Observation>()
        .is_some_and(|observation| observation.projection().started())
}

pub(crate) fn machine_catalog_epoch(cx: &App) -> Option<u64> {
    let projection = cx.try_global::<observation::Observation>()?.projection();
    projection.started().then(|| projection.epoch())
}

pub(crate) fn machine_catalog_fresh(cx: &App) -> bool {
    cx.try_global::<observation::Observation>()
        .is_some_and(|observation| observation.projection().fresh())
}

pub(crate) fn machine_catalog_stale(cx: &App) -> bool {
    cx.try_global::<observation::Observation>()
        .is_some_and(|observation| observation.projection().stale())
}

/// A stable class for control output; Core's arbitrary error text is not echoed.
pub(crate) fn machine_catalog_error_code(cx: &App) -> Option<&'static str> {
    let error = machine_catalog_error(cx)?;
    Some(match error {
        observation::Error::Core(sheprd_core_client::Error::Refused { .. }) => "refused",
        observation::Error::Core(sheprd_core_client::Error::Timeout(_)) => "timeout",
        observation::Error::Core(sheprd_core_client::Error::Lost) => "disconnected",
        observation::Error::Core(sheprd_core_client::Error::Unsupported(_)) => "unsupported",
        observation::Error::Core(
            sheprd_core_client::Error::Frame(_)
            | sheprd_core_client::Error::Invalid(..)
            | sheprd_core_client::Error::Json(_)
            | sheprd_core_client::Error::Protocol(_),
        ) => "invalid_response",
        _ => "failed",
    })
}

#[cfg(test)]
pub(crate) fn set_machine_catalog_for_test(
    cx: &mut App,
    machines: Option<Vec<sheprd_core_client::wire::MachineView>>,
) {
    observation::set_machines_for_test(cx, machines);
}

#[cfg(test)]
pub(crate) fn set_remote_document_for_test(
    cx: &mut App,
    machine_id: &str,
    doc: sheprd_core_client::wire::Doc,
) {
    observation::set_remote_document_for_test(cx, machine_id, doc);
}

#[cfg(test)]
pub(crate) fn set_selected_remote_machine_state_for_test(
    cx: &mut App,
    machine_id: &str,
    state: &str,
) {
    observation::set_selected_remote_machine_state_for_test(cx, machine_id, state);
}

#[cfg(test)]
pub(crate) fn set_machine_catalog_error_for_test(cx: &mut App, error: sheprd_core_client::Error) {
    observation::set_machine_error_for_test(cx, error);
}

#[cfg(test)]
pub(crate) fn set_machine_catalog_pending_for_test(cx: &mut App) {
    observation::set_machine_catalog_pending_for_test(cx);
}

use crate::{
    SheprdWindow,
    browser::{CoreGroupKey, Location, Scope, Store, TabId},
    i18n::fl,
    window::Flash,
};
use gpui::{prelude::*, *};
use panes::{Axis, Panes};
use sheprd_backend::{Backend as _, Spawn, TerminalId, TerminalInfo};
use sheprd_client::ConnectTarget;
use std::collections::HashMap;

type Started = std::sync::mpsc::Receiver<Result<TerminalInfo, sheprd_core_client::Error>>;

/// What a terminal being started is for.
#[derive(Clone, Debug)]
pub(crate) enum Purpose {
    /// A new tab in a workspace.
    Tab { scope: Scope, workspace: String },
    /// A new pane beside `at` in a tab.
    Split {
        tab: TabId,
        at: TerminalId,
        axis: Axis,
    },
}

/// One tab's panes as the window holds them. The tab's saved location
/// follows, written when a change settles rather than on every drag step.
#[derive(Debug)]
struct TabPanes {
    /// The pane keys go to while the tab has focus.
    active: TerminalId,
    /// A mirror tab only watches: it neither types nor splits.
    observe: bool,
    panes: Panes,
    unsaved: bool,
    /// How far the zoomed pane has grown over the tab; it shrinks back to
    /// its place when unzoomed.
    zoom: crate::motion::Toggle,
    /// The pane filling the tab, the rest hidden behind it, kept while it
    /// shrinks back.
    zoomed: Option<TerminalId>,
}

impl TabPanes {
    fn new(panes: Panes, observe: bool) -> Self {
        Self {
            active: panes.first().clone(),
            observe,
            panes,
            unsaved: false,
            zoom: crate::motion::Toggle::default(),
            zoomed: None,
        }
    }

    /// Zooms `terminal`, or unzooms, animating from where the zoom is now.
    fn set_zoom(&mut self, terminal: Option<TerminalId>) {
        let now = crate::motion::now();
        match terminal {
            Some(terminal) => {
                self.zoomed = Some(terminal);
                self.zoom.set(true, now, false);
            }
            None => self.zoom.set(false, now, false),
        }
    }

    /// The zoomed pane while it shows, and how much of the tab it fills.
    fn zoom_at(&self, now: std::time::Instant) -> Option<(&TerminalId, f32)> {
        let zoomed = self.zoomed.as_ref()?;
        (self.zoom.is_open() || self.zoom.moving(now)).then(|| (zoomed, self.zoom.shown(now)))
    }
}

/// The window's herdr-core state.
#[derive(Default)]
pub(crate) struct CoreTerminals {
    /// The shell shown when no Herdr 1 is installed.
    fallback: fallback::Fallback,
    /// A shown tab whose active pane should take the keyboard once its view
    /// exists.
    focus_wanted: Option<TabId>,
    pub(crate) hub: Hub,
    infos: infos::Infos,
    /// How the views draw now, to tell when the window's look changed.
    look: Option<Look>,
    /// A shell starting, and what for.
    starting: Option<(Purpose, Started)>,
    tabs: HashMap<TabId, TabPanes>,
    views: HashMap<(TabId, TerminalId), Entity<CoreTerminalView>>,
    /// A terminal to start once the hub connects: a shell, or the program
    /// its spawn names.
    wanted: Option<(Purpose, Spawn)>,
    workspace_ui: workspace_ui::State,
}

impl CoreTerminals {
    /// Every view on screen: the tabs' panes and the stand-in shell.
    fn shown(&self) -> impl Iterator<Item = &Entity<CoreTerminalView>> {
        let fallback = match &self.fallback {
            fallback::Fallback::Shown(view) => Some(view),
            _ => None,
        };
        self.views.values().chain(fallback)
    }

    /// Whether any shown terminal has output the window has not drawn.
    pub(crate) fn updated(&self, cx: &App) -> bool {
        self.shown().any(|view| view.read(cx).changed())
    }

    /// Whether a herdr-core terminal holds the keyboard.
    pub(crate) fn focused(&self, window: &Window, cx: &App) -> bool {
        self.shown()
            .any(|view| view.read(cx).focus_handle().contains_focused(window, cx))
    }

    /// What herdr-core reports of the terminals in the panes of tabs that
    /// drive them, which a chat may be drawn over. A mirror only watches.
    pub(crate) fn chat_terminals(&self) -> impl Iterator<Item = &TerminalInfo> {
        self.tabs
            .values()
            .filter(|shown| !shown.observe)
            .flat_map(|shown| shown.panes.terminals())
            .filter_map(|terminal| self.infos.get(terminal))
    }

    /// The pane a new chat in `tab` opens beside: the one in use, in a tab
    /// that drives its terminals.
    pub(crate) fn chat_split_at(&self, tab: TabId) -> Option<TerminalId> {
        self.tabs
            .get(&tab)
            .filter(|shown| !shown.observe)
            .map(|shown| shown.active.clone())
    }

    /// Shows `panes` as `tab`, a mirror when `observe`, with what herdr-core
    /// reports of `infos`, as a restored tab and a finished list would.
    #[cfg(test)]
    pub(crate) fn show_for_test(
        &mut self,
        tab: TabId,
        panes: Panes,
        observe: bool,
        infos: Vec<TerminalInfo>,
    ) {
        self.tabs.insert(tab, TabPanes::new(panes, observe));
        for info in infos {
            self.infos.insert(info);
        }
    }

    /// Whether `tab` is a herdr-core tab that drives its terminals, not a
    /// mirror that only watches.
    pub(crate) fn drives(&self, tab: TabId) -> bool {
        self.tabs.get(&tab).is_some_and(|shown| !shown.observe)
    }

    /// Makes `terminal` the pane in use in `tab`; whether that changed.
    pub(crate) fn set_active(&mut self, tab: TabId, terminal: TerminalId) -> bool {
        let Some(shown) = self.tabs.get_mut(&tab) else {
            return false;
        };
        if shown.active == terminal || !shown.panes.contains(&terminal) {
            return false;
        }
        shown.active = terminal;
        true
    }

    /// The tab whose panes hold `terminal`, among tabs that drive them.
    pub(crate) fn tab_of(&self, terminal: &TerminalId) -> Option<TabId> {
        self.tabs
            .iter()
            .find(|(_, shown)| !shown.observe && shown.panes.contains(terminal))
            .map(|(id, _)| *id)
    }

    /// The tab and pane whose view holds the keyboard.
    pub(crate) fn focused_pane(&self, window: &Window, cx: &App) -> Option<(TabId, TerminalId)> {
        self.views
            .iter()
            .find(|(_, view)| view.read(cx).focus_handle().is_focused(window))
            .map(|(key, _)| key.clone())
    }
}

impl SheprdWindow {
    /// Whether a new tab runs in herdr-core. The window's core is on this
    /// computer, so a tab on another host stays with that host's Herdr, where
    /// its shell starts on the host and in the workspace's folder.
    pub(crate) fn new_tabs_use_core(&self) -> bool {
        self.selected_core_group().is_some()
            || (self.config.herdr_2_terminals && !self.selected_is_remote())
    }

    /// The `new_core_terminal` command: starts a shell in herdr-core and
    /// shows it in a tab of the focused workspace. With no core running, the
    /// separately installed `herdr-core` starts. Refused on another
    /// host, where the shell would run on this computer instead.
    pub(crate) fn new_core_terminal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_core_group().is_none() && self.selected_is_remote() {
            self.show_flash(Flash::warning(fl!("core-terminals-local-only")), cx);
            return;
        }
        let Some((scope, workspace)) = self.active_browser_key(cx) else {
            self.show_flash(Flash::warning(fl!("core-terminals-open-workspace")), cx);
            return;
        };
        if let Some(key) = self.selected_core_group()
            && self.core_terminals.hub.backend().is_none_or(|backend| {
                backend.core().is_lost() || backend.core().hello().core.id != key.hub_core_id()
            })
        {
            self.show_flash(Flash::warning(fl!("core-workspace-stale")), cx);
            return;
        }
        self.dismiss_menu(window, cx);
        self.start_core_program(Purpose::Tab { scope, workspace }, Spawn::default());
    }

    /// Starts `spawn` in herdr-core for `purpose`, once the hub connects.
    pub(crate) fn start_core_program(&mut self, purpose: Purpose, spawn: Spawn) {
        self.core_terminals.wanted = Some((purpose, spawn));
        self.core_terminals.hub.connect(Start::IfMissing);
        self.start_core_shell();
    }

    /// Starts the wanted shell once the hub is connected, in the folder of
    /// the pane it splits when herdr-core knows it.
    fn start_core_shell(&mut self) -> bool {
        let terminals = &mut self.core_terminals;
        if terminals.starting.is_some() {
            return false;
        }
        let Some(backend) = terminals.hub.backend().cloned() else {
            return false;
        };
        let wrong_core = terminals.wanted.as_ref().is_some_and(|(purpose, _)| {
            !purpose_matches_core(purpose, &backend.core().hello().core.id)
        });
        if wrong_core {
            terminals.wanted = None;
            tracing::warn!("Refused Core tab spawn after Core identity changed");
            return true;
        }
        let Some((purpose, mut spawn)) = terminals.wanted.take() else {
            return false;
        };
        if spawn.cwd.is_none()
            && let Purpose::Split { at, .. } = &purpose
        {
            spawn.cwd = terminals.infos.get(at).and_then(|info| info.cwd.clone());
        }
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        let spawned = std::thread::Builder::new()
            .name("herdr-core-spawn".into())
            .spawn(move || {
                let _ = send.send(backend.spawn(&spawn));
            });
        match spawned {
            Ok(_) => terminals.starting = Some((purpose, receive)),
            Err(error) => tracing::warn!(%error, "Could not start a herdr-core spawn worker"),
        }
        false
    }

    /// Runs every window tick: joins Core for local metadata and restored tabs,
    /// places a started terminal, gives each shown pane its view, and takes new screens.
    pub(crate) fn poll_core_terminals(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.endpoints.iter().any(|endpoint| {
            endpoint.id == crate::endpoint::LOCAL
                && matches!(endpoint.connection.target, ConnectTarget::Local)
        }) {
            // A plain local window reads an already-running Core's metadata.
            // Joining cannot start a runtime or change the legacy endpoint.
            self.core_terminals.hub.connect(Start::Join);
        }
        let connected = self.core_terminals.hub.backend().is_some();
        if self.core_terminals.hub.poll() {
            // A lost connection ended every pane's worker. Their views go, so
            // the next sync joins the core again and attaches each terminal
            // anew; nothing is started or replayed. A pane that held the
            // keyboard has its tab take it back once the new view is there.
            if connected && self.core_terminals.hub.backend().is_none() {
                if let Some((tab, _)) = self.core_terminals.focused_pane(window, cx) {
                    self.core_terminals.focus_wanted = Some(tab);
                }
                self.core_terminals.views.clear();
            }
            if let State::Unavailable(error) = self.core_terminals.hub.state()
                && self.core_terminals.wanted.take().is_some()
            {
                let reason = error.to_string();
                self.show_flash(
                    Flash::warning(fl!("core-terminals-unavailable", reason = reason)),
                    cx,
                );
            }
            if self.start_core_shell() {
                self.show_flash(Flash::warning(fl!("core-workspace-stale")), cx);
            }
            cx.notify();
        }
        self.place_started_core_terminal(window, cx);
        self.sync_core_views(cx);
        if let Some(tab) = self.core_terminals.focus_wanted
            && self.core_pane_view(tab).is_some()
        {
            // Focus the user gave something else while the pane waited, such
            // as a picker's search field, is newer than the wish: keep it.
            if !self.group_shows_page(tab, cx) {
                self.core_terminals.focus_wanted = None;
            } else if keyboard_free(window.focused(cx).as_ref(), &self.focus) {
                self.focus_core_terminal(tab, window, cx);
            } else {
                self.core_terminals.focus_wanted = None;
            }
        }
        self.poll_core_fallback(cx);
        self.refresh_core_looks(cx);
        if let Some(view) = self.core_fallback_view() {
            view.update(cx, |view, cx| view.poll(cx));
        }
        self.poll_core_panes(window, cx);
        // One app-wide subscription supplies the chrome in every window.
        if !cx.has_global::<observation::Observation>() {
            cx.set_global(observation::Observation::default());
        }
        let backend = self.core_terminals.hub.backend().cloned();
        let observation = cx.global_mut::<observation::Observation>();
        let before = observation.projection().revision();
        observation.poll();
        if let Some(backend) = backend {
            observation.ensure(backend);
        }
        let projection = observation.projection();
        let info_changed = self.core_terminals.infos.sync(
            projection.revision(),
            projection.terminals(),
            projection.fresh(),
        );
        let changed = before != projection.revision() || info_changed;
        if changed {
            cx.notify();
        }
        self.clear_stale_core_group(cx);
        self.poll_core_workspace_edit(window, cx);
        if !cx.has_active_drag() {
            self.save_core_panes(cx);
        }
        // A zoom in motion draws a frame each tick until it lands.
        let now = crate::motion::now();
        if self
            .core_terminals
            .tabs
            .values()
            .any(|tab| tab.zoom.moving(now))
        {
            cx.notify();
        }
    }

    fn place_started_core_terminal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((_, started)) = &self.core_terminals.starting else {
            return;
        };
        let outcome = match started.try_recv() {
            Ok(outcome) => outcome,
            Err(std::sync::mpsc::TryRecvError::Empty) => return,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.core_terminals.starting = None;
                return;
            }
        };
        let Some((purpose, _)) = self.core_terminals.starting.take() else {
            return;
        };
        let info = match outcome {
            // Known at once, so a chat it labels attaches without waiting
            // for the next list.
            Ok(info) => self.core_terminals.infos.insert(info),
            Err(error) => {
                let reason = error.to_string();
                self.show_flash(
                    Flash::warning(fl!("core-terminals-start-failed", reason = reason)),
                    cx,
                );
                return;
            }
        };
        match purpose {
            Purpose::Tab { scope, workspace } => {
                let location = Location::CoreTerminal {
                    panes: Panes::single(info.id),
                    observe: false,
                };
                let Some(id) = Store::update(cx, |store| {
                    store.open(scope, &workspace, Some(location), None)
                }) else {
                    self.show_flash(Flash::warning(fl!("core-terminals-too-many-tabs")), cx);
                    return;
                };
                self.sync_core_views(cx);
                self.show_browser_tab(id, window, cx);
            }
            Purpose::Split { tab, at, axis } => {
                self.place_split(tab, &at, axis, info.id, window, cx)
            }
        }
        cx.notify();
    }

    /// Gives a herdr-core tab's active pane the keyboard as the tab is
    /// shown, or as soon as its view exists for a restored tab still waiting
    /// on the hub.
    pub(crate) fn focus_core_terminal(&mut self, tab: TabId, window: &mut Window, cx: &mut App) {
        self.core_terminals.focus_wanted = None;
        // A pane drawn as a chat gives its chat the keyboard.
        let chat = self
            .core_terminals
            .tabs
            .get(&tab)
            .and_then(|shown| self.core_chat_focus(tab, &shown.active, window, cx));
        if let Some(focus) = chat {
            window.focus(&focus, cx);
            return;
        }
        match self.core_pane_view(tab) {
            Some(view) => window.focus(&view.read(cx).focus_handle().clone(), cx),
            None => self.core_terminals.focus_wanted = Some(tab),
        }
    }

    /// The view of `tab`'s active pane.
    fn core_pane_view(&self, tab: TabId) -> Option<&Entity<CoreTerminalView>> {
        let active = &self.core_terminals.tabs.get(&tab)?.active;
        self.core_terminals.views.get(&(tab, active.clone()))
    }

    /// How the window draws terminals, for its herdr-core views.
    pub(crate) fn core_look(&self) -> Look {
        Look {
            background: self.fill().base(self.terminal_theme().background),
            cell_height: self.config.terminal.line_height(),
            cell_width: self.cell_width,
            font: self.config.terminal.font(),
            ground: self.ground(),
            option_as_alt: self.config.option_as_alt,
            padding: self.config.terminal_padding,
        }
    }

    /// Keeps every view drawing as the window does after a font, theme, or
    /// option change.
    fn refresh_core_looks(&mut self, cx: &mut Context<Self>) {
        if self.core_terminals.shown().next().is_none() {
            return;
        }
        let look = self.core_look();
        if self.core_terminals.look.as_ref() == Some(&look) {
            return;
        }
        let views: Vec<Entity<CoreTerminalView>> = self.core_terminals.shown().cloned().collect();
        for view in views {
            view.update(cx, |view, cx| view.set_look(look.clone(), cx));
        }
        self.core_terminals.look = Some(look);
    }
}

/// Whether a pane waiting for the keyboard may take it: nothing holds it, or
/// only the window itself does.
fn keyboard_free(focused: Option<&FocusHandle>, window: &FocusHandle) -> bool {
    focused.is_none_or(|focused| focused == window)
}

fn purpose_matches_core(purpose: &Purpose, actual_core_id: &str) -> bool {
    let Purpose::Tab { scope, workspace } = purpose else {
        return true;
    };
    let Scope::Core(_) = scope else {
        return true;
    };
    let Some(key) = scope.core_group(workspace) else {
        return false;
    };
    CoreGroupKey::new(
        actual_core_id,
        None,
        workspace_document::NAME,
        key.group_id(),
    )
    .as_ref()
        == Some(&key)
}
