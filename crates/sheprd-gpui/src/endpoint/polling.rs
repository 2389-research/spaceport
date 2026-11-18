//! The window's periodic endpoint work: draining each connection, activating
//! the selected surface, and probing sessions.
use super::{LOCAL, Redraw};
use crate::{
    SheprdWindow,
    i18n::{Localize, fl},
    notification_mutes::{Daemon, Mutes},
    state::{Answer, DialogResponse},
};
use gpui::{ClipboardItem, Context};
use sheprd_client::RemoteHost;
use std::time::Instant;

impl SheprdWindow {
    /// `request`'s answer in `answer`'s slot once it has arrived, cleared
    /// from the selected connection's mailbox too so later updates do not
    /// keep copying it.
    pub(crate) fn take_answer(&mut self, answer: Answer, request: &str) -> Option<DialogResponse> {
        let result = self.live.take_answer(answer, request)?;
        self.endpoints[self.selected_endpoint]
            .connection
            .retire_answer(answer, request);
        Some(result)
    }

    /// Writes daemon-forwarded OSC 52 payloads to the pasteboard and reports
    /// the copy the way a local selection does. Returns whether anything was
    /// written, so the window repaints for the flash.
    pub(super) fn apply_clipboard_writes(
        &mut self,
        writes: impl IntoIterator<Item = String>,
        cx: &mut Context<Self>,
    ) -> bool {
        let mut writes = writes.into_iter().peekable();
        if writes.peek().is_none() {
            return false;
        }
        for text in writes {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
        if self.config.clipboard_toast.enabled {
            self.show_flash(crate::window::Flash::success(fl!("endpoint-copied")), cx);
        }
        true
    }

    pub(crate) fn poll_endpoints(&mut self, cx: &mut Context<Self>) {
        // A Core-only view has no legacy endpoint to poll. Its Core link is
        // handled later in the window tick.
        if self.endpoints.is_empty() {
            return;
        }
        self.recover_connections(
            Instant::now(),
            std::time::SystemTime::now(),
            super::wake::network_changes(),
        );
        // Record the focused target the user last saw before a newer snapshot
        // can replace it; input held across a gap may only go there.
        self.flush_pending_input(cx);
        if let Some(error) = self.catalog.poll_write() {
            crate::storage_warning::warn_storage_failure("Save host selection", &error); // i18n: data
            self.notices.show(
                crate::notice::Cause::Configuration,
                fl!("endpoint-save-selection-failed", error = error.localize()),
            );
            cx.notify();
        }
        if let Some(result) = self.catalog.poll() {
            match result {
                Ok(update) => {
                    self.catalog.accept(&update);
                    self.reconcile_devices(update.hosts, update.wsl, cx);
                }
                Err(error) => {
                    self.notices.show(
                        crate::notice::Cause::Configuration,
                        fl!("endpoint-catalog-failed", error = error.localize()),
                    );
                    cx.notify();
                }
            }
        }
        let mut changed = Redraw::None;
        // Whether the selected endpoint itself moved on. Only then does the
        // window take its state: another endpoint changing, or this one's
        // inbox being busy for a poll, must not replace what the window has
        // stamped since, such as the split drag request it is waiting on.
        let mut selected_changed = false;
        // OSC 52 writes are drained per endpoint so they are written once and
        // never linger in a live state that a later poll would re-read.
        let mut clipboard_writes = std::collections::VecDeque::new();
        for (index, endpoint) in self.endpoints.iter_mut().enumerate() {
            let updated = endpoint.poll(Instant::now());
            // Nothing to keep current, or to silence, while nothing is muted.
            let muted = Mutes::any(cx)
                .and(endpoint.live.latest_snapshot())
                .filter(|_| endpoint.live.status.is_connected())
                .cloned()
                .map(|snapshot| (Daemon::of(endpoint), snapshot));
            // Only a change touches the global, which every window observes.
            if updated == Redraw::Window
                && let Some((daemon, snapshot)) = &muted
            {
                let now = std::time::SystemTime::now();
                if Mutes::any(cx).is_some_and(|mutes| !mutes.current(daemon, snapshot, now)) {
                    Mutes::update(cx, |mutes| mutes.reconcile(daemon, snapshot, now));
                }
            }
            clipboard_writes.append(&mut endpoint.live.clipboard_writes);
            // Bells are a presentation effect: only the selected endpoint's
            // ring, as the TUI drops them from inactive endpoints.
            let bells = std::mem::take(&mut endpoint.live.bells);
            let host_muted = Mutes::any(cx).is_some_and(|mutes| mutes.host(&endpoint.id));
            if index == self.selected_endpoint && !host_muted {
                self.bell.queue(bells);
            }
            selected_changed |= index == self.selected_endpoint && updated != Redraw::None;
            let silenced = Mutes::any(cx)
                .map(|mutes| match &muted {
                    Some((daemon, snapshot)) => mutes.silenced(daemon, snapshot),
                    None => mutes.silenced_offline(&endpoint.id),
                })
                .unwrap_or_default();
            self.sound.poll(
                &mut endpoint.sounds,
                &mut endpoint.live,
                index == self.selected_endpoint && self.active,
                Instant::now(),
                |event| {
                    silenced.notice(
                        event.workspace_id.as_deref(),
                        event.tab_id.as_deref(),
                        event.pane_id.as_deref(),
                    )
                },
            );
            changed = changed.max(updated);
            if endpoint.may_dial() && Instant::now() >= endpoint.retry_at() {
                endpoint.connect(self.options, index == 0 && self.selected_endpoint == 0);
                selected_changed |= index == self.selected_endpoint;
                changed = Redraw::Window;
            }
            // Only a requested surface flushes, which in practice is the
            // selected endpoint's, so its error is the one the window shows.
            if let Err(error) = endpoint.flush_config_reload()
                && index == self.selected_endpoint
            {
                self.notices.show(
                    crate::notice::Cause::Configuration,
                    fl!("endpoint-reload-config-failed", error = error.localize()),
                );
            }
        }
        crate::palette::LastSeen::trim(
            self.endpoints
                .iter_mut()
                .map(|endpoint| &mut endpoint.last_seen),
        );
        if self.apply_clipboard_writes(clipboard_writes, cx) {
            changed = Redraw::Window;
        }
        self.restore_selection(cx);
        if self.tick_toasts(
            self.menu.page.is_some() || self.toasts_hidden,
            Instant::now(),
            cx,
        ) {
            changed = Redraw::Window;
        }
        let endpoint = &mut self.endpoints[self.selected_endpoint];
        if self.selected_generation != endpoint.generation {
            self.reset_selected(cx);
        }
        let endpoint = &mut self.endpoints[self.selected_endpoint];
        if selected_changed {
            let budget = self
                .live
                .activation
                .deadline()
                .filter(|_| !endpoint.initial_surface);
            self.live.apply_window_update(endpoint.live.clone());
            if let Some(deadline) = budget {
                self.live
                    .activation
                    .transition(crate::state::activation::Transition::Select(deadline));
            }
            if !self.live.status.is_connected() {
                self.notices.clear();
            }
        }
        if endpoint.outage().is_some() {
            self.presentation.hold();
        } else {
            self.presentation.resume();
        }
        // Activation is timed from the snapshot it needs. An endpoint that is
        // down or still handshaking is retried in place, never given up for Local.
        // The handle stops before the disconnect state reaches the inbox, so
        // `live` can still hold the lost connection's snapshot for a poll; the
        // outage is recorded as soon as the stop is seen.
        if self.live.snapshot.is_none() || endpoint.outage().is_some() {
            let now = Instant::now();
            self.live
                .activation
                .transition(crate::state::activation::Transition::Renew(now));
            if let Ok(mut state) = endpoint.connection.inbox.try_lock() {
                state
                    .activation
                    .transition(crate::state::activation::Transition::Renew(now));
            }
        }
        if !endpoint.initial_surface && !endpoint.detached() {
            if let Some(deadline) = self.live.activation.deadline()
                && let Ok(mut state) = endpoint.connection.inbox.try_lock()
            {
                state
                    .activation
                    .transition(crate::state::activation::Transition::Select(deadline));
            }
            self.live
                .activation
                .transition(crate::state::activation::Transition::Wait(Instant::now()));
            if let Ok(mut state) = endpoint.connection.inbox.try_lock() {
                state
                    .activation
                    .transition(crate::state::activation::Transition::Wait(Instant::now()));
            }
        }
        self.pending_releases
            .retain_mut(|release| !release.resolved());
        let retry = match &self.live.activation {
            crate::state::activation::Fence::Retry { boot, focus, .. } => {
                Some((boot.clone(), focus.clone()))
            }
            _ => None,
        };
        if (!endpoint.initial_surface || retry.is_some())
            && self.pending_releases.is_empty()
            && let (Some(handle), Some(snapshot)) =
                (&endpoint.connection.handle, &endpoint.live.snapshot)
            && let Ok(mut state) = endpoint.connection.inbox.try_lock()
        {
            // A newer inbox transition can supersede this window's retry.
            let current_retry = retry.as_ref().is_none_or(|(boot, focus)| {
                matches!(&state.activation, crate::state::activation::Fence::Retry { boot: current, focus: target, .. } if current == boot && target == focus)
            });
            if current_retry
                && retry
                    .as_ref()
                    .is_some_and(|(boot, _)| *boot != snapshot.boot_id)
            {
                state
                    .activation
                    .transition(crate::state::activation::Transition::Boot(
                        &snapshot.boot_id,
                    ));
            } else if current_retry {
                let result = if endpoint.initial_surface {
                    Ok(())
                } else {
                    handle.resize(&snapshot.boot_id, self.options)
                }
                .and_then(|()| handle.set_surface_active(&snapshot.boot_id, true));
                let queued = result.is_ok();
                if let Err(error) = &result
                    && !matches!(error, sheprd_client::Error::Full)
                {
                    self.notices.show(
                        crate::notice::Cause::Activation,
                        fl!("endpoint-activate-failed", error = error.localize()),
                    );
                }
                state.invalidate_surface();
                state
                    .activation
                    .transition(crate::state::activation::Transition::Queued {
                        result,
                        boot: snapshot.boot_id.clone(),
                        focus: retry.and_then(|(_, focus)| focus),
                        active: true,
                        now: Instant::now(),
                    });
                if queued {
                    endpoint.initial_surface = true;
                    self.last_queued_options = Some(self.options);
                    self.sent_focus = None;
                }
            }
            state.dirty = true;
            self.live.apply_window_update(state.clone());
            // take_update already moved this response out of the inbox.
            self.live.dialog_response = endpoint.live.dialog_response.clone();
            changed = Redraw::Window;
        }
        // The window's state is current: a dropped row the daemon has answered
        // is done.
        self.finish_workspace_settle();
        // Held input precedes any deferred navigation.
        self.flush_pending_input(cx);
        if self.navigation_ready() {
            if let Ok(mut state) = self.endpoints[self.selected_endpoint]
                .connection
                .inbox
                .try_lock()
                && state.activation.request() == self.live.activation.request()
            {
                state
                    .activation
                    .transition(crate::state::activation::Transition::Opened);
            }
            self.live
                .activation
                .transition(crate::state::activation::Transition::Opened);
            if let Some(id) = self.pending_toast {
                self.navigate_toast(id, cx);
            } else if let Some(target) = self.pending_navigation.take() {
                self.dispatch_navigation(target.as_deref(), cx);
            }
        } else {
            let now = Instant::now();
            if let Ok(mut state) = self.endpoints[self.selected_endpoint]
                .connection
                .inbox
                .try_lock()
                && state.activation.request() == self.live.activation.request()
            {
                state
                    .activation
                    .transition(crate::state::activation::Transition::Expire(now));
            }
            self.live
                .activation
                .transition(crate::state::activation::Transition::Expire(now));
            if self.live.activation.failed() {
                let device = self.endpoints[self.selected_endpoint].label.clone();
                let error = match self.live.notices.text() {
                    Some(reason) => fl!(
                        "endpoint-activation-failed-because",
                        device = device,
                        reason = reason
                    ),
                    None => fl!("endpoint-activation-failed", device = device),
                };
                if self.selected_endpoint != 0 {
                    let _ = self.switch_endpoint(LOCAL, cx);
                } else {
                    // Recover Local with a fresh active handshake, even if the
                    // previous surface lane or its acknowledgement was unavailable.
                    self.reconnect(cx);
                }
                self.notices.show(crate::notice::Cause::Activation, error);
                changed = Redraw::Window;
            }
        }
        self.sync_server_keymap(cx);
        match changed {
            Redraw::None => {}
            Redraw::Terminal => self.redraw_terminal(cx),
            Redraw::Window => cx.notify(),
        }
    }

    /// Scan local sessions while the popup asks for them, and ask the saved
    /// devices for their own sessions on their own slower interval. Probing is
    /// I/O to the disk and to each host, so both run on workers and only this
    /// page starts one; results are cached between opens.
    pub(crate) fn poll_sessions(&mut self, cx: &mut Context<Self>) {
        if self.endpoints.is_empty() {
            return;
        }
        let development = self.local_development();
        let open = self.menu.page == Some(crate::menu::Page::Devices)
            && self.menu.unfolded_device.is_some();
        let now = Instant::now();
        let targets = self.probe_targets();
        self.sessions.devices.forget_replaced(&targets);
        // Hold the catalog steady through the mutation and its successful exit
        // animation. Workers stay bounded; finished answers wait in their inbox.
        if self.sessions.mutation.is_some() {
            if open && self.sessions.departure.is_some() {
                cx.notify();
            }
            return;
        }
        // New answers add or take away rows under the unfolded device; the
        // keys stay on the row they were on, not on its old place.
        // Only the unfolded device shows its sessions, so only it is asked.
        let unfolded = self.menu.unfolded_device.clone().filter(|_| open);
        let local = unfolded.as_deref() == Some(self.endpoints[0].id.as_str());
        let changed = self.keep_picker_selection(|this| {
            let scanned = this.sessions.poll(development, local, now);
            let wanted = unfolded.as_deref().filter(|_| !local);
            this.sessions.devices.poll(&targets, wanted, now) || scanned
        });
        if changed {
            cx.notify();
        }
    }

    /// The devices this window may ask for their sessions: every enabled saved
    /// device on an SSH host or in a WSL distribution. One the user disabled is
    /// never dialled, and the local endpoint has no host to ask.
    pub(crate) fn probe_targets(&self) -> Vec<(String, RemoteHost)> {
        self.endpoints
            .iter()
            .skip(1)
            .filter(|endpoint| endpoint.enabled())
            .filter_map(|endpoint| {
                Some((
                    endpoint.id.clone(),
                    endpoint.connection.target.remote_host()?,
                ))
            })
            .collect()
    }
}
