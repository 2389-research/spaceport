// ABOUTME: Owns one embedded SSH connection and its ordered Herdr input.
// ABOUTME: Drains the worker continuously and coalesces display updates for Swift.
#[path = "history.rs"]
pub(crate) mod history;
use crate::{
    projection::{
        FocusAction, HostChallenge, InputTarget, MobileView, OwnedNavigationTarget, Projection,
    },
    ssh::{
        EnclaveSigner, EnrollmentRequest, MobilePty, PtyRoutes, SshAuth, SshError, SshHost,
        open_bridge,
    },
};
use spaceport_activation::Failure;
use spaceport_client::{
    Client, ClientEvent, ClientHandle, ConnectOptions,
    protocol::{ClientKeyCode, ClientKeyKind, ClientPaneInputEvent, ClientSurfaceSize},
};
use std::{
    io,
    sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, thiserror::Error, uniffi::Error)]
pub enum MobileError {
    #[error("Herdr did not answer the history search in time.")]
    HistoryTimeout,
    #[error("A history search is already underway.")]
    HistoryBusy,
    #[error("Herdr returned an invalid history result.")]
    HistoryResult,
    #[error("The displayed terminal has changed. Wait for the current view.")]
    StaleView,
    #[error("Wait for Herdr to confirm the selected terminal.")]
    NavigationPending,
    #[error("Herdr did not answer the connection check in time.")]
    ResumeTimeout,
    #[error("Herdr could not confirm the retained connection.")]
    ResumeUnconfirmed,
    #[error("The terminal is disconnected.")]
    Disconnected,
    #[error("The terminal dimensions are outside the supported range.")]
    Geometry,
    #[error("The host could not open a direct terminal.")]
    TerminalUnavailable,
    #[error("Herdr could not resolve the selected terminal.")]
    TerminalLookup,
    #[error("The direct terminal did not answer in time.")]
    TerminalTimeout,
    #[error("The direct terminal has ended.")]
    TerminalEnded,
    #[error("Wait for the selected terminal to appear before typing.")]
    TerminalStarting,
    #[error("The terminal input buffer is full or the paste is too large.")]
    TerminalQueueFull,
    #[error("Could not transfer the screenshot to this host.")]
    ImageTransfer,
    #[error("Could not confirm the screenshot on the host. A temporary copy may remain.")]
    ImageRetentionUnconfirmed,
    #[error("This key is unsupported.")]
    InvalidKey,
    #[error("The scroll is outside the displayed terminal.")]
    InvalidScroll,
    #[error("{message}")]
    Client { message: String },
}

impl From<spaceport_client::Error> for MobileError {
    fn from(source: spaceport_client::Error) -> Self {
        // UniFFI is the presentation boundary. The transport preserves typed
        // causes until they reach this public error.
        Self::Client {
            message: source.to_string(),
        }
    }
}

pub(crate) fn geometry(
    cols: u16,
    rows: u16,
    cell_width: u32,
    cell_height: u32,
) -> Result<ConnectOptions, MobileError> {
    let options = ConnectOptions {
        surface_size: ClientSurfaceSize { cols, rows },
        cell_width_px: cell_width,
        cell_height_px: cell_height,
    };
    options.validate().map_err(|_| MobileError::Geometry)?;
    Ok(options)
}

#[derive(uniffi::Object)]
pub struct MobileSession {
    enrollment: Mutex<Option<tokio::sync::oneshot::Sender<EnrollmentRequest>>>,
    /// Signalled whenever the drain takes in an event, so a blocking search can
    /// wait for a frame instead of asking the daemon about content it is still
    /// writing.
    frames: Arc<Condvar>,
    handle: ClientHandle,
    projection: Arc<Mutex<Projection>>,
    routes: Arc<PtyRoutes>,
}

impl Drop for MobileSession {
    fn drop(&mut self) {
        self.routes.cancel();
        self.handle.disconnect();
    }
}

