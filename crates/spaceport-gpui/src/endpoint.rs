//! GUI-owned endpoint catalog and connection lifetimes. Each attempt has its own
//! inbox, so retired workers can never publish into a replacement connection.
// i18n: migrated
use super::{
    LiveState, NavigationTarget, SpaceportWindow, WheelAccumulator, connection::ConnectionBridge,
};
use crate::i18n::{Localize, fl};
use crate::state::activation::TIMEOUT as ACTIVATION_TIMEOUT;
use gpui::Context;
use spaceport_client::{ClientHandle, ConnectOptions, ConnectTarget, VersionMismatch};
use std::{
    collections::HashSet,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

mod catalog;
mod config_reload;
mod error;
pub(crate) mod fence;
#[cfg(test)]
mod fixtures;
mod follow;
mod order;
pub(crate) mod origin;
mod polling;
mod wake;

pub use error::Error;
// What only the socket-driving fixtures below name: the daemon's status and SSH
// refusal they install, and the link curve's own bounds they time against.
#[cfg(test)]
pub(crate) use crate::state::ConnectionStatus;
#[cfg(all(test, unix))]
pub(crate) use spaceport_client::SshFailure;
#[cfg(all(test, unix))]
pub(crate) use spaceport_endpoint::{MAX_RETRY_DELAY, STABLE_CONNECTION_PERIOD};
#[cfg(all(test, unix))]
pub(crate) use std::time::Duration;
// The endpoint program below GPUI, reached through the paths this module has
// always named it by: the policy stage 1 and 2 lowered, and the host the link
// machine and the live state live in.
pub(crate) use spaceport_endpoint::{
    Action, Catalog, History, Host, LOCAL, Phase, Step, Transition, Transport, WSL_PREFIX,
    WakeClock, focus_requested, saved_profile_id, watch_network,
};

/// How much of the window an update changes. Ordered, so several updates
/// combine into the widest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Redraw {
    #[default]
    None,
    /// Only the terminal surface: the sidebar keeps its last layout.
    Terminal,
    Window,
}

pub(super) struct Release {
    inbox: Arc<Mutex<LiveState>>,
    drained: Arc<AtomicBool>,
    phase: ReleasePhase,
    boot: String,
}

enum ReleasePhase {
    Deferred(ClientHandle),
    Sent(String),
    Disconnecting,
}

impl Release {
    fn resolved(&mut self) -> bool {
        // Disconnect() requests shutdown; only the event receiver closing proves
        // that this generation's transport is gone. Catalog removal alone is not
        // sufficient evidence to let another surface take ownership.
        if self.drained.load(Ordering::Acquire) {
            return true;
        }
        let Ok(mut state) = self.inbox.try_lock() else {
            return false;
        };
        if let ReleasePhase::Deferred(handle) = &self.phase {
            // Queue and register under the same lock, but never wait for that lock
            // on the UI thread. The destination stays fenced until acknowledgement.
            if !state.status.is_connected()
                || state
                    .snapshot
                    .as_ref()
                    .is_none_or(|s| s.boot_id != self.boot)
            {
                handle.disconnect();
                self.phase = ReleasePhase::Disconnecting;
                return false;
            }
            state.set_outer_focus(false);
            state.invalidate_surface();
            state.dirty = true;
            let result = handle
                .set_focus(&self.boot, false)
                .and_then(|()| handle.set_surface_active(&self.boot, false));
            match result {
                Ok(request) => {
                    state
                        .activation
                        .transition(crate::state::activation::Transition::Queued {
                            result: Ok(request.clone()),
                            boot: self.boot.clone(),
                            focus: None,
                            active: false,
                            now: Instant::now(),
                        });
                    self.phase = ReleasePhase::Sent(request);
                }
                // A full queue proves neither request entered the ordered queue,
                // which is transient backpressure rather than a dead transport:
                // the release keeps its destination fenced and retries next poll,
                // exactly as an active barrier does.
                Err(spaceport_client::Error::Full) => {}
                Err(_) => {
                    handle.disconnect();
                    self.phase = ReleasePhase::Disconnecting;
                }
            }
        }
        match &self.phase {
            ReleasePhase::Sent(request) => state.activation.request().is_some_and(|a| {
                a.request == *request
                    && a.boot == self.boot
                    && !a.active
                    && !state.activation.failed()
                    && state.activation.revision().is_some()
            }),
            _ => false,
        }
    }
}

