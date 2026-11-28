// ABOUTME: Projects authoritative Herdr snapshots and terminal cells for a phone.
// ABOUTME: Binds input to a connection, daemon boot, focused pane and popup.
mod go_to;
mod pane_map;
mod pane_trail;
use crate::controller::pane_tools::MobilePaneActions;
use spaceport_client::pane_history::{History, Step};
mod resume;
mod wheel;
pub use pane_map::{MobilePaneMap, MobilePaneRect};
pub(crate) use spaceport_activation::OwnedNavigationTarget;
use spaceport_activation::{Failure, Fence, Transition};
use spaceport_client::{
    ConnectOptions,
    agent_idle::{IdleTracker, now_ms},
    protocol::*,
    response,
};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct InputTarget {
    pub connection: u64,
    pub boot_id: String,
    pub pane_id: String,
    pub popup_id: Option<String>,
    #[uniffi(default = 0)]
    pub input_epoch: u64,
}

#[derive(Clone, uniffi::Record)]
pub struct MobileCell {
    pub symbol: String,
    pub fg: u32,
    pub bg: u32,
    pub modifier: u16,
    pub skip: bool,
    /// Index into `MobileTerminal::hyperlinks`, numbered by the frame rather
    /// than the sliced window, so a link's cells share one index with the
    /// pane beside them.
    #[uniffi(default = None)]
    pub hyperlink: Option<u32>,
}

/// How far a pane's viewport sits above its newest row, and the furthest the
/// daemon lets it go, in rows of the pane's own grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Record)]
pub struct MobileScroll {
    pub offset_from_bottom: u64,
    pub max_offset_from_bottom: u64,
    pub viewport_rows: u64,
}

#[derive(uniffi::Record)]
pub struct MobileTerminal {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<MobileCell>,
    pub cursor_x: Option<u16>,
    pub cursor_y: Option<u16>,
    /// The hyperlink targets the cells index, as the frame spells them: a
    /// `mailto:` link is carried unchanged and left for the reader to judge.
    #[uniffi(default = [])]
    pub hyperlinks: Vec<String>,
    /// The focused pane's history position; a popup keeps none.
    #[uniffi(default = None)]
    pub scroll: Option<MobileScroll>,
}

#[derive(uniffi::Record)]
pub struct MobileWorkspace {
    pub id: String,
    pub label: String,
    pub status: String,
}

#[derive(uniffi::Record)]
pub struct MobileTab {
    pub id: String,
    pub workspace_id: String,
    pub label: String,
    pub status: String,
    #[uniffi(default = false)]
    pub zoomed: bool,
}

#[derive(uniffi::Record)]
pub struct MobilePane {
    pub state_change_seq: Option<u64>,
    pub agent_name: Option<String>,
    pub id: String,
    pub tab_id: String,
    pub label: String,
    pub status: String,
    #[uniffi(default = None)]
    pub idle_first_observed_ms: Option<u64>,
    #[uniffi(default = false)]
    pub idle_onset_observed: bool,
    #[uniffi(default = None)]
    pub terminal_title: Option<String>,
    #[uniffi(default = None)]
    pub agent_identity: Option<String>,
}

#[derive(Clone, uniffi::Record)]
pub struct HostChallenge {
    pub fingerprint: String,
    pub changed: bool,
}

#[derive(uniffi::Record)]
pub struct MobileView {
    pub version: u64,
    pub boot_id: String,
    pub workspaces: Vec<MobileWorkspace>,
    pub tabs: Vec<MobileTab>,
    pub panes: Vec<MobilePane>,
    pub pane_map: Option<MobilePaneMap>,
    pub focused_workspace: Option<String>,
    pub focused_tab: Option<String>,
    pub focused_pane: Option<String>,
    pub terminal: Option<MobileTerminal>,
    pub input_target: Option<InputTarget>,
    pub error: Option<String>,
    pub disconnected: bool,
    pub host_challenge: Option<HostChallenge>,
    pub go_to_pending: bool,
    pub go_to_arrival: Option<u64>,
    #[uniffi(default = None)]
    pub roster_observed_at_ms: Option<u64>,
    #[uniffi(default = None)]
    pub back_pane: Option<String>,
    #[uniffi(default = None)]
    pub forward_pane: Option<String>,
    #[uniffi(default = None)]
    pub pane_actions: Option<MobilePaneActions>,
}