#[uniffi::export]
impl MobileSession {
    #[uniffi::constructor]
    pub fn open(
        host: SshHost,
        auth: SshAuth,
        session: String,
        cols: u16,
        rows: u16,
        cell_width: u32,
        cell_height: u32,
    ) -> Result<Arc<Self>, MobileError> {
        let options = geometry(cols, rows, cell_width, cell_height)?;
        let projection = Arc::new(Mutex::new(Projection::new(options)));
        let connecting = projection.clone();
        let (sender, request) = tokio::sync::oneshot::channel();
        let sender = matches!(auth, SshAuth::Password { .. }).then_some(sender);
        let routes = Arc::new(PtyRoutes::default());
        let worker_routes = routes.clone();
        let path_routes = routes.clone();
        let named_session = session.clone();
        let client = spaceport_client::connect_embedded_bridge_with_path(
            &session,
            options,
            move |command, stop| {
                open_bridge(
                    host,
                    auth,
                    command.to_owned(),
                    stop,
                    Some(request),
                    Some((worker_routes, named_session)),
                )
                .map_err(|error| connection_error(&connecting, error))
            },
            move |path| path_routes.accept_path(path),
        )?;
        Self::from_client(client, projection, sender, routes)
    }

    /// Install once over this password login, then independently verify key-only SSH access.
    pub fn authorize_phone(
        &self,
        public_key: Vec<u8>,
        signer: Arc<dyn EnclaveSigner>,
    ) -> Result<(), SshError> {
        let sender = lock(&self.enrollment).take().ok_or(SshError::Cancelled)?;
        EnrollmentRequest::submit(sender, public_key, signer)
    }

    /// Returns the latest view only when something has changed since the last read.
    pub fn poll(&self, after: u64) -> Option<MobileView> {
        self.poll_view(after, false)
    }

    pub fn poll_for_attach(&self, after: u64) -> Option<MobileView> {
        self.poll_view(after, true)
    }

    pub fn attach_terminal(
        &self,
        target: InputTarget,
        cols: u16,
        rows: u16,
    ) -> Result<Arc<MobilePty>, MobileError> {
        if self.handle.is_disconnected() {
            return Err(MobileError::Disconnected);
        }
        self.routes
            .attach(self.projection.clone(), target, cols, rows)
    }

    /// Leave the bridge attached while refusing input from the old presentation.
    pub fn park(&self) {
        self.routes.cancel();
        let mut state = lock(&self.projection);
        state.park();
        let release = state
            .snapshot
            .as_ref()
            .ok_or(MobileError::StaleView)
            .and_then(|s| {
                self.handle.set_focus(&s.boot_id, false)?;
                Ok(self.handle.set_surface_active(&s.boot_id, false)?)
            });
        match release {
            Ok(request) => state.release_request = Some(request),
            Err(error) => {
                // A partially queued release cannot establish relinquished ownership.
                state.end(error.to_string());
                self.handle.disconnect();
            }
        }
    }

    /// Queue a correlated surface-interest request on the original SSH channel.
    pub fn begin_resume(&self, boot_id: String) -> Result<String, MobileError> {
        let mut state = self.current(&boot_id)?;
        let request = self.handle.set_surface_active(&boot_id, true)?;
        state.start_resume(request.clone(), Instant::now());
        Ok(request)
    }

    /// Only the matching reply and a sufficiently recent projection can unlock input.
    pub fn check_resume(&self, boot_id: String, request: String) -> Result<bool, MobileError> {
        let mut state = self.current(&boot_id)?;
        if !state.finish_resume(&request, Instant::now())? {
            return Ok(false);
        }
        if let Some(boot) = state.initial_focus() {
            if let Err(error) = self.handle.set_focus(boot, true) {
                state.end(error.to_string());
                self.handle.disconnect();
                return Err(error.into());
            }
            state.reported_focus = true;
        }
        Ok(true)
    }

    /// A boot snapshot must have arrived before stream loss can authorize mobile recovery.
    pub fn transport_lost(&self) -> bool {
        self.handle.transport_lost() && lock(&self.projection).snapshot.is_some()
    }

