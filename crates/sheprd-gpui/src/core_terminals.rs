// ABOUTME: Owns Core terminal presentation and worker coordination.
// ABOUTME: Refuses Core work when the application session has not opted in.
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
pub(crate) use hub::{Hub, Mismatch, Start, State};
pub(crate) use view::{CoreTerminalView, Look};

pub(crate) fn core_group_workspace<'a>(
    key: &CoreGroupKey,
    cx: &'a App,
) -> Option<&'a workspace_document::Workspace> {
    let projection = cx.try_global::<observation::Observation>()?.projection();
    if !projection.fresh() {
        return None;
    }
    if &projection.group_key(key.group_id())? != key {
        return None;
    }
    projection
        .document()?
        .value
        .workspaces()
        .iter()
        .find(|row| row.id == key.group_id())
}

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

/// What one window keeps of a tab's panes: the one in use and the one that
/// fills the tab. The tree itself, and whether the tab only watches, belong
/// to the app's tab [`Store`], which every window reads and edits.
#[derive(Debug)]
struct TabPanes {
    /// The pane keys go to while the tab has focus.
    active: TerminalId,
    /// How far the zoomed pane has grown over the tab; it shrinks back to
    /// its place when unzoomed.
    zoom: crate::motion::Toggle,
    /// The pane filling the tab, the rest hidden behind it, kept while it
    /// shrinks back.
    zoomed: Option<TerminalId>,
}

impl TabPanes {
    fn new(active: TerminalId) -> Self {
        Self {
            active,
            zoom: crate::motion::Toggle::default(),
            zoomed: None,
        }
    }

    /// Lets go of a pane that left `panes`, as when another window closed
    /// it: the first pane is in use then, and nothing fills the tab.
    fn follow(&mut self, panes: &Panes) {
        if !panes.contains(&self.active) {
            self.active = panes.first().clone();
        }
        if self
            .zoomed
            .as_ref()
            .is_some_and(|zoomed| !panes.contains(zoomed))
        {
            self.zoomed = None;
            self.zoom = crate::motion::Toggle::default();
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

/// `tab`'s pane tree and whether it only watches, as the app's store holds
/// them.
fn layout(tab: TabId, cx: &App) -> Option<(&Panes, bool)> {
    match &cx.try_global::<Store>()?.get(tab)?.location {
        Some(Location::CoreTerminal { panes, observe }) => Some((panes, *observe)),
        _ => None,
    }
}

/// The window's herdr-core state.
#[derive(Default)]
pub(crate) struct CoreTerminals {
    /// A split's line being dragged and where it is held. The store takes
    /// the share once the drag ends, so a drag saves once.
    dragging: Option<(split::SplitDrag, panes::Share)>,
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
    pub(crate) fn new(enabled: bool) -> Self {
        Self {
            hub: Hub::new(enabled),
            ..Self::default()
        }
    }

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

    /// The pane in use and the tree of a tab this window shows and drives;
    /// a mirror only watches.
    fn driven<'a>(&'a self, tab: TabId, cx: &'a App) -> Option<(&'a TerminalId, &'a Panes)> {
        let shown = self.tabs.get(&tab)?;
        let (panes, observe) = layout(tab, cx)?;
        // Another window may have closed the pane since this one's last tick.
        let active = match panes.contains(&shown.active) {
            true => &shown.active,
            false => panes.first(),
        };
        (!observe).then_some((active, panes))
    }

    /// What herdr-core reports of the terminals in the panes of tabs that
    /// drive them, which a chat may be drawn over. A mirror only watches.
    pub(crate) fn chat_terminals<'a>(
        &'a self,
        cx: &'a App,
    ) -> impl Iterator<Item = &'a TerminalInfo> {
        self.tabs
            .keys()
            .filter_map(|tab| self.driven(*tab, cx))
            .flat_map(|(_, panes)| panes.terminals())
            .filter_map(|terminal| self.infos.get(terminal))
    }

    /// The pane a new chat in `tab` opens beside: the one in use, in a tab
    /// that drives its terminals.
    pub(crate) fn chat_split_at(&self, tab: TabId, cx: &App) -> Option<TerminalId> {
        self.driven(tab, cx).map(|(active, _)| active.clone())
    }