pub(super) struct Endpoint {
    pub id: String,
    pub label: String,
    pub connection: ConnectionBridge,
    /// The link machine and the live state it is read from, below GPUI: seen
    /// through the readings and performers below, and never entering a view.
    pub(crate) host: Host,
    /// The target the saved entry named when this endpoint was last reconciled
    /// against it. Its own session may have been picked in the sessions list
    /// since, so a catalog change can only be told from such a pick by
    /// remembering what the catalog said.
    saved: Option<ConnectTarget>,
    pub collapsed: bool,
    pub collapsed_repos: HashSet<String>,
    /// Each tab's last frame before `live()` moved on, for Go To's preview.
    pub(crate) last_seen: crate::palette::LastSeen,
    /// Back and Forward through the panes `live()` has focused.
    pub(crate) history: History,
    pub generation: u64,
    pub(crate) toasts: crate::notifications::Toasts,
    /// Derived from `live().snapshot`; refreshed by `sync_live` whenever
    /// `live()` changes.
    pub(crate) config_diagnostic: crate::config_diagnostic::ConfigDiagnostic,
    /// A sleep or network change this endpoint has not yet recovered from;
    /// see [`SpaceportWindow::recover_connections`].
    recovery: Option<wake::Trigger>,
    initial_surface: bool,
    /// A `server.reload_config` waiting for this connection's surface; see
    /// [`Self::reload_config`]. It outlives a reconnection, since a dropped
    /// link can come back to the same daemon process.
    config_reload_pending: bool,
    /// A focus asked for on this host since the window last looked; see
    /// [`SpaceportWindow::follow_focus`].
    focus_requested: bool,
    /// When the window last stopped showing this host.
    left_at: Option<Instant>,
    sounds: spaceport_sound::Policy,
}

// The rules below are the crate host's: the app reads them through the endpoint
// it holds, so no caller spells a path into the link machine.
impl Endpoint {
    pub(crate) fn live(&self) -> &LiveState {
        self.host.live()
    }

    pub(crate) fn live_mut(&mut self) -> &mut LiveState {
        self.host.live_mut()
    }

    /// The stored lifecycle, independent of a window's older worker projection.
    pub(crate) fn phase(&self) -> Phase {
        self.host.phase()
    }