    /// A replacement connection cannot safely carry an unconfirmed shared focus selection.
    pub fn focus_interrupted(&self) -> bool {
        lock(&self.projection).focus_interrupted()
    }

    pub fn disconnect(&self) {
        self.routes.cancel();
        self.handle.disconnect();
        let mut state = lock(&self.projection);
        if !state.disconnected {
            state.history = Default::default();
            state.disconnected = true;
            state.version += 1;
        }
    }

    /// Returns the first projection version belonging to this jump.
    pub fn go_to_pane(
        &self,
        boot_id: String,
        workspace_id: String,
        tab_id: String,
        pane_id: String,
        expected_agent: Option<String>,
    ) -> Result<u64, MobileError> {
        let mut state = self.active(&boot_id)?;
        let request_version =
            state.begin_go_to(&boot_id, workspace_id, tab_id, pane_id, expected_agent)?;
        let result = Self::advance_go_to(&self.handle, &mut state);
        self.routes.invalidate(&state);
        result?;
        Ok(request_version)
    }

    pub fn cancel_go_to(&self) {
        lock(&self.projection).cancel_go_to();
    }

    pub fn focus_workspace(&self, boot_id: String, id: String) -> Result<(), MobileError> {
        self.navigate(&boot_id, OwnedNavigationTarget::Workspace(id).into())
    }
    pub fn focus_tab(&self, boot_id: String, id: String) -> Result<(), MobileError> {
        self.navigate(&boot_id, OwnedNavigationTarget::Tab(id).into())
    }
    pub fn create_tab(
        &self,
        boot_id: String,
        workspace: String,
        source_tab: String,
    ) -> Result<(), MobileError> {
        self.navigate(
            &boot_id,
            FocusAction::CreateTab {
                workspace,
                source_tab,
            },
        )
    }
    pub fn focus_pane(&self, boot_id: String, id: String) -> Result<(), MobileError> {
        self.navigate(&boot_id, OwnedNavigationTarget::Pane(id).into())
    }

    /// Run on a background thread. Swift rechecks the original input route before pasting.
    pub fn stage_draft_image(
        &self,
        target: InputTarget,
        png: Vec<u8>,
    ) -> Result<Arc<crate::StagedImage>, MobileError> {
        {
            let state = self.current(&target.boot_id)?;
            if state.target().as_ref() != Some(&target)
                && state.direct_target().as_ref() != Some(&target)
            {
                return Err(MobileError::StaleView);
            }
        }
        self.routes.images.stage(png)
    }

    pub fn send_text(&self, target: InputTarget, text: String) -> Result<(), MobileError> {
        self.input(target, [ClientPaneInputEvent::TextCommit(text)])
    }
    pub fn send_paste(
        &self,
        target: InputTarget,
        text: String,
        press_return: bool,
    ) -> Result<(), MobileError> {
        // Keep Return in the same addressed message when another client changes focus.
        let events = std::iter::once(ClientPaneInputEvent::Paste(text))
            .chain(press_return.then(|| key_press(ClientKeyCode::Enter, 0)));
        self.input(target, events)
    }

    pub fn scroll(
        &self,
        target: InputTarget,
        lines: i16,
        column: u16,
        row: u16,
    ) -> Result<(), MobileError> {
        let mut state = self.active(&target.boot_id)?;
        let events = state.wheel(&target, lines, column, row)?;
        if !events.is_empty() {
            state.note_input(Instant::now());
            self.send(&target, events)?;
        }
        Ok(())
    }
    pub fn send_key(
        &self,
        target: InputTarget,
        key: String,
        modifiers: u8,
    ) -> Result<(), MobileError> {
        let code = match key.as_str() {
            "Backspace" => ClientKeyCode::Backspace,
            "Enter" => ClientKeyCode::Enter,
            "Left" => ClientKeyCode::Left,
            "Right" => ClientKeyCode::Right,
            "Up" => ClientKeyCode::Up,
            "Down" => ClientKeyCode::Down,
            "Home" => ClientKeyCode::Home,
            "End" => ClientKeyCode::End,
            "PageUp" => ClientKeyCode::PageUp,
            "PageDown" => ClientKeyCode::PageDown,
            "Tab" => ClientKeyCode::Tab,
            "BackTab" => ClientKeyCode::BackTab,
            "Delete" => ClientKeyCode::Delete,
            "Insert" => ClientKeyCode::Insert,
            "Esc" => ClientKeyCode::Esc,
            _ => {
                let mut chars = key.chars();
                match (chars.next(), chars.next()) {
                    (Some(ch), None) if !ch.is_control() => ClientKeyCode::Char(ch),
                    _ => return Err(MobileError::InvalidKey),
                }
            }
        };
        self.input(target, [key_press(code, modifiers)])
    }