pub(crate) enum FocusAction {
    Select(OwnedNavigationTarget),
    CreateTab {
        workspace: String,
        source_tab: String,
    },
}

impl From<OwnedNavigationTarget> for FocusAction {
    fn from(target: OwnedNavigationTarget) -> Self {
        Self::Select(target)
    }
}

impl FocusAction {
    pub(crate) fn reachable_in(&self, snapshot: &ClientShellSnapshot) -> bool {
        match self {
            Self::Select(target) => target.reachable_in(snapshot),
            Self::CreateTab {
                workspace,
                source_tab,
            } => {
                snapshot.focused_workspace_id.as_ref() == Some(workspace)
                    && snapshot.focused_tab_id.as_ref() == Some(source_tab)
            }
        }
    }
}

#[derive(Clone)]
struct FocusRequest {
    id: String,
    /// Until the matching creation receipt supplies a destination, any current tab is unsafe.
    creating_from: Option<String>,
}

#[derive(Clone)]
pub(crate) struct ObservedSnapshot {
    snapshot: Arc<ClientShellSnapshot>,
    observed_at_ms: u64,
}

impl std::ops::Deref for ObservedSnapshot {
    type Target = ClientShellSnapshot;

    fn deref(&self) -> &Self::Target {
        &self.snapshot
    }
}

#[derive(Clone)]
pub(crate) struct Projection {
    pub(crate) diagnostic_capabilities: Option<String>,
    pub(crate) direct_attach: bool,
    pub(crate) history: crate::history::HistoryInbox,
    pane_trail: History,
    pub(crate) trail_host: Option<crate::controller::pane_tools::TrailHost>,
    pub(crate) pane_actions: MobilePaneActions,
    idle: IdleTracker,
    connection: u64,
    input_epoch: u64,
    visibility: resume::Visibility,
    pub(crate) parked_roster: Option<crate::controller::ParkedRosterCheck>,
    pub(crate) release_request: Option<String>,
    pub(crate) version: u64,
    pub(crate) options: ConnectOptions,
    pub(crate) snapshot: Option<ObservedSnapshot>,
    pending: Option<ObservedSnapshot>,
    surface: Option<Arc<PaneSurfaceFrame>>,
    focus_interrupted: bool,
    fence: Fence,
    /// The focus request's receipt may name a created tab or refuse the move.
    /// The ordered barrier has its own request ID.
    focus_request: Option<FocusRequest>,
    route: Option<go_to::Walk>,
    go_to_arrival: Option<u64>,
    last_input: Option<Instant>,
    reassert_since: Option<Instant>,
    pub(crate) reported_focus: bool,
    pub(crate) disconnected: bool,
    pub(crate) error: Option<String>,
    pub(crate) challenge: Option<HostChallenge>,
}

