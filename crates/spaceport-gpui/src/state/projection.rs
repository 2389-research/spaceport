//! Keep a coherent displayed pair while newer metadata waits for its surface.
use super::{ConnectionStatus, LiveState, coherent};
use spaceport_client::{
    SurfaceImages,
    projection::same_input_projection,
    protocol::{ClientShellSnapshot, PaneSurfaceFrame},
};
use std::sync::Arc;

impl LiveState {
    pub(crate) fn agent_idle(
        &self,
        pane: &str,
    ) -> Option<spaceport_client::agent_idle::IdleObservation> {
        self.idle_tracker.get(pane)
    }

    pub(crate) fn agent_confirmed_at(&self, now_ms: u64) -> u64 {
        if self.status.is_connected() {
            now_ms
        } else {
            self.last_agent_snapshot_ms
        }
    }
    /// The newest received metadata, which may be ahead of the displayed pair.
    /// Notices, sounds, and notice clicks read it; terminal
    /// rendering, hit testing, and input stay on `snapshot`.
    pub(crate) fn latest_snapshot(&self) -> Option<&Arc<ClientShellSnapshot>> {
        self.pending_snapshot.as_ref().or(self.snapshot.as_ref())
    }

    /// What the sound policy reads of this endpoint for one poll: the
    /// connection's two cancellation tokens, whether it is connected, the
    /// queue it drains, and the newest metadata the caller resolved, since
    /// [`Self::latest_snapshot`] borrows `&self`.
    pub(crate) fn sound_live<'a>(
        &'a mut self,
        snapshot: Option<&'a ClientShellSnapshot>,
    ) -> spaceport_sound::Live<'a> {
        spaceport_sound::Live {
            cancel: &self.sound_cancel,
            connection_cancel: &self.sound_connection_cancel,
            connected: self.status.is_connected(),
            reload: &mut self.reload_sound,
            events: &mut self.sound_events,
            snapshot,
        }
    }

    pub(crate) fn invalidate_surface(&mut self) {
        self.surface = None;
        if let Some(snapshot) = self.pending_snapshot.take() {
            self.commit_snapshot(snapshot);
        }
        self.dirty = true;
    }

    pub(super) fn receive_snapshot(&mut self, snapshot: Arc<ClientShellSnapshot>) {
        if self.latest_snapshot().is_some_and(|latest| {
            latest.boot_id == snapshot.boot_id && latest.revision > snapshot.revision
        }) {
            return;
        }
        if self.surface_ready()
            && self
                .snapshot
                .as_ref()
                .is_some_and(|old| same_input_projection(old, &snapshot))
            && self
                .surface
                .as_ref()
                .is_some_and(|surface| surface.projection_revision != snapshot.revision)
        {
            self.pending_snapshot = Some(snapshot);
            return;
        }
        self.pending_snapshot = None;
        self.commit_snapshot(snapshot);
    }

    pub(super) fn receive_surface(&mut self, surface: Arc<PaneSurfaceFrame>) {
        if let Some(pending) = &self.pending_snapshot {
            if !coherent(pending, &surface) {
                return;
            }
            let snapshot = self.pending_snapshot.take();
            if let Some(snapshot) = snapshot {
                self.commit_snapshot(snapshot);
            }
        }
        if self
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| coherent(snapshot, &surface))
        {
            if let Some(images) = self.pending_surface_images.take() {
                self.surface_images = images;
            }
            self.surface = Some(surface);
        }
    }

    pub(super) fn receive_surface_images(&mut self, images: Arc<SurfaceImages>) {
        if self.surface.is_some() {
            // Session emits pixels before their surface. A UI poll between
            // those events must keep the old picture's own asset catalog.
            self.pending_surface_images = Some(images);
        } else {
            self.surface_images = images;
            self.pending_surface_images = None;
        }
    }

    fn commit_snapshot(&mut self, snapshot: Arc<ClientShellSnapshot>) {
        let boot_changed = self
            .snapshot
            .as_ref()
            .is_some_and(|old| old.boot_id != snapshot.boot_id);
        if boot_changed {
            self.notifications.clear();
            self.cancel_sounds();
            self.sound_cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
            self.bells = 0;
            self.window_title = None;
            self.surface_images = Default::default();
            self.pending_surface_images = None;
        }
        self.activation
            .transition(super::activation::Transition::Boot(&snapshot.boot_id));
        self.missing_installation = false;
        if self
            .surface
            .as_ref()
            .is_some_and(|surface| !coherent(&snapshot, surface))
            || self
                .snapshot
                .as_ref()
                .is_some_and(|old| !same_input_projection(old, &snapshot))
        {
            self.surface = None;
        }
        if self
            .announcement_dismissal
            .as_ref()
            .is_some_and(|dismissal| !dismissal.covers(&snapshot))
        {
            self.announcement_dismissal = None;
        }
        if self.notices.connection_reason().is_some() {
            self.notices.clear();
        }
        self.status = ConnectionStatus::Connected;
        self.previous_pane = match &self.snapshot {
            Some(old) if old.boot_id == snapshot.boot_id => old
                .focused_pane_id
                .clone()
                .filter(|old| Some(old) != snapshot.focused_pane_id.as_ref())
                .or_else(|| self.previous_pane.take()),
            _ => None,
        };
        let previous = self
            .snapshot
            .as_deref()
            .filter(|old| old.boot_id == snapshot.boot_id);
        self.agent_recency.observe(previous, &snapshot);
        self.last_agent_snapshot_ms = spaceport_client::agent_idle::now_ms();
        self.idle_tracker
            .observe(&snapshot, self.last_agent_snapshot_ms);
        self.snapshot = Some(snapshot);
    }
}