    pub fn resize(
        &self,
        boot_id: String,
        cols: u16,
        rows: u16,
        cell_width: u32,
        cell_height: u32,
    ) -> Result<(), MobileError> {
        let options = geometry(cols, rows, cell_width, cell_height)?;
        let mut state = self.active(&boot_id)?;
        if state.options == options {
            return Ok(());
        }
        Self::resize_tab(&self.handle, &mut state, &boot_id, options)
    }

    /// Interaction can reclaim a tab another client sized. Polling never does.
    pub fn claim_size(&self, boot_id: String) -> Result<(), MobileError> {
        let mut state = self.active(&boot_id)?;
        state.note_input(Instant::now());
        if !state.sized() {
            let options = state.options;
            Self::resize_tab(&self.handle, &mut state, &boot_id, options)?;
        }
        Ok(())
    }
}

impl MobileSession {
    fn poll_view(&self, after: u64, direct_attach: bool) -> Option<MobileView> {
        let state = {
            let mut state = lock(&self.projection);
            if state.direct_attach != direct_attach {
                if !direct_attach {
                    self.routes.cancel();
                }
                state.direct_attach = direct_attach;
                state.version += 1;
            } else if after == state.version {
                return None;
            }
            state.clone()
        };
        Some(state.view())
    }

    pub(crate) fn from_client(
        client: Client,
        projection: Arc<Mutex<Projection>>,
        enrollment: Option<tokio::sync::oneshot::Sender<EnrollmentRequest>>,
        routes: Arc<PtyRoutes>,
    ) -> Result<Arc<Self>, MobileError> {
        let frames = Arc::new(Condvar::new());
        let session = Arc::new(Self {
            enrollment: Mutex::new(enrollment),
            frames: frames.clone(),
            handle: client.handle.clone(),
            projection: projection.clone(),
            routes: routes.clone(),
        });
        thread::Builder::new()
            .name("spaceport-mobile-state".into())
            .spawn(move || drain(client, projection, routes, frames))
            .map_err(|e| MobileError::Client {
                message: e.to_string(),
            })?;
        Ok(session)
    }

