//! The notices a connection's daemon sends, as bounded presentation data
//! validated against a snapshot before anything acts on one: a click resolves
//! its target through the same boot fence a toast uses. Which notice is shown
//! when, and every word in it, are the app's.

use crate::{Labels, NavigationTarget, OwnedNavigationTarget};
use spaceport_client::protocol::{
    ClientShellSnapshot, SemanticNotification, SemanticNotificationKind, ToastHerdrPosition,
};
use spaceport_client::text::safe_text;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// How many notices a connection queues before the oldest waiting one goes.
pub const PENDING_LIMIT: usize = 8;
/// How many toasts one window shows at a time.
pub const VISIBLE_LIMIT: usize = 1;
/// How long a notice waits for the agent's state to confirm it.
pub const GRACE: Duration = Duration::from_secs(1);
/// The shortest wait between two reads of that state.
pub const RECHECK: Duration = Duration::from_millis(50);
static ARRIVAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Where a notice is in its life. Each stage carries only what means
/// something in it. The app's scheduler moves a waiting notice to ready,
/// [`Notice::promote`] shows it and [`Notice::post`] hands it to the OS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Waiting out its delay and for the agent's state to confirm it.
    /// `checked` is when that evidence was last read.
    Waiting { checked: Option<Instant> },
    /// Passed the policy: queued for a toast, or for the OS notification
    /// center under system delivery.
    Ready,
    /// Shown as a toast until `expires`. `displayed` is the last tick that
    /// saw it and whether it was paused then, which extends its lifetime.
    Visible {
        expires: Instant,
        displayed: (Instant, bool),
    },
    /// Handed to the OS notification center. Retained only so a click can
    /// resolve its target through the same boot-fenced validation as a toast.
    Posted,
}

#[derive(Clone)]
pub struct Notice {
    pub title: String,
    pub body: Option<String>,
    pub kind: SemanticNotificationKind,
    /// The sending agent's identity, when Herdr named one. The mark it is
    /// drawn as belongs to the app, which maps it when it paints.
    pub agent: Option<String>,
    pub position: ToastHerdrPosition,
    pub workspace_id: Option<String>,
    pub tab_id: Option<String>,
    pub pane_id: Option<String>,
    pub boot: Option<String>,
    pub initialized: bool,
    pub arrived: Instant,
    pub order: u64,
    pub explicit_position: bool,
    pub client_local: bool,
    pub stage: Stage,
    pub suppression_target: Option<OwnedNavigationTarget>,
    /// The body is the client's task and place lines, each shown on one row.
    pub described: bool,
}

impl Notice {
    /// Local diagnostics carry recovery instructions that must not be clipped
    /// to the daemon toast's short body preview.
    pub fn is_local_feedback(&self) -> bool {
        self.client_local
    }

    pub fn preview(mut self) -> Self {
        self.client_local = true;
        self
    }

    /// A QA preview shows the app's own messages as written. `new` cleans
    /// daemon text with `safe_text`, which also drops the direction marks a
    /// right-to-left language (and `en-XB`) relies on, so the preview's
    /// title and body are set from the messages after it.
    pub fn with_message(mut self, title: String, body: String) -> Self {
        self.title = title;
        self.body = Some(body);
        self
    }