impl Projection {
    pub(crate) fn new(options: ConnectOptions) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self {
            diagnostic_capabilities: None,
            direct_attach: false,
            history: Default::default(),
            pane_trail: History::default(),
            trail_host: None,
            pane_actions: Default::default(),
            idle: IdleTracker::default(),
            connection: NEXT.fetch_add(1, Ordering::Relaxed),
            input_epoch: 0,
            visibility: resume::Visibility::Active,
            parked_roster: None,
            release_request: None,
            version: 1,
            options,
            snapshot: None,
            pending: None,
            surface: None,
            focus_interrupted: false,
            fence: Fence::Open,
            focus_request: None,
            route: None,
            go_to_arrival: None,
            last_input: None,
            reassert_since: None,
            reported_focus: false,
            disconnected: false,
            error: None,
            challenge: None,
        }
    }

    fn adopt_snapshot(&mut self, snapshot: ObservedSnapshot) {
        if self.is_active() {
            self.idle.observe(&snapshot, snapshot.observed_at_ms);
        }
        self.snapshot = Some(snapshot);
    }

    pub(crate) fn snapshot(&mut self, snapshot: Arc<ClientShellSnapshot>) {
        self.snapshot_at(snapshot, now_ms());
    }

    pub(crate) fn snapshot_at(&mut self, snapshot: Arc<ClientShellSnapshot>, observed_at_ms: u64) {
        let snapshot = ObservedSnapshot {
            snapshot,
            observed_at_ms,
        };
        if (self.navigating() || self.route.is_some())
            && self
                .snapshot
                .as_ref()
                .is_some_and(|s| s.boot_id != snapshot.boot_id)
        {
            self.end("Herdr restarted while switching panes. Reconnect to continue.".into());
            return;
        }
        if self
            .pending
            .as_ref()
            .or(self.snapshot.as_ref())
            .is_some_and(|latest| {
                latest.boot_id == snapshot.boot_id && latest.revision > snapshot.revision
            })
        {
            return;
        }
        if !self.navigating()
            && let Some((old, surface)) = self.coherent()
            && spaceport_client::projection::same_input_projection(old, &snapshot)
            && surface.projection_revision != snapshot.revision
        {
            self.pending = Some(snapshot);
            return;
        }
        self.pending = None;
        if self.surface.as_ref().is_some_and(|f| {
            f.boot_id != snapshot.boot_id || f.projection_revision != snapshot.revision
        }) || self
            .snapshot
            .as_deref()
            .is_some_and(|s| !spaceport_client::projection::same_input_projection(s, &snapshot))
        {
            self.surface = None;
        }
        self.adopt_snapshot(snapshot);
        self.settle();
    }

    pub(crate) fn surface(&mut self, surface: Arc<PaneSurfaceFrame>) {
        // Frames sent before the daemon accepted deactivation belong to the old presentation.
        if !self.accepts_surfaces() {
            return;
        }

        if self
            .pending
            .as_ref()
            .or(self.snapshot.as_ref())
            .is_none_or(|s| {
                s.boot_id != surface.boot_id || s.revision != surface.projection_revision
            })
        {
            return;
        }
        if let Some(pending) = self.pending.take() {
            self.adopt_snapshot(pending);
        }
        self.surface = Some(surface);
        self.settle();
    }

    fn coherent(&self) -> Option<(&ClientShellSnapshot, &PaneSurfaceFrame)> {
        let snap = self.snapshot.as_deref()?;
        let surface = self.surface.as_deref()?;
        (snap.boot_id == surface.boot_id && snap.revision == surface.projection_revision)
            .then_some((snap, surface))
    }

    pub(crate) fn sized(&self) -> bool {
        self.surface.as_ref().is_some_and(|s| {
            s.frame.width == self.options.surface_size.cols
                && s.frame.height == self.options.surface_size.rows
        })
    }

    pub(crate) fn focus_interrupted(&self) -> bool {
        self.focus_interrupted || self.navigating() || self.go_to_pending()
    }

    pub(crate) fn navigating(&self) -> bool {
        self.fence.fenced()
    }

    pub(crate) fn initial_focus(&self) -> Option<&str> {
        if self.reported_focus || self.disconnected || self.navigating() || !self.is_active() {
            return None;
        }
        self.coherent()
            .map(|(snapshot, _)| snapshot.boot_id.as_str())
    }

    pub(crate) fn note_input(&mut self, now: Instant) {
        self.last_input = Some(now);
    }

    pub(crate) fn reassert_due(&mut self, now: Instant) -> bool {
        use spaceport_client::projection::{RESIZE_REASSERT, reasserts};
        if self.direct_attach
            || self.disconnected
            || !self.is_active()
            || self.navigating()
            || self.sized()
            || self.coherent().is_none()
            || !reasserts(self.last_input, now)
        {
            self.reassert_since = None;
            return false;
        }
        let since = self.reassert_since.get_or_insert(now);
        if now.saturating_duration_since(*since) < RESIZE_REASSERT {
            return false;
        }
        *since = now;
        true
    }

    /// The failure that ended this barrier, once it has one, folding in the
    /// deadline the fence itself carries.
    pub(crate) fn fence_failure(&mut self) -> Option<Failure> {
        self.fence.transition(Transition::Expire(Instant::now()));
        match &self.fence {
            Fence::Failed { cause, .. } => Some(*cause),
            _ => None,
        }
    }

    /// Registers the ordered barrier an accepted focus request is waiting on.
    pub(crate) fn start_fence(
        &mut self,
        request: String,
        action: impl Into<FocusAction>,
        barrier: Result<String, spaceport_client::SendError>,
    ) {
        let (focus, creating_from) = match action.into() {
            FocusAction::Select(target) => (Some(target), None),
            FocusAction::CreateTab { source_tab, .. } => (None, Some(source_tab)),
        };
        self.start_activation(request, focus, creating_from, barrier);
    }

    pub(crate) fn start_activation(
        &mut self,
        request: String,
        focus: Option<OwnedNavigationTarget>,
        creating_from: Option<String>,
        barrier: Result<String, spaceport_client::SendError>,
    ) {
        self.surface = None;
        if let Some(pending) = self.pending.take() {
            self.adopt_snapshot(pending);
        }
        self.focus_request = Some(FocusRequest {
            id: request,
            creating_from,
        });
        let boot = self.boot_id();
        self.barrier_queued(barrier, boot, focus);
    }

    /// The barrier the queue refused, waiting to be offered again.
    pub(crate) fn retry_barrier(&self) -> Option<(String, Option<OwnedNavigationTarget>)> {
        match &self.fence {
            Fence::Retry { boot, focus } => Some((boot.clone(), focus.clone())),
            _ => None,
        }
    }

    /// Records where an offered barrier went. A queue that refused it as full
    /// proves it never entered the ordered queue, so the fence keeps the
    /// destination and offers it again rather than ending the session.
    pub(crate) fn barrier_queued(
        &mut self,
        result: Result<String, spaceport_client::SendError>,
        boot: String,
        focus: Option<OwnedNavigationTarget>,
    ) {
        if let Err(error) = &result
            && !matches!(error, spaceport_client::Error::Full)
        {
            self.error = Some(error.to_string());
        }
        self.fence.transition(Transition::Queued {
            result,
            boot,
            focus,
            active: true,
            now: Instant::now(),
        });
    }

    fn boot_id(&self) -> String {
        self.snapshot
            .as_ref()
            .map(|snapshot| snapshot.boot_id.clone())
            .unwrap_or_default()
    }

    /// Opens the activation's surface once the projection both halves agree on
    /// is the one the barrier was acknowledged at, and holds the destination it
    /// was aimed at.
    fn settle(&mut self) {
        if self
            .focus_request
            .as_ref()
            .is_some_and(|r| r.creating_from.is_some())
        {
            return;
        }
        if let Some((snapshot, surface)) = self.coherent()
            && self.fence.ready(snapshot, surface)
        {
            self.fence.transition(Transition::Opened);
        }
        self.observe_pane();
    }

    pub(crate) fn receipt(&mut self, id: &str, payload: &serde_json::Value) {
        let result = response::result(payload);
        if self.parked_roster.as_mut().is_some_and(|check| {
            check.reply(
                id,
                result
                    .as_ref()
                    .ok()
                    .and_then(|r| surface_revision(r, false)),
            )
        }) {
            return;
        }
        if self.release_request.as_deref() == Some(id) {
            self.release_request = None;
            if result
                .ok()
                .and_then(|r| surface_revision(r, false))
                .is_none()
            {
                self.end("Herdr did not release the hidden terminal surface.".into());
            }
            return;
        }
        if self.resume_reply(
            id,
            result.as_ref().ok().and_then(|r| surface_revision(r, true)),
        ) {
            return;
        }
        if let Err(error) = &result {
            self.error = Some(error.to_string());
        }
        if let Some(request) = self.focus_request.take_if(|r| r.id == id) {
            if let Some(source_tab) = request.creating_from
                && !matches!(result, Err(spaceport_client::Error::Endpoint { .. }))
            {
                let Some(tab) = response::tagged(payload, "tab_created")
                    .ok()
                    .and_then(|r| r["tab"]["tab_id"].as_str())
                    .filter(|id| !id.is_empty() && *id != source_tab)
                else {
                    self.end("Herdr did not identify the new terminal. Reconnect to check the workspace.".into());
                    return;
                };
                self.fence
                    .transition(Transition::CreatedTab(tab.to_owned()));
            }
            // The requested pane did not acquire focus. The ordered barrier
            // still establishes which pane actually owns subsequent input, so
            // the fence keeps waiting for the barrier alone.
            if result.is_err() {
                self.route = None;
                self.fence.transition(Transition::Settled);
            }
            self.settle();
            return;
        }
        self.fence.transition(Transition::Response {
            request: id,
            result: result.ok(),
        });
        if matches!(
            self.fence,
            Fence::Failed {
                cause: Failure::InvalidResponse,
                ..
            }
        ) && self
            .fence
            .request()
            .is_some_and(|request| request.request == id)
        {
            self.end(
                "Herdr could not confirm the selected terminal. Reconnect to continue.".into(),
            );
            return;
        }
        self.settle();
    }

    pub(crate) fn rejected(&mut self, id: Option<&str>, message: String) {
        if let (Some(check), Some(id)) = (&mut self.parked_roster, id)
            && check.reply(id, None)
        {
            return;
        }
        if id.is_some() && self.release_request.as_deref() == id {
            self.end(message);
            return;
        }
        if id.is_some_and(|id| self.resume_reply(id, None)) {
            return;
        }
        if let Some(id) = id {
            self.fence.transition(Transition::Rejected(id));
        }
        if self.fence.failed() {
            // The barrier the daemon refused ends the session: nothing is left
            // to establish which pane owns input.
            self.end(message);
            return;
        }
        if self
            .focus_request
            .as_ref()
            .is_some_and(|r| Some(r.id.as_str()) == id)
        {
            self.focus_request = None;
            self.route = None;
            self.fence.transition(Transition::Settled);
        }
        self.error = Some(message);
        self.settle();
    }

    pub(crate) fn end(&mut self, reason: String) {
        self.focus_interrupted = self.focus_interrupted();
        self.disconnected = true;
        self.history = Default::default();
        self.fence = Fence::Open;
        self.focus_request = None;
        self.route = None;
        self.error = Some(reason);
    }

    pub(crate) fn target(&self) -> Option<InputTarget> {
        if self.route.is_some() || !self.is_active() {
            return None;
        }
        self.focused_target()
    }

    /// A matching snapshot can arrive before its surface. Hold the bytes until the pair agrees.
    pub(crate) fn can_read_direct(&self, target: &InputTarget) -> Result<bool, crate::MobileError> {
        if self.direct_target().as_ref() == Some(target) {
            return Ok(true);
        }
        if self.is_active()
            && !self.disconnected
            && !self.navigating()
            && !self.go_to_pending()
            && self.connection == target.connection
            && self.input_epoch == target.input_epoch
            && self.coherent().is_none()
            && self.snapshot.as_deref().is_some_and(|s| {
                s.boot_id == target.boot_id && s.focused_pane_id.as_ref() == Some(&target.pane_id)
            })
        {
            return Ok(false);
        }
        Err(crate::MobileError::StaleView)
    }

    pub(crate) fn direct_target(&self) -> Option<InputTarget> {
        if self.route.is_some() || !self.is_active() {
            return None;
        }
        self.coherent_target()
            .filter(|target| target.popup_id.is_none())
    }

    pub(crate) fn history_pane(
        &self,
        target: &InputTarget,
    ) -> Result<&PaneSurfacePane, crate::MobileError> {
        if target.popup_id.is_some() || self.direct_target().as_ref() != Some(target) {
            return Err(crate::MobileError::StaleView);
        }
        self.coherent()
            .and_then(|(_, frame)| frame.panes.iter().find(|p| p.pane_id == target.pane_id))
            .ok_or(crate::MobileError::StaleView)
    }

    fn focused_target(&self) -> Option<InputTarget> {
        let target = self.coherent_target()?;
        self.sized().then_some(target)
    }

    fn coherent_target(&self) -> Option<InputTarget> {
        if self.disconnected || self.navigating() {
            return None;
        }
        let (snapshot, surface) = self.coherent()?;
        let pane = snapshot.focused_pane_id.as_ref()?;
        if !surface
            .panes
            .iter()
            .any(|p| &p.pane_id == pane && p.focused)
        {
            return None;
        }
        Some(InputTarget {
            connection: self.connection,
            input_epoch: self.input_epoch,
            boot_id: snapshot.boot_id.clone(),
            pane_id: pane.clone(),
            popup_id: surface.popup.as_ref().map(|p| p.terminal_id.clone()),
        })
    }

    pub(crate) fn view(&self) -> MobileView {
        let s = self.snapshot.as_deref();
        let agents: HashMap<_, _> = s
            .into_iter()
            .flat_map(|s| &s.agents)
            .map(|a| (a.pane_id.as_str(), a))
            .collect();
        MobileView {
            back_pane: self.trail_destination(Step::Back),
            forward_pane: self.trail_destination(Step::Forward),
            pane_actions: (!self.disconnected
                && self.is_active()
                && !self.navigating()
                && !self.go_to_pending())
            .then_some(self.pane_actions),
            version: self.version,
            roster_observed_at_ms: self.snapshot.as_ref().map(|s| s.observed_at_ms),
            go_to_pending: self.route.is_some(),
            go_to_arrival: self.go_to_arrival,
            boot_id: s.map(|s| s.boot_id.clone()).unwrap_or_default(),
            workspaces: s
                .into_iter()
                .flat_map(|s| &s.workspaces)
                .map(|w| MobileWorkspace {
                    id: w.workspace_id.clone(),
                    label: w.label.clone(),
                    status: status(Some(w.agent_status)).into(),
                })
                .collect(),
            tabs: s
                .into_iter()
                .flat_map(|s| &s.tabs)
                .map(|t| MobileTab {
                    zoomed: t.zoomed,
                    id: t.tab_id.clone(),
                    workspace_id: t.workspace_id.clone(),
                    label: t.label.clone(),
                    status: status(Some(t.agent_status)).into(),
                })
                .collect(),
            panes: s
                .into_iter()
                .flat_map(|s| &s.panes)
                .map(|p| {
                    let agent = agents
                        .get(p.pane_id.as_str())
                        .copied()
                        .filter(|a| a.tab_id == p.tab_id && a.workspace_id == p.workspace_id);
                    let label = p
                        .label
                        .as_ref()
                        .or_else(|| {
                            agent.and_then(|a| a.title.as_ref().or(a.display_agent.as_ref()))
                        })
                        .unwrap_or(&p.pane_id);
                    let idle = agent
                        .and_then(|a| self.idle.get(&a.pane_id))
                        .filter(|_| self.is_active());
                    MobilePane {
                        agent_identity: agent
                            .and_then(|a| a.agent.as_ref())
                            .filter(|identity| identity.len() <= 256)
                            .cloned(),
                        terminal_title: agent.and_then(spaceport_truncate::agent_terminal_title),
                        idle_first_observed_ms: idle.map(|age| age.first_observed_ms),
                        idle_onset_observed: idle.is_some_and(|age| age.onset_observed),
                        state_change_seq: agent.map(|a| a.state_change_seq),
                        agent_name: agent.map(|a| spaceport_client::text::agent_name(a).to_owned()),
                        id: p.pane_id.clone(),
                        tab_id: p.tab_id.clone(),
                        label: label.clone(),
                        status: status(agent.map(|a| a.agent_status)).into(),
                    }
                })
                .collect(),
            pane_map: self.coherent().and_then(|(s, f)| pane_map::pane_map(s, f)),
            focused_workspace: s.and_then(|s| s.focused_workspace_id.clone()),
            focused_tab: s.and_then(|s| s.focused_tab_id.clone()),
            focused_pane: s.and_then(|s| s.focused_pane_id.clone()),
            terminal: self
                .coherent()
                .filter(|(_, frame)| !self.direct_attach || frame.popup.is_some())
                .and_then(|(s, f)| terminal(s, f)),
            input_target: if self.direct_attach {
                self.direct_target().or_else(|| self.target())
            } else {
                self.target()
            },
            error: self.error.clone(),
            disconnected: self.disconnected,
            host_challenge: self.challenge.clone(),
        }
    }
}