    fn current(&self, boot: &str) -> Result<MutexGuard<'_, Projection>, MobileError> {
        let state = lock(&self.projection);
        if state.disconnected || self.handle.is_disconnected() {
            return Err(MobileError::Disconnected);
        }
        if state.snapshot.as_ref().is_none_or(|s| s.boot_id != boot) {
            return Err(MobileError::StaleView);
        }
        Ok(state)
    }

    fn active(&self, boot: &str) -> Result<MutexGuard<'_, Projection>, MobileError> {
        let state = self.current(boot)?;
        if !state.is_active() {
            return Err(MobileError::StaleView);
        }
        Ok(state)
    }

    fn resize_tab(
        handle: &ClientHandle,
        state: &mut Projection,
        boot: &str,
        options: ConnectOptions,
    ) -> Result<(), MobileError> {
        handle.resize(boot, options)?;
        state.options = options;
        state.version += 1;
        if !state.navigating()
            && let Some(tab) = state
                .snapshot
                .as_ref()
                .and_then(|s| s.focused_tab_id.as_deref())
        {
            handle.focus_tab(boot, tab)?;
        }
        if state.reported_focus || state.initial_focus().is_some() {
            handle.set_focus(boot, true)?;
            state.reported_focus = true;
        }
        Ok(())
    }

    fn navigate(&self, boot: &str, target: FocusAction) -> Result<(), MobileError> {
        let mut state = self.active(boot)?;
        if state.navigating() || state.go_to_pending() {
            return Err(MobileError::NavigationPending);
        }
        let result = Self::queue_focus(&self.handle, &mut state, boot, target);
        self.routes.invalidate(&state);
        result
    }

    fn advance_go_to(handle: &ClientHandle, state: &mut Projection) -> Result<(), MobileError> {
        let result = state.go_to_step().and_then(|next| {
            if let Some(next) = next {
                let boot = state
                    .snapshot
                    .as_ref()
                    .ok_or(MobileError::StaleView)?
                    .boot_id
                    .clone();
                Self::queue_focus(handle, state, &boot, next.into())?;
            }
            Ok(())
        });
        if result.is_err() {
            state.cancel_go_to();
        }
        result
    }

    fn queue_focus(
        handle: &ClientHandle,
        state: &mut Projection,
        boot: &str,
        target: FocusAction,
    ) -> Result<(), MobileError> {
        if state
            .snapshot
            .as_deref()
            .is_none_or(|s| !target.reachable_in(s))
        {
            return Err(MobileError::StaleView);
        }
        let navigation = match &target {
            FocusAction::Select(OwnedNavigationTarget::Workspace(id)) => {
                handle.focus_workspace(boot, id)
            }
            FocusAction::Select(OwnedNavigationTarget::Tab(id)) => handle.focus_tab(boot, id),
            FocusAction::Select(OwnedNavigationTarget::Pane(id)) => handle.focus_pane(boot, id),
            FocusAction::CreateTab { workspace, .. } => handle.request(
                boot,
                spaceport_client::Method::TabCreate,
                serde_json::json!({"workspace_id": workspace, "focus": true}),
            ),
        }?;
        state.note_input(Instant::now());
        state.version += 1;
        // The barrier is what establishes which pane owns input. A queue that
        // refused it as full proves it never entered the ordered queue, so the
        // fence keeps the destination and offers it again; the connection is
        // never dropped for a barrier that was simply not taken yet.
        state.start_fence(navigation, target, handle.set_surface_active(boot, true));
        Ok(())
    }

    fn input(
        &self,
        target: InputTarget,
        events: impl IntoIterator<Item = ClientPaneInputEvent>,
    ) -> Result<(), MobileError> {
        let mut state = self.current(&target.boot_id)?;
        if state.target().as_ref() != Some(&target) {
            return Err(MobileError::StaleView);
        }
        state.note_input(Instant::now());
        self.send(&target, events)
    }

    fn send(
        &self,
        target: &InputTarget,
        events: impl IntoIterator<Item = ClientPaneInputEvent>,
    ) -> Result<(), MobileError> {
        if let Some(popup) = &target.popup_id {
            self.handle
                .send_popup_input(&target.boot_id, popup, events)?
        } else {
            self.handle
                .send_input(&target.boot_id, &target.pane_id, events)?
        }
        Ok(())
    }
}

fn key_press(code: ClientKeyCode, modifiers: u8) -> ClientPaneInputEvent {
    ClientPaneInputEvent::Key {
        code,
        modifiers,
        kind: ClientKeyKind::Press,
        repeat_count: 1,
        shifted_codepoint: None,
        generated_text: None,
        tracks_release: false,
        physical_key_id: None,
        windows_record: None,
    }
}

/// What a barrier that never completed is reported as. Only the words live
/// here: which cause it was is `spaceport-activation`'s.
fn fence_failure_reason(cause: Failure) -> String {
    match cause {
        Failure::Timeout => {
            "Herdr did not confirm the selected terminal. Reconnect to continue.".into()
        }
        Failure::InvalidResponse => {
            "Herdr could not confirm the selected terminal. Reconnect to continue.".into()
        }
        Failure::Rejected => {
            "Herdr refused to activate the selected terminal. Reconnect to continue.".into()
        }
        Failure::Queue => {
            "The phone could not ask Herdr to activate the selected terminal. Reconnect to continue."
                .into()
        }
        Failure::BootChanged => {
            "Herdr restarted while switching panes. Reconnect to continue.".into()
        }
    }
}