    /// The endpoint's state as `herdr-gpui` commands report it, in English.
    pub fn status(&self) -> &'static str {
        self.phase().status()
    }

    /// The endpoint's state as the device picker shows it.
    pub fn status_label(&self) -> String {
        fl!("endpoint-status", state = self.phase().as_str())
    }

    pub(crate) fn enabled(&self) -> bool {
        self.host.enabled()
    }

    pub(super) fn detached(&self) -> bool {
        self.host.detached()
    }

    pub(super) fn retry_at(&self) -> Instant {
        self.host.retry_at()
    }

    pub(crate) fn herdr_missing(&self) -> bool {
        self.host.herdr_missing()
    }

    pub(crate) fn herdr_stopped(&self) -> bool {
        self.host.herdr_stopped()
    }

    pub(crate) fn version_mismatch(&self) -> Option<&VersionMismatch> {
        self.host.version_mismatch()
    }

    pub(crate) fn outage(&self) -> Option<&str> {
        self.host.outage()
    }

    #[cfg(all(test, unix))]
    pub(super) fn retry_delay(&self) -> Duration {
        self.host.retry_delay()
    }

    /// Whether a live transport has delivered a snapshot.
    pub(crate) fn serving(&self) -> bool {
        self.host.serving(self.transport())
    }

    /// Whether the endpoint wants a connection and has no transport.
    pub(super) fn may_dial(&self) -> bool {
        self.host.may_dial(self.connection.retry_ready())
    }

    pub fn surface_requested(&self) -> bool {
        self.initial_surface
    }

    /// What the link machine needs of this endpoint's socket, which it does not
    /// hold: whether an attempt has a handle, whether that handle's worker is
    /// still there, and whether this host's daemon could be asked again.
    fn transport(&self) -> Transport {
        Transport {
            handled: self.connection.handle.is_some(),
            alive: self
                .connection
                .handle
                .as_ref()
                .is_some_and(|handle| !handle.is_disconnected()),
            can_restart: crate::daemon::may_start(&self.connection.target)
                || matches!(
                    self.connection.target,
                    ConnectTarget::Ssh { .. } | ConnectTarget::Wsl { .. }
                ),
        }
    }

    /// Enters `transition` on the host and performs what it comes back with.
    fn enter(&mut self, transition: Transition<'_>) {
        let action = self.host.enter(transition);
        self.perform(action);
    }

    /// Performs what the link machine asked of this endpoint's socket. The
    /// window owns the transport, the toasts and the connection identity, so
    /// each of those is done here and nowhere below GPUI.
    fn perform(&mut self, action: Action) {
        match action {
            Action::None => {}
            Action::CheckLiveness => {
                if let Some(handle) = &self.connection.handle {
                    handle.check_liveness();
                }
            }
            Action::Detach => self.retire_transport(),
            Action::Retire { target } => {
                self.retire_transport();
                *self.host.live_mut() = LiveState::default();
                self.connection = ConnectionBridge::new(target);
                self.recovery = None;
            }
        }
    }

    /// What stopping an attempt leaves of the window's own facts: nothing queued
    /// for the toast list, no surface asked for, and a new connection identity.
    fn stop_side(&mut self) {
        self.toasts.entries.clear();
        self.initial_surface = false;
        self.focus_requested = false;
        self.generation += 1;
    }

    /// Retires the transport and takes the state it left.
    fn retire_transport(&mut self) {
        self.stop_side();
        self.connection.detach(false);
        self.take_live();
        self.host.retired();
    }

    /// Takes whatever the bridge holds, which a retirement has just replaced.
    fn take_live(&mut self) {
        *self.host.live_mut() = self.connection.take_update().unwrap_or_default();
    }

    /// Trades this endpoint's connection, and its projection, for another
    /// client of the same daemon that said hello with an active surface, as
    /// an editor group's own connection does. The caller keeps the one this
    /// endpoint had.
    pub(crate) fn trade_connection(
        &mut self,
        connection: &mut ConnectionBridge,
        live: &mut LiveState,
    ) {
        std::mem::swap(&mut self.connection, connection);
        std::mem::swap(self.host.live_mut(), live);
        self.initial_surface = true;
        self.enter(Transition::Trade(self.transport()));
        self.sync_live();
    }

    /// Refreshes state derived from `live` after it is replaced.
    pub(crate) fn sync_live(&mut self) {
        self.enter(Transition::Observe {
            now: Instant::now(),
            transport: self.transport(),
        });
        self.sync_live_projection();
    }

    fn sync_live_projection(&mut self) {
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        if let Some(warning) = self.host.live_mut().local_peer_warning.take() {
            self.toasts
                .receive([crate::daemon::warning::notice(&warning, Instant::now())]);
        }
        // Without surface support there is no navigation fence to watch, so
        // a travel waits for the next focus change instead.
        let navigating = self.host.live().snapshot.is_some()
            && (self.host.live().activation_pending() || !self.host.live().supports_surface);
        self.history
            .observe(self.host.live().snapshot.as_deref(), navigating);
        self.config_diagnostic.sync(
            self.host
                .live()
                .snapshot
                .as_deref()
                .and_then(|snapshot| snapshot.config_diagnostic.as_deref()),
        );
    }

    pub fn new(id: String, label: String, target: ConnectTarget, enabled: bool) -> Self {
        Self {
            id,
            label,
            connection: ConnectionBridge::new(target),
            host: Host::new(enabled),
            saved: None,
            collapsed: false,
            collapsed_repos: HashSet::new(),
            last_seen: Default::default(),
            history: Default::default(),
            generation: 0,
            toasts: Default::default(),
            config_diagnostic: Default::default(),
            recovery: None,
            initial_surface: false,
            config_reload_pending: false,
            focus_requested: false,
            left_at: None,
            sounds: Default::default(),
        }
    }

    fn stop(&mut self) {
        self.enter(Transition::Stop);
        self.sync_live();
    }

    /// The SSH target and session this device was saved with. The sessions list
    /// may have pointed the live connection at another of the host's sessions,
    /// so whatever speaks for the saved device (duplicate checks, the device's
    /// own menu) reads this instead. One never reconciled against the catalog
    /// has only its live target to go on.
    pub(crate) fn saved_ssh(&self) -> Option<(&str, &str)> {
        match self.saved_target() {
            ConnectTarget::Ssh { target, session } => Some((target, session)),
            _ => None,
        }
    }

    /// The target this device was saved with, SSH or WSL alike; see
    /// [`Self::saved_ssh`].
    pub(crate) fn saved_target(&self) -> &ConnectTarget {
        self.saved.as_ref().unwrap_or(&self.connection.target)
    }

    /// Point this endpoint at another target, retiring the old transport. The
    /// endpoint keeps its identity, label, and sidebar state; nothing the old
    /// connection produced survives it.
    fn retarget(&mut self, target: ConnectTarget, enabled: bool) {
        self.enter(Transition::Retarget { target, enabled });
        self.sync_live();
    }

    fn connect(&mut self, options: ConnectOptions, active: bool) {
        // The link machine records the attempt and the backoff it earns; the
        // dial itself belongs to the window that holds the options.
        self.enter(Transition::Connect);
        self.stop_side();
        self.initial_surface = active;
        self.connection
            .reconnect(options, false, active, crate::daemon::Start::WhenMissing);
        self.take_live();
        self.sync_live();
        self.toasts
            .receive(self.host.live_mut().notifications.drain(..));
    }

    fn poll(&mut self, now: Instant) -> Redraw {
        let mut changed = Redraw::None;
        if let Some(mut state) = self.connection.take_update() {
            changed = if self.host.live().only_surface_changed(&state) {
                Redraw::Terminal
            } else {
                Redraw::Window
            };
            if state.notifications_lost
                || !state.status.is_connected()
                || self
                    .host
                    .live()
                    .snapshot
                    .as_ref()
                    .zip(state.snapshot.as_ref())
                    .is_some_and(|(old, new)| old.boot_id != new.boot_id)
            {
                self.toasts.entries.clear();
            }
            self.toasts.receive(state.notifications.drain(..));
            self.last_seen.observe(
                (
                    self.host.live().snapshot.as_deref(),
                    self.host.live().surface.as_ref(),
                ),
                (state.snapshot.as_deref(), state.surface.as_ref()),
                crate::palette::unix_now(),
            );
            self.observe_focus(&state, now);
            self.enter(Transition::Update {
                state: &mut state,
                now,
                transport: self.transport(),
            });
            self.sync_live_projection();
        }
        if self
            .connection
            .handle
            .as_ref()
            .is_some_and(ClientHandle::is_disconnected)
        {
            self.connection.handle = None;
            self.enter(Transition::Lost {
                now,
                lost: fl!("endpoint-connection-lost"),
            });
            changed = Redraw::Window;
        }
        if let Some(trigger) = self.recovery
            && self.recover(trigger, now)
        {
            self.recovery = None;
        }
        self.enter(Transition::Observe {
            now,
            transport: self.transport(),
        });
        changed
    }

    /// After a sleep or a network change, once this poll has taken what the
    /// worker reported; see [`Host::recover`] for the rules. Returns false, to
    /// be called again next poll, when the worker's report cannot be read
    /// without blocking.
    fn recover(&mut self, trigger: wake::Trigger, now: Instant) -> bool {
        let action = self.host.recover(
            trigger,
            now,
            &self.connection.target,
            self.transport(),
            || self.connection.reports_connected(),
        );
        match action {
            None => false,
            Some(action) => {
                self.perform(action);
                true
            }
        }
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        self.stop();
    }
}