    /// Feedback for a user action is not governed by daemon notification mutes,
    /// activity evidence, or focused-pane suppression. It still uses the queue.
    pub fn local_feedback(
        notification: SemanticNotification,
        now: Instant,
        labels: &Labels,
    ) -> Self {
        let mut notice = Self::new(notification, now, labels);
        notice.client_local = true;
        notice
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    pub fn local_feedback_multiline(
        mut notification: SemanticNotification,
        now: Instant,
        labels: &Labels,
    ) -> Self {
        // Local recovery instructions may put a copyable command on its own
        // line. Keep those line breaks, with the same scan bound and all other
        // control/direction characters removed as for daemon notifications.
        let body = notification
            .body
            .take()
            .map(|body| {
                body.chars()
                    .take(512)
                    .filter(|c| *c == '\n' || !spaceport_client::text::unsafe_char(*c))
                    .collect::<String>()
            })
            .filter(|body| !body.trim().is_empty());
        let mut notice = Self::local_feedback(notification, now, labels);
        notice.body = body;
        notice
    }

    /// Shows the notice as a toast, whose lifetime starts at `now`.
    pub fn promote(&mut self, now: Instant) {
        self.stage = Stage::Visible {
            expires: now
                + Duration::from_secs(match self.kind {
                    SemanticNotificationKind::NeedsAttention => 8,
                    SemanticNotificationKind::Finished | SemanticNotificationKind::Custom => 5,
                    SemanticNotificationKind::UpdateInstalled => 3,
                }),
            displayed: (now, false),
        };
    }

    /// Records the notice as handed to the OS notification center. The app's
    /// system-delivery scheduler posts ready notices; a QA preview posts its own.
    pub fn post(&mut self) {
        self.stage = Stage::Posted;
    }

    /// Still waiting out its delay or for the agent's state to confirm it.
    pub fn is_waiting(&self) -> bool {
        matches!(self.stage, Stage::Waiting { .. })
    }

    /// Passed the policy and neither shown nor posted yet.
    pub fn is_ready(&self) -> bool {
        self.stage == Stage::Ready
    }

    /// Shown as a toast.
    pub fn is_visible(&self) -> bool {
        matches!(self.stage, Stage::Visible { .. })
    }

    /// Handed to the OS notification center.
    pub fn is_posted(&self) -> bool {
        self.stage == Stage::Posted
    }

    /// When a showing toast leaves. A notice in any other stage has no
    /// lifetime.
    pub fn expires(&self) -> Option<Instant> {
        match self.stage {
            Stage::Visible { expires, .. } => Some(expires),
            _ => None,
        }
    }

    /// Moves a showing toast's expiry; a test's way to age one. Only the app's
    /// headless Unix lifecycle tests age a toast.
    #[cfg(all(feature = "test-support", unix))]
    pub fn set_expires(&mut self, at: Instant) {
        if let Stage::Visible { expires, .. } = &mut self.stage {
            *expires = at;
        }
    }

    pub fn new(notification: SemanticNotification, now: Instant, labels: &Labels) -> Self {
        let title = safe_text(&notification.title, 160);
        let suppression_target = notification
            .tab_id
            .clone()
            .map(NavigationTarget::Tab)
            .or_else(|| {
                notification
                    .workspace_id
                    .clone()
                    .map(NavigationTarget::Workspace)
            });
        Self {
            title: if title.trim().is_empty() {
                labels.notification_untitled.clone()
            } else {
                title
            },
            body: notification
                .body
                .map(|body| safe_text(&body, 512))
                .filter(|body| !body.trim().is_empty()),
            kind: notification.kind,
            // Identities are short labels; a longer one names no mark.
            agent: notification
                .agent
                .as_deref()
                .filter(|agent| agent.len() <= 32)
                .map(str::to_ascii_lowercase),
            position: notification
                .position
                .unwrap_or(ToastHerdrPosition::BottomRight),
            workspace_id: notification.workspace_id,
            tab_id: notification.tab_id,
            pane_id: notification.pane_id,
            boot: None,
            initialized: false,
            arrived: now,
            order: ARRIVAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            explicit_position: notification.position.is_some(),
            client_local: false,
            stage: Stage::Waiting { checked: None },
            suppression_target,
            described: false,
        }
    }

    pub fn with_snapshot(
        mut self,
        snapshot: Option<&ClientShellSnapshot>,
        labels: &Labels,
    ) -> Self {
        if let Some(snapshot) = snapshot {
            self.initialize(snapshot, labels);
        }
        self
    }

    /// Resolves this notice against `snapshot`, retaining inferred parents too,
    /// so a later reparenting cannot retarget a click.
    pub fn initialize(&mut self, snapshot: &ClientShellSnapshot, labels: &Labels) {
        if self.initialized
            || self
                .boot
                .as_ref()
                .is_some_and(|boot| *boot != snapshot.boot_id)
        {
            return;
        }
        self.boot.get_or_insert_with(|| snapshot.boot_id.clone());
        if self.resolve_target(snapshot).is_some() {
            if let Some(pane) = snapshot
                .panes
                .iter()
                .find(|p| Some(&p.pane_id) == self.pane_id.as_ref())
            {
                self.tab_id = Some(pane.tab_id.clone());
                self.workspace_id = Some(pane.workspace_id.clone());
            } else if let Some(tab) = snapshot
                .tabs
                .iter()
                .find(|t| Some(&t.tab_id) == self.tab_id.as_ref())
            {
                self.workspace_id = Some(tab.workspace_id.clone());
            }
        }
        self.initialized = self.resolve_target(snapshot).is_some();
        // Name the agent's task and place from the same validated snapshot.
        // Custom notices carry the sender's own text, which stays.
        if self.initialized
            && !self.client_local
            && matches!(
                self.kind,
                SemanticNotificationKind::NeedsAttention | SemanticNotificationKind::Finished
            )
            && let Some(pane) = self.pane_id.as_deref()
            && let Some(body) = (labels.describe)(snapshot, pane)
        {
            self.body = Some(body);
            self.described = true;
        }
    }

    pub fn target<'a>(
        &'a self,
        snapshot: &ClientShellSnapshot,
    ) -> Option<NavigationTarget<&'a str>> {
        if !self.initialized {
            return None;
        }
        self.resolve_target(snapshot)
    }

    fn resolve_target<'a>(
        &'a self,
        snapshot: &ClientShellSnapshot,
    ) -> Option<NavigationTarget<&'a str>> {
        if self.boot.as_deref() != Some(snapshot.boot_id.as_str()) {
            return None;
        }
        let workspace = self.workspace_id.as_deref();
        let tab = self.tab_id.as_deref();
        if let Some(id) = self.pane_id.as_deref() {
            let pane = snapshot.panes.iter().find(|p| p.pane_id == id)?;
            if workspace.is_some_and(|w| w != pane.workspace_id)
                || tab.is_some_and(|t| t != pane.tab_id)
            {
                return None;
            }
            let parent = snapshot
                .tabs
                .iter()
                .find(|t| t.tab_id == pane.tab_id && t.workspace_id == pane.workspace_id)?;
            snapshot
                .workspaces
                .iter()
                .find(|w| w.workspace_id == parent.workspace_id)?;
            return Some(NavigationTarget::Pane(id));
        }
        if let Some(id) = tab {
            let tab = snapshot.tabs.iter().find(|t| t.tab_id == id)?;
            if workspace.is_some_and(|w| w != tab.workspace_id) {
                return None;
            }
            snapshot
                .workspaces
                .iter()
                .find(|w| w.workspace_id == tab.workspace_id)?;
            return Some(NavigationTarget::Tab(id));
        }
        let id = workspace?;
        snapshot.workspaces.iter().find(|w| w.workspace_id == id)?;
        Some(NavigationTarget::Workspace(id))
    }
}

/// One connection's queued notices, bounded and deduplicated by pane.
#[derive(Default)]
pub struct Toasts {
    pub entries: VecDeque<(u64, Notice)>,
    next_id: u64,
    pub enabled_since: Option<Instant>,
}

impl Toasts {
    pub fn receive(&mut self, notices: impl IntoIterator<Item = Notice>) {
        for notice in notices {
            if !notice.client_local
                && self
                    .enabled_since
                    .is_some_and(|cutoff| notice.arrived <= cutoff)
            {
                continue;
            }
            if let Some(pane) = &notice.pane_id {
                self.entries
                    .retain(|(_, old)| old.pane_id.as_ref() != Some(pane));
            }
            if self.entries.len() == PENDING_LIMIT * 2 + 1
                && let Some(index) = self.entries.iter().position(|(_, n)| !n.is_visible())
            {
                self.entries.remove(index);
            }
            self.entries.push_back((self.next_id, notice));
            self.next_id = self.next_id.wrapping_add(1);
        }
    }

    pub fn dismiss(&mut self, id: u64) {
        self.entries.retain(|(entry, _)| *entry != id);
    }
}