pub(crate) fn connection_error(
    state: &Mutex<Projection>,
    error: SshError,
) -> spaceport_client::Error {
    let challenge = match &error {
        SshError::UnknownHost { fingerprint } => Some(HostChallenge {
            fingerprint: fingerprint.clone(),
            changed: false,
        }),
        SshError::ChangedHost { fingerprint } => Some(HostChallenge {
            fingerprint: fingerprint.clone(),
            changed: true,
        }),
        _ => None,
    };
    if let Some(challenge) = challenge {
        let mut state = lock(state);
        state.challenge = Some(challenge);
        state.version += 1;
    }
    spaceport_client::Error::from(io::Error::other(error))
}

pub(crate) fn lock<T>(state: &Mutex<T>) -> MutexGuard<'_, T> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

fn drain(
    client: Client,
    projection: Arc<Mutex<Projection>>,
    routes: Arc<PtyRoutes>,
    frames: Arc<Condvar>,
) {
    loop {
        let event = client.events.recv_timeout(Duration::from_millis(250));
        let mut state = lock(&projection);
        if state.disconnected {
            return;
        }
        // Process incoming snapshots before claiming their tab. A queued focus
        // change must never cause a claim for the tab being left.
        let mut received = true;
        if let Some(cause) = state.fence_failure() {
            state.end(fence_failure_reason(cause));
        } else {
            match event {
                Ok(ClientEvent::Snapshot(s)) => state.snapshot(s),
                Ok(ClientEvent::Surface(s)) => state.surface(s),
                Ok(ClientEvent::Response {
                    request_id,
                    response,
                }) => {
                    if let Some(reply) = state.history.take_reply(&request_id) {
                        if let Some(reply) = reply {
                            let _ = reply
                                .send(spaceport_client::scrollback::decode_response(&response));
                        }
                        received = false;
                    } else {
                        state.receipt(&request_id, &response);
                    }
                }
                Ok(ClientEvent::CommandRejected { request_id, reason }) => {
                    if let Some(reply) = request_id
                        .as_deref()
                        .and_then(|id| state.history.take_reply(id))
                    {
                        if let Some(reply) = reply {
                            let _ = reply.send(Err(reason));
                        }
                        received = false;
                    } else {
                        state.rejected(request_id.as_deref(), reason.to_string());
                    }
                }
                Ok(ClientEvent::Disconnected { reason, .. }) => state.end(reason),
                Err(error) if error.is_disconnected() => state.end("Herdr disconnected.".into()),
                _ => received = false,
            }
        }
        // A barrier the ordered queue refused never entered it, so it is
        // offered again rather than leaving input fenced on a request that was
        // never sent.
        if let Some((boot, focus)) = state.retry_barrier() {
            let result = client.handle.set_surface_active(&boot, true);
            state.barrier_queued(result, boot, focus);
            received = true;
        }
        if let Err(error) = MobileSession::advance_go_to(&client.handle, &mut state) {
            state.error = Some(error.to_string());
            received = true;
        }
        if let Some(boot) = state.initial_focus() {
            match client.handle.set_focus(boot, true) {
                Ok(()) => state.reported_focus = true,
                Err(spaceport_client::Error::Full) => {}
                Err(error) => {
                    state.end(error.to_string());
                    received = true;
                }
            }
        }
        if state.reassert_due(Instant::now())
            && let Some(boot) = state.snapshot.as_ref().map(|s| s.boot_id.clone())
        {
            let options = state.options;
            if let Err(error) =
                MobileSession::resize_tab(&client.handle, &mut state, &boot, options)
            {
                state.error = Some(error.to_string());
                received = true;
            }
        }
        if received {
            if state.direct_attach {
                routes.invalidate(&state);
            }
            state.version += 1;
            // Wakes a search waiting for the pane's content to settle.
            frames.notify_all();
        }
        if state.disconnected {
            client.handle.disconnect();
            return;
        }
    }
}