impl SpaceportWindow {
    /// Move every matching live target in this window away from a session that
    /// the user confirmed for deletion. This retires transports without I/O on
    /// the UI thread. The caller reconnects the selected endpoint if it moved.
    pub(super) fn retarget_session_for_deletion(&mut self, target: &ConnectTarget) -> bool {
        let mut selected_changed = false;
        for (index, endpoint) in self.endpoints.iter_mut().enumerate() {
            let replacement = match target {
                ConnectTarget::Session {
                    name,
                    development: false,
                } if name != "default"
                    && index == 0
                    && target.socket_path().ok().is_some_and(|path| {
                        endpoint.connection.target.socket_path().ok() == Some(path)
                    }) =>
                {
                    Some(ConnectTarget::Session {
                        name: "default".into(),
                        development: false,
                    })
                }
                _ if target
                    .remote_session()
                    .is_some_and(|session| session != "default")
                    && endpoint.connection.target == *target =>
                {
                    target.remote_host().map(|host| host.target("default"))
                }
                _ => None,
            };
            if let Some(replacement) = replacement {
                let enabled = endpoint.enabled();
                endpoint.retarget(replacement, enabled);
                selected_changed |= index == self.selected_endpoint;
            }
        }
        selected_changed
    }

    pub(super) fn reconnect(&mut self, cx: &mut Context<Self>) {
        let index = self.selected_endpoint;
        if !self.endpoints[index].enabled() {
            return;
        }
        self.version_notice_shown = false;
        self.endpoints[index].enter(Transition::ResetAttempts);
        self.endpoints[index].connect(self.options, index == 0);
        self.reset_selected(cx);
    }