    /// Shows a new tab of `panes`, a mirror when `observe`, with what
    /// herdr-core reports of `infos`, as a restored tab and a finished list
    /// would.
    #[cfg(test)]
    #[allow(clippy::unwrap_used)]
    pub(crate) fn show_for_test(
        &mut self,
        panes: Panes,
        observe: bool,
        infos: Vec<TerminalInfo>,
        cx: &mut App,
    ) -> TabId {
        let active = panes.first().clone();
        let location = Location::CoreTerminal { panes, observe };
        let tab = Store::update(cx, |store| {
            store.open(Scope::endpoint("test"), "w", Some(location), None)
        })
        .unwrap();
        self.tabs.insert(tab, TabPanes::new(active));
        for info in infos {
            self.infos.insert(info);
        }
        tab
    }

    /// Whether `tab` is a herdr-core tab that drives its terminals, not a
    /// mirror that only watches.
    pub(crate) fn drives(&self, tab: TabId, cx: &App) -> bool {
        self.driven(tab, cx).is_some()
    }

    /// Makes `terminal` the pane in use in `tab`; whether that changed.
    pub(crate) fn set_active(&mut self, tab: TabId, terminal: TerminalId, cx: &App) -> bool {
        let Some(shown) = self.tabs.get_mut(&tab) else {
            return false;
        };
        let held = layout(tab, cx).is_some_and(|(panes, _)| panes.contains(&terminal));
        if shown.active == terminal || !held {
            return false;
        }
        shown.active = terminal;
        true
    }

    /// The tab whose panes hold `terminal`, among tabs that drive them.
    pub(crate) fn tab_of(&self, terminal: &TerminalId, cx: &App) -> Option<TabId> {
        self.tabs.keys().copied().find(|tab| {
            self.driven(*tab, cx)
                .is_some_and(|(_, panes)| panes.contains(terminal))
        })
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
    pub(crate) fn core_enabled(&self) -> bool {
        self.core_terminals.hub.enabled()
    }

    /// Whether a new tab runs in herdr-core. The window's core is on this
    /// computer, so a tab on another host stays with that host's Herdr, where
    /// its shell starts on the host and in the workspace's folder.
    pub(crate) fn new_tabs_use_core(&self) -> bool {
        self.core_enabled()
            && (self.selected_core_group().is_some()
                || (self.config.herdr_2_terminals && !self.selected_is_remote()))
    }

    /// The `new_core_terminal` command: starts a shell in herdr-core and
    /// shows it in a tab of the focused workspace. With no core running, the
    /// separately installed `herdr-core` starts. Refused on another
    /// host, where the shell would run on this computer instead.
    pub(crate) fn new_core_terminal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.core_enabled() {
            self.show_flash(Flash::warning(crate::core_feature::disabled_message()), cx);
            return;
        }
        if self.selected_core_group().is_none() && self.selected_is_remote() {
            self.show_flash(Flash::warning(fl!("core-terminals-local-only")), cx);
            return;
        }
        let Some((scope, workspace)) = self.active_browser_key(cx) else {
            self.show_flash(Flash::warning(fl!("core-terminals-open-workspace")), cx);
            return;
        };
        if let Some(key) = self.selected_core_group() {
            match self.core_terminals.hub.backend_for(key.hub_core_id()) {
                Ok(_) => {}
                // A group's terminal starts only on its own live Core; a
                // window not joined yet does not join from here.
                Err(Mismatch::Absent | Mismatch::Lost | Mismatch::Other) => {
                    self.show_flash(Flash::warning(fl!("core-workspace-stale")), cx);
                    return;
                }
            }
        }
        self.dismiss_menu(window, cx);
        self.start_core_program(Purpose::Tab { scope, workspace }, Spawn::default());
    }

    /// Starts `spawn` in herdr-core for `purpose`, once the hub connects.
    pub(crate) fn start_core_program(&mut self, purpose: Purpose, spawn: Spawn) {
        if !self.core_enabled() {
            return;
        }
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
        if !self.core_enabled() {
            return;
        }
        if self.selected_core_group().is_some()
            || self.endpoints.iter().any(|endpoint| {
                endpoint.id == crate::endpoint::LOCAL
                    && matches!(endpoint.connection.target, ConnectTarget::Local)
            })
        {
            // A selected Core group has its own exact hub identity. A plain
            // local window may also read an already-running Core's metadata.
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
        // A pane another window closed takes its view with it; the tab's
        // pane in use gets the keyboard it held.
        let orphaned = self
            .core_terminals
            .focused_pane(window, cx)
            .filter(|(tab, pane)| layout(*tab, cx).is_some_and(|(panes, _)| !panes.contains(pane)))
            .map(|(tab, _)| tab);
        self.sync_core_views(cx);
        if let Some(tab) = orphaned {
            self.focus_core_terminal(tab, window, cx);
        }
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
            self.drop_core_split(cx);
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