fn status(agent: Option<AgentStatus>) -> &'static str {
    match agent {
        Some(AgentStatus::Blocked) => "blocked",
        Some(AgentStatus::Working) => "working",
        Some(AgentStatus::Done) => "done",
        Some(AgentStatus::Idle) => "idle",
        Some(AgentStatus::Unknown) => "unknown",
        None => "",
    }
}

fn terminal(snapshot: &ClientShellSnapshot, surface: &PaneSurfaceFrame) -> Option<MobileTerminal> {
    let (frame, rect, scroll) = if let Some(popup) = &surface.popup {
        let frame = &popup.frame;
        (
            frame,
            SurfaceRect {
                x: 0,
                y: 0,
                width: frame.width,
                height: frame.height,
            },
            None,
        )
    } else {
        let focused = snapshot.focused_pane_id.as_ref()?;
        let pane = surface.panes.iter().find(|p| &p.pane_id == focused)?;
        let scroll = pane.scroll.map(|scroll| MobileScroll {
            offset_from_bottom: scroll.offset_from_bottom,
            max_offset_from_bottom: scroll.max_offset_from_bottom,
            viewport_rows: scroll.viewport_rows,
        });
        (&surface.frame, pane.inner_rect, scroll)
    };
    let right = rect.x.checked_add(rect.width)?;
    let bottom = rect.y.checked_add(rect.height)?;
    if right > frame.width || bottom > frame.height {
        return None;
    }
    let mut cells = Vec::with_capacity(usize::from(rect.width) * usize::from(rect.height));
    for row in rect.y..bottom {
        let start = usize::from(row) * usize::from(frame.width) + usize::from(rect.x);
        for cell in frame.cells.get(start..start + usize::from(rect.width))? {
            cells.push(MobileCell {
                symbol: cell.symbol.clone(),
                fg: cell.fg,
                bg: cell.bg,
                modifier: cell.modifier,
                skip: cell.skip,
                hyperlink: cell.hyperlink,
            });
        }
    }
    let cursor = frame
        .cursor
        .as_ref()
        .filter(|c| c.visible && (rect.x..right).contains(&c.x) && (rect.y..bottom).contains(&c.y));
    Some(MobileTerminal {
        width: rect.width,
        height: rect.height,
        cells,
        cursor_x: cursor.map(|c| c.x - rect.x),
        cursor_y: cursor.map(|c| c.y - rect.y),
        hyperlinks: frame.hyperlinks.clone(),
        scroll,
    })
}

fn surface_revision(result: &serde_json::Value, active: bool) -> Option<u64> {
    (result["type"] == "client_shell_surface_set" && result["active"] == active)
        .then(|| result["projection_revision"].as_u64())
        .flatten()
}