    pub(super) fn detach_endpoint(&mut self, cx: &mut Context<Self>) {
        let endpoint = &mut self.endpoints[self.selected_endpoint];
        endpoint.enter(Transition::Detach);
        endpoint.sync_live_projection();
        self.reset_selected(cx);
    }

    /// The current state of endpoint `index`: the window's own copy for the
    /// selected endpoint, the endpoint's otherwise. `reset_selected` copies
    /// the endpoint's state into the window's; from then on only the window's
    /// copy follows the selected connection.
    ///
    /// Panics when `index` is not an endpoint, as indexing `endpoints` does.
    pub(crate) fn live_of(&self, index: usize) -> &LiveState {
        spaceport_endpoint::live_of(self.selected_endpoint, &self.live, index, |index| {
            &self.endpoints[index].host
        })
    }

    /// Every endpoint with its current state, in `self.endpoints` order.
    pub(crate) fn hosts(&self) -> impl Iterator<Item = (usize, &Endpoint, &LiveState)> {
        self.endpoints
            .iter()
            .enumerate()
            .map(|(index, endpoint)| (index, endpoint, self.live_of(index)))
    }

    pub(super) fn reset_selected(&mut self, cx: &mut Context<Self>) {
        self.cancel_file_transfer(cx);
        for image in &self.pending_images {
            image.cancel();
        }
        self.clear_pending_input();
        self.clear_flash(false);
        self.menu.reset();
        self.selection_epoch += 1;
        self.discard_composition(cx);
        let endpoint = &self.endpoints[self.selected_endpoint];
        self.selected_generation = endpoint.generation;
        self.live = endpoint.live().clone();
        if !endpoint.initial_surface {
            self.live.invalidate_surface();
        }
        // Another connection's picture is not this one's, so a detach or a
        // retarget starts from an empty terminal area. A reconnect after a drop
        // keeps the last picture, dimmed, until the new connection's arrives.
        if endpoint.outage().is_none() {
            self.presentation.clear();
        }
        self.selection = None;
        self.terminal_mouse = None;
        self.pressed_terminal_link = None;
        self.notices.clear();
        self.last_queued_options = None;
        self.sent_focus = None;
        self.wheel = WheelAccumulator::default();
        if !endpoint.detached() && !endpoint.initial_surface {
            let deadline = Instant::now() + ACTIVATION_TIMEOUT;
            self.live
                .activation
                .transition(crate::state::activation::Transition::Select(deadline));
            if let Ok(mut state) = endpoint.connection.inbox.try_lock() {
                state
                    .activation
                    .transition(crate::state::activation::Transition::Select(deadline));
            }
        }
        self.pending_navigation = None;
        self.pending_toast = None;
    }

