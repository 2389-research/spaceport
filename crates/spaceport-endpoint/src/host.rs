//! One host's link machine and the live state it is read from.
//!
//! [`Host`] is an endpoint once its socket is taken out: the [`Link`](link::Link) it is in,
//! the [`LiveState`] its connection last reported, and the retry counter
//! between them. It holds no handle and opens nothing. Every change enters as
//! a [`Transition`] and comes back as the [`Action`] the window performs on the
//! bridge it keeps, so no line here is a socket, a toast or a word.

use crate::Trigger;
use spaceport_client::ConnectTarget;
use spaceport_live::LiveState;
use std::time::{Duration, Instant};

mod fence;
// `pub(crate)` so the crate root can hand the app's fixtures `Link` and
// `Failure` under `test-support`; the machine itself is this module's.
pub(crate) mod link;
mod live_of;
mod phase;

pub use fence::{ConnectionRef, InputFence, SelectionFence, WeakConnectionRef};
pub use link::Transition;
pub use live_of::live_of;
pub use phase::Phase;

#[cfg(any(test, feature = "test-support"))]
mod fixtures;

/// What the link machine decided the window must do on a host's socket once a
/// transition was entered. The window performs every one of these; none of them
/// happens here. A dial is not one: the window decides to dial by calling
/// `connect`, and the [`Transition::Connect`] it enters only records the attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// The link changed and no transport must.
    None,
    /// Retire the current transport: detach the bridge, and take the state it
    /// leaves as the host's.
    Detach,
    /// Throw the transport away and start one for `target`.
    Retire { target: ConnectTarget },
    /// Ask the live transport to prove itself, which a healthy one does in a
    /// round trip.
    CheckLiveness,
}

/// What a poll can see of a host's socket without blocking: the facts the
/// link's rules need and [`Host`] does not hold, because the bridge they come
/// from is the window's.
#[derive(Clone, Copy, Debug)]
pub struct Transport {
    /// A client handle exists for this host's current attempt.
    pub handled: bool,
    /// That handle's worker has not reported itself ended.
    pub alive: bool,
    /// This host's daemon may be started from here, or it is a remote one this
    /// app may attach to: the app's `daemon::may_start` plus the SSH and WSL
    /// targets, whose stop can be recovered by asking them again.
    pub can_restart: bool,
}

/// One saved, local or WSL host as the window drives it.
pub struct Host {
    link: link::Link,
    live: LiveState,
    attempts: u32,
}

impl Host {
    pub fn new(enabled: bool) -> Self {
        Self {
            link: link::Link::initial(enabled),
            live: LiveState::default(),
            attempts: 0,
        }
    }

    /// The state this host's connection last reported. The window reads its own
    /// copy of the selected host's, not this one, while a poll is pending.
    pub fn live(&self) -> &LiveState {
        &self.live
    }

    /// The window's poll and the fixtures write here, as the readings below
    /// take their values from it.
    pub fn live_mut(&mut self) -> &mut LiveState {
        &mut self.live
    }

    /// The stored lifecycle, independent of a window's older worker projection.
    pub fn phase(&self) -> Phase {
        phase::of(&self.link)
    }

    pub fn enabled(&self) -> bool {
        !matches!(self.link, link::Link::Disabled)
    }

    pub fn detached(&self) -> bool {
        matches!(self.link, link::Link::Detached { .. })
    }

    /// When this host may dial again, `now` when it may dial at once.
    pub fn retry_at(&self) -> Instant {
        self.link.retry_at().unwrap_or_else(Instant::now)
    }

    /// Whether a live transport has delivered a snapshot. Narrower than
    /// [`Phase::Online`], which a link still waiting for its first snapshot
    /// already is.
    pub fn serving(&self, transport: Transport) -> bool {
        transport.alive && self.live.status.is_connected() && self.live.snapshot.is_some()
    }

    /// Whether this host wants a connection and has no transport. `retry_ready`
    /// is the window's reading of the inbox: drained of the worker's final
    /// events, so a dial now would not race them.
    pub fn may_dial(&self, retry_ready: bool) -> bool {
        matches!(self.link, link::Link::Waiting { .. }) && !self.herdr_stopped() && retry_ready
    }

    /// The backoff this host waits before its next attempt, doubling per attempt
    /// up to thirty seconds.
    pub fn retry_delay(&self) -> Duration {
        link::retry_delay(self.attempts)
    }

    /// After a sleep or a network change, once this poll has taken what the
    /// worker reported. A live link is asked to prove itself, which a healthy
    /// one does in a round trip; a changed path does nothing more, since such
    /// reports also come for changes no connection minds.
    ///
    /// After a sleep or a network coming back, failures before it, often from
    /// dark wakes with no network, say nothing about the network now, so the
    /// backoff starts over. A remote attempt still dialing was started with
    /// no usable path and may hang until its timeout, so it is abandoned and
    /// dialed again, unless the worker has reported it up since the update
    /// was taken. A dropped host is dialed at once, except one refused for a
    /// reason only the user can fix. Detached and disabled hosts stay as they
    /// are.
    ///
    /// `None` asks to be called again next poll, when the worker's report
    /// cannot be read without blocking: `reports_connected` runs only when the
    /// rules need it, and never waits on the inbox.
    pub fn recover(
        &mut self,
        trigger: Trigger,
        now: Instant,
        target: &ConnectTarget,
        transport: Transport,
        reports_connected: impl FnOnce() -> Option<bool>,
    ) -> Option<Action> {
        let thorough = trigger >= Trigger::NetworkRestored;
        let networked = target.is_remote() && !matches!(target, ConnectTarget::Wsl { .. });
        let mut retired = false;
        if thorough && networked && transport.handled && !self.live.status.is_connected() {
            match reports_connected() {
                None => return None,
                Some(true) => {}
                Some(false) => {
                    retired = true;
                    let _ = self.enter(Transition::Stop);
                }
            }
        }
        if thorough {
            let _ = self.enter(Transition::ResetAttempts);
        }
        // A stopped attempt has no transport left, which is what the window's
        // `Action::Detach` will have done by the time it redials.
        let handled = transport.handled && !retired;
        if !handled && thorough && !self.waits_for_user() {
            let _ = self.enter(Transition::RetryAt(self.retry_at().min(now)));
        }
        Some(if retired {
            Action::Detach
        } else if handled {
            Action::CheckLiveness
        } else {
            Action::None
        })
    }
}
