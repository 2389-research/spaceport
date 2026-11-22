//! When an endpoint's notifications sound, adapted from Herdr's policy.
//!
//! Herdr sends every notification to each attached client, so a window plays
//! only what its own endpoint saw and only when the delay, the agent's newest
//! status, and the focus ask for it.

use spaceport_protocol::{
    AgentStatus, ClientShellSnapshot, SemanticNotification, SemanticNotificationKind as Kind,
};
use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// The most notifications the connection queues for the policy, and the most
/// the policy holds while they wait for their delay.
pub const MAX_PENDING: usize = 32;

/// What one endpoint's live state offers the policy for a single poll, and the
/// queues it drains. The app owns the state; this crate only reads it.
pub struct Live<'a> {
    /// Replaced on every connection generation; a new `Arc` retires the work
    /// queued against the old one.
    pub cancel: &'a Arc<AtomicBool>,
    /// Set when the connection itself is retiring.
    pub connection_cancel: &'a Arc<AtomicBool>,
    /// Whether the daemon is connected, as `ConnectionStatus::is_connected`.
    pub connected: bool,
    /// A settings reload the worker has not seen; `Service::poll` clears it.
    pub reload: &'a mut bool,
    /// Bounded by the connection at [`MAX_PENDING`]; drained here.
    pub events: &'a mut VecDeque<(Instant, SemanticNotification)>,
    /// The newest metadata, which may be ahead of the displayed pair.
    pub snapshot: Option<&'a ClientShellSnapshot>,
}

struct Pending {
    event: SemanticNotification,
    deadline: Instant,
    expires: Instant,
    validate: bool,
}

#[derive(Default)]
pub struct Policy {
    cancel: Option<Arc<AtomicBool>>,
    pending: VecDeque<Pending>,
    incoming: VecDeque<(Instant, SemanticNotification)>,
}

impl Policy {
    /// Sounds what `live` asks for, once each notification's delay has passed
    /// and its agent's newest status agrees, calling `emit` for each sound.
    pub fn poll(
        &mut self,
        live: Live<'_>,
        delay: Option<u64>,
        focused: bool,
        now: Instant,
        mut emit: impl FnMut(SemanticNotification),
    ) {
        if self
            .cancel
            .as_ref()
            .is_none_or(|old| !Arc::ptr_eq(old, live.cancel))
        {
            self.pending.clear();
            self.incoming.clear();
            self.cancel = Some(Arc::clone(live.cancel));
        }
        if live.cancel.load(Ordering::Acquire)
            || live.connection_cancel.load(Ordering::Acquire)
            || !live.connected
        {
            self.pending.clear();
            self.incoming.clear();
            live.events.clear();
            return;
        }
        for event in live.events.drain(..) {
            if self.incoming.len() == MAX_PENDING {
                self.incoming.pop_front();
            }
            self.incoming.push_back(event);
        }
        let Some(delay) = delay else {
            return;
        };
        for (received, event) in std::mem::take(&mut self.incoming) {
            if let Some(pane) = &event.pane_id {
                self.pending
                    .retain(|p| p.event.pane_id.as_ref() != Some(pane));
            }
            let delay = if event.kind == Kind::Custom { 0 } else { delay };
            if self.pending.len() == MAX_PENDING {
                self.pending.pop_front();
            }
            self.pending.push_back(Pending {
                validate: delay > 0 || event.kind == Kind::Finished,
                event,
                deadline: received + Duration::from_secs(delay),
                expires: received + Duration::from_secs(1),
            });
            self.tick(live.snapshot, focused, received, &mut emit);
        }
        self.tick(live.snapshot, focused, now, emit);
    }

    /// How many notifications still wait for their delay. The tests that mute
    /// one mid-wait read this; the app polls and observes what plays.
    #[cfg(any(test, feature = "test-support"))]
    pub fn waiting(&self) -> usize {
        self.pending.len()
    }

    /// Forgets the notifications still waiting whose workspace or tab the
    /// user has muted since they arrived.
    pub(crate) fn forget(&mut self, silenced: impl Fn(&SemanticNotification) -> bool) {
        self.incoming.retain(|(_, event)| !silenced(event));
        self.pending.retain(|pending| !silenced(&pending.event));
    }

    fn tick(
        &mut self,
        snapshot: Option<&ClientShellSnapshot>,
        focused: bool,
        now: Instant,
        mut emit: impl FnMut(SemanticNotification),
    ) {
        for mut pending in std::mem::take(&mut self.pending) {
            if pending.deadline > now {
                self.pending.push_back(pending);
                continue;
            }
            let event = &pending.event;
            if pending.validate {
                let status = event
                    .pane_id
                    .as_ref()
                    .and_then(|pane| snapshot?.agents.iter().find(|a| &a.pane_id == pane))
                    .map(|a| a.agent_status);
                let awaiting = event.pane_id.is_some()
                    && (status.is_none()
                        || (event.kind == Kind::Finished && status == Some(AgentStatus::Working)));
                if awaiting && now < pending.expires {
                    pending.deadline = (now + Duration::from_millis(50)).min(pending.expires);
                    self.pending.push_back(pending);
                    continue;
                }
                let valid = match event.kind {
                    Kind::Finished => status == Some(AgentStatus::Done),
                    Kind::NeedsAttention => {
                        event.pane_id.is_none() || status == Some(AgentStatus::Blocked)
                    }
                    Kind::Custom | Kind::UpdateInstalled => !awaiting,
                };
                if !valid {
                    continue;
                }
            }
            let target_active = snapshot.is_some_and(|s| {
                if let Some(tab) = &event.tab_id {
                    s.focused_tab_id.as_ref() == Some(tab)
                } else {
                    event
                        .workspace_id
                        .as_ref()
                        .is_some_and(|workspace| s.focused_workspace_id.as_ref() == Some(workspace))
                }
            });
            if event.sound.is_some() && !(event.kind == Kind::Finished && focused && target_active)
            {
                emit(pending.event);
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