    /// Shows the host `id` and remembers the choice. A refusal is returned,
    /// and the one a person can act on also shows in the status bar.
    pub(super) fn select_endpoint(
        &mut self,
        id: &str,
        cx: &mut Context<Self>,
    ) -> crate::Result<()> {
        self.switch_endpoint(id, cx)?;
        self.leave_core_group(cx);
        self.catalog.choose(id);
        if let Some(error) = self.catalog.poll_write() {
            self.notices.show(
                crate::notice::Cause::Configuration,
                fl!("endpoint-save-selection-failed", error = error.localize()),
            );
            cx.notify();
        }
        Ok(())
    }

    /// The installation this window's local endpoint belongs to. A development
    /// target lists the development catalog's sessions, not the release ones.
    pub(super) fn local_development(&self) -> bool {
        matches!(
            self.endpoints[0].connection.target,
            ConnectTarget::Session {
                development: true,
                ..
            }
        )
    }

    /// Attach this window to another named local session. The local endpoint
    /// keeps its identity, so the session changes its target rather than adding
    /// an endpoint for every session on the machine.
    pub(super) fn select_local_session(&mut self, name: &str, cx: &mut Context<Self>) {
        let target = ConnectTarget::Session {
            name: name.to_owned(),
            development: self.local_development(),
        };
        if self.selected_endpoint == 0 && self.endpoints[0].connection.target == target {
            return;
        }
        // Retarget before selecting: the poll loop reconnects endpoint zero, so
        // it must never still name the session it is leaving.
        if self.endpoints[0].connection.target != target {
            let enabled = self.endpoints[0].enabled();
            self.endpoints[0].retarget(target, enabled);
        }
        if self.selected_endpoint != 0 && self.switch_endpoint(LOCAL, cx).is_err() {
            return;
        }
        // This is a deliberate move off any remote selection, which must not be
        // restored over it on the next launch.
        self.catalog.choose(LOCAL);
        self.reconnect(cx);
        // After the reconnect: resetting the connection clears the error slot.
        if let Some(error) = self.catalog.poll_write() {
            self.notices.show(
                crate::notice::Cause::Configuration,
                fl!("endpoint-save-selection-failed", error = error.localize()),
            );
        }
        cx.notify();
    }

    /// Attach this window to another session of a saved device. The device keeps
    /// its identity, so the session changes that endpoint's target rather than
    /// adding an endpoint for every session the host has.
    pub(super) fn select_device_session(
        &mut self,
        id: &str,
        session: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self
            .endpoints
            .iter()
            .position(|endpoint| endpoint.id == id && endpoint.enabled())
        else {
            return;
        };
        // Only a remote device has a session to name; the local endpoint has
        // its own path through `select_local_session`.
        let Some(host) = self.endpoints[index].connection.target.remote_host() else {
            return;
        };
        let target = host.target(session);
        if self.selected_endpoint == index && self.endpoints[index].connection.target == target {
            return;
        }
        // Retarget before switching, exactly as attaching to a local session
        // does: the poll loop reconnects this endpoint, so it must never still
        // name the session the window is leaving.
        let retargeted = self.endpoints[index].connection.target != target;
        if retargeted {
            let enabled = self.endpoints[index].enabled();
            self.endpoints[index].retarget(target, enabled);
        }
        if self.selected_endpoint != index && self.switch_endpoint(id, cx).is_err() {
            return;
        }
        self.catalog.choose(id);
        // Only a session this device was not already on needs a new connection:
        // choosing the device itself keeps the transport it has, the way choosing
        // it from the picker does.
        if retargeted {
            self.reconnect(cx);
        }
        self.leave_core_group(cx);
        // After the reconnect: resetting the connection clears the error slot.
        if let Some(error) = self.catalog.poll_write() {
            self.notices.show(
                crate::notice::Cause::Configuration,
                fl!("endpoint-save-selection-failed", error = error.localize()),
            );
        }
        cx.notify();
    }

    fn switch_endpoint(&mut self, id: &str, cx: &mut Context<Self>) -> Result<(), Error> {
        let Some(index) = self
            .endpoints
            .iter()
            .position(|e| e.id == id && e.enabled())
        else {
            return Err(Error::NoHost { id: id.to_owned() });
        };
        if index == self.selected_endpoint {
            return Ok(());
        }
        if index != 0
            && self.endpoints[self.selected_endpoint]
                .live()
                .status
                .is_connected()
            && !self.endpoints[self.selected_endpoint]
                .live()
                .supports_surface
        {
            let error = Error::SurfaceSwitchingUnsupported;
            self.notices
                .show(crate::notice::Cause::Activation, error.localize());
            cx.notify();
            return Err(error);
        }
        self.leave_selected(Instant::now());
        self.release_selected();
        if index == 0 {
            // Local is the escape hatch and never waits on a remote release. An
            // unsent release must still retire its exact source transport.
            for release in &self.pending_releases {
                if let ReleasePhase::Deferred(handle) = &release.phase {
                    handle.disconnect();
                }
            }
            self.pending_releases.clear();
        }
        self.selected_endpoint = index;
        self.device_filter.follow(id);
        self.reset_selected(cx);
        // The picture kept for an outage is the previous endpoint's.
        self.presentation.clear();
        cx.notify();
        Ok(())
    }

    fn release_selected(&mut self) {
        let endpoint = &mut self.endpoints[self.selected_endpoint];
        // A handshake started with an active surface cannot be demoted without
        // its boot ID. Retire that attempt rather than let it finish in background.
        if endpoint.initial_surface
            && (endpoint.connection.handle.is_none() || endpoint.live().snapshot.is_none())
        {
            endpoint.stop();
            endpoint.enter(Transition::RetryAt(Instant::now()));
            return;
        }
        if endpoint.initial_surface
            && let (Some(handle), Some(snapshot)) =
                (&endpoint.connection.handle, &endpoint.live().snapshot)
        {
            let mut release = Release {
                inbox: endpoint.connection.inbox.clone(),
                drained: endpoint.connection.drained.clone(),
                phase: ReleasePhase::Deferred(handle.clone()),
                boot: snapshot.boot_id.clone(),
            };
            if !release.resolved() {
                self.pending_releases.push(release);
            }
        } else if let Ok(mut state) = endpoint.connection.inbox.try_lock() {
            state.set_outer_focus(false);
            state.invalidate_surface();
        }
        endpoint.initial_surface = false;
        endpoint.live_mut().invalidate_surface();
    }

    pub(super) fn navigate_endpoint(
        &mut self,
        endpoint: &str,
        target: NavigationTarget<&str>,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.menu.page.is_some() {
            return false;
        }
        if self.select_endpoint(endpoint, cx).is_err() {
            return false;
        }
        self.pending_toast = None;
        self.pending_navigation = None;
        if self.input_ready() {
            self.navigate(target, cx);
        } else {
            self.pending_navigation = Some((&target).into());
        }
        true
    }

    pub(super) fn input_ready(&self) -> bool {
        self.pending_toast.is_none() && self.navigation_ready()
    }

    // A coherent surface permits the deferred navigation attempt, not terminal
    // input while its toast target is still waiting for inbox validation.
    pub(crate) fn navigation_ready(&self) -> bool {
        self.surface_activation_ready() && self.surface_matches_options()
    }

    /// Whether the daemon's frame is the size this client asked for. A
    /// mismatch means someone else resized the tab (a CLI, or another
    /// window): the size has to be asked for again, not only waited on.
    pub(crate) fn surface_matches_options(&self) -> bool {
        self.live.surface.as_ref().is_some_and(|surface| {
            surface.frame.width == self.options.surface_size.cols
                && surface.frame.height == self.options.surface_size.rows
        })
    }

    pub(crate) fn surface_activation_ready(&self) -> bool {
        self.endpoints
            .get(self.selected_endpoint)
            .is_some_and(|endpoint| {
                endpoint.connection.handle.is_some()
                    && endpoint.surface_requested()
                    && self.pending_releases.is_empty()
                    && self.live.surface_ready()
            })
    }
}

// The fixtures drive the real client over bound Unix sockets and POSIX processes.
#[cfg(all(test, unix))]
mod lifecycle_tests;

#[cfg(test)]
mod tests;
