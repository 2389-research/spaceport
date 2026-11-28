//! The link state machine: which attempt a host is on, what went wrong, and the
//! retry curve that follows.
use super::{Action, Host, Transport};
use spaceport_client::{ConnectTarget, VersionMismatch};
use spaceport_live::{ConnectionStatus, LiveState};
use std::time::{Duration, Instant};

/// Upstream rechecks failed SSH machines every 30 seconds, so authentication
/// repaired outside the app (a new master, a loaded key) is picked up promptly.
pub const MAX_RETRY_DELAY: Duration = Duration::from_secs(30);
/// A link up this long has proved itself, so its backoff starts over.
pub const STABLE_CONNECTION_PERIOD: Duration = Duration::from_secs(60);

pub enum Link {
    Disabled,
    Detached {
        retry_at: Instant,
        why: Option<Failure>,
    },
    Waiting {
        retry_at: Instant,
        why: Option<Failure>,
    },
    Dialing {
        retry_at: Instant,
        why: Option<Failure>,
    },
    Up {
        retry_at: Instant,
        since: Option<Instant>,
    },
}

#[derive(Clone)]
pub enum Failure {
    Transport {
        reason: Option<String>,
        needs_user: bool,
    },
    Missing {
        reason: Option<String>,
    },
    Stopped {
        reason: Option<String>,
    },
    Version {
        mismatch: VersionMismatch,
        reason: Option<String>,
    },
}

impl Failure {
    fn reason(&self) -> Option<&str> {
        match self {
            Self::Transport { reason, .. }
            | Self::Missing { reason }
            | Self::Stopped { reason }
            | Self::Version { reason, .. } => reason.as_deref(),
        }
    }

    fn with_reason(mut self, reason: Option<String>) -> Self {
        match &mut self {
            Self::Transport { reason: old, .. }
            | Self::Missing { reason: old }
            | Self::Stopped { reason: old }
            | Self::Version { reason: old, .. } => *old = reason,
        }
        self
    }

    fn needs_user(&self) -> bool {
        matches!(
            self,
            Self::Missing { .. }
                | Self::Stopped { .. }
                | Self::Transport {
                    needs_user: true,
                    ..
                }
        )
    }
}

impl Link {
    pub(super) fn initial(enabled: bool) -> Self {
        if enabled {
            Self::Waiting {
                retry_at: Instant::now(),
                why: None,
            }
        } else {
            Self::Disabled
        }
    }

    fn why(&self) -> Option<&Failure> {
        match self {
            Self::Detached { why, .. } | Self::Waiting { why, .. } | Self::Dialing { why, .. } => {
                why.as_ref()
            }
            Self::Disabled | Self::Up { .. } => None,
        }
    }

    pub(super) fn retry_at(&self) -> Option<Instant> {
        match self {
            Self::Detached { retry_at, .. }
            | Self::Waiting { retry_at, .. }
            | Self::Dialing { retry_at, .. }
            | Self::Up { retry_at, .. } => Some(*retry_at),
            Self::Disabled => None,
        }
    }
}

/// Every way a host's link can move. The window enters one and performs the
/// [`Action`] it comes back with; nothing here touches a transport.
pub enum Transition<'a> {
    Stop,
    Detach,
    /// The saved entry named another target, or changed whether it is wanted.
    Retarget {
        target: ConnectTarget,
        enabled: bool,
    },
    /// The window is dialing: one more attempt, and the link dials until a
    /// poll says otherwise.
    Connect,
    Observe {
        now: Instant,
        transport: Transport,
    },
    Update {
        state: &'a mut LiveState,
        now: Instant,
        transport: Transport,
    },
    /// The transport's worker ended. `lost` is the app's own word for a loss
    /// its notices cannot name.
    Lost {
        now: Instant,
        lost: String,
    },
    RetryAt(Instant),
    ResetAttempts,
    /// Another client's handshake replaced this host's live state, swapped in by
    /// the window before it enters.
    Trade(Transport),
    #[cfg(any(test, feature = "test-support"))]
    Fixture(Link),
}

/// Whether `live` ended for a reason only the user can fix.
fn needs_user(live: &LiveState) -> bool {
    live.missing_installation || live.ssh_failure.is_some_and(|failure| failure.needs_user())
}

/// The backoff before attempt `attempts` + 1: half a second, doubling per
/// attempt, capped at thirty seconds.
pub(super) fn retry_delay(attempts: u32) -> Duration {
    Duration::from_millis(500u64 << attempts.min(8)).min(MAX_RETRY_DELAY)
}

impl Host {
    /// The only writer of `link`; every transport change enters here and comes
    /// back as the action the window performs on its socket.
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 183 lines")]
    pub fn enter(&mut self, transition: Transition<'_>) -> Action {
        let retry_at = self.link.retry_at().unwrap_or_else(Instant::now);
        let why = self.link.why().cloned();
        let next = match transition {
            Transition::Stop | Transition::Detach => {
                let detached = matches!(transition, Transition::Detach) || self.detached();
                let next = if !self.enabled() {
                    Link::Disabled
                } else if detached {
                    Link::Detached {
                        retry_at,
                        why: why.and_then(|why| match why {
                            Failure::Transport { .. } => None,
                            why => Some(why.with_reason(None)),
                        }),
                    }
                } else {
                    Link::Waiting { retry_at, why }
                };
                self.link = next;
                return Action::Detach;
            }
            Transition::Retarget { target, enabled } => {
                self.attempts = 0;
                self.link = Link::initial(enabled);
                return Action::Retire { target };
            }
            Transition::Connect => {
                self.attempts = self.attempts.saturating_add(1);
                self.link = Link::Dialing {
                    retry_at: Instant::now() + self.retry_delay(),
                    why,
                };
                return Action::None;
            }
            Transition::Observe { now, transport } => {
                if !self.enabled() || self.detached() {
                    return Action::None;
                }
                let mut retry_at = retry_at;
                // Only a retired handle starts an outage. A worker diagnostic
                // can refine that outage after its delayed inbox update arrives.
                let reason = self.outage().map(|reason| {
                    if !transport.handled && !self.live.status.is_connected() {
                        self.live
                            .notices
                            .connection_reason()
                            .map(str::to_owned)
                            .unwrap_or_else(|| reason.into())
                    } else {
                        reason.into()
                    }
                });
                let why = if let Some(mismatch) = &self.live.version_mismatch {
                    Some(Failure::Version {
                        mismatch: mismatch.clone(),
                        reason,
                    })
                } else if self.live.server_stopped && transport.can_restart {
                    Some(Failure::Stopped {
                        reason: reason
                            .or_else(|| self.live.notices.connection_reason().map(str::to_owned)),
                    })
                } else if self.live.missing_installation {
                    Some(Failure::Missing { reason })
                } else if self.live.status.is_connected() {
                    // An accepted handshake replaces update/install advice immediately.
                    self.outage().map(|reason| Failure::Transport {
                        reason: Some(reason.into()),
                        needs_user: false,
                    })
                } else if let Some(why) = why {
                    match why {
                        Failure::Transport { .. }
                            if reason.is_none()
                                && self.live.notices.connection_reason().is_none()
                                && !needs_user(&self.live) =>
                        {
                            None
                        }
                        Failure::Transport { .. } => Some(Failure::Transport {
                            reason,
                            needs_user: needs_user(&self.live),
                        }),
                        why => Some(why.with_reason(reason)),
                    }
                } else if self.live.notices.connection_reason().is_some() || needs_user(&self.live)
                {
                    Some(Failure::Transport {
                        reason,
                        needs_user: needs_user(&self.live),
                    })
                } else {
                    None
                };
                if !transport.handled
                    && why.as_ref().is_some_and(Failure::needs_user)
                    && !self.link.why().is_some_and(Failure::needs_user)
                {
                    retry_at = retry_at.max(now + MAX_RETRY_DELAY);
                }
                if self.live.status.is_connected() && (self.serving(transport) || why.is_none()) {
                    let since = if self.serving(transport) {
                        match self.link {
                            Link::Up { since, .. } => since.or(Some(now)),
                            _ => Some(now),
                        }
                    } else {
                        None
                    };
                    if since
                        .is_some_and(|since| now.duration_since(since) >= STABLE_CONNECTION_PERIOD)
                    {
                        self.attempts = 0;
                    }
                    Link::Up { retry_at, since }
                } else if transport.handled {
                    Link::Dialing { retry_at, why }
                } else {
                    Link::Waiting { retry_at, why }
                }
            }
            Transition::Update {
                state,
                now,
                transport,
            } => {
                std::mem::swap(&mut self.live, state);
                return self.enter(Transition::Observe { now, transport });
            }
            Transition::ResetAttempts => {
                self.attempts = 0;
                return Action::None;
            }
            Transition::Lost { now, lost } => {
                let delay = if self.waits_for_user() {
                    MAX_RETRY_DELAY
                } else {
                    self.retry_delay()
                };
                let reason = Some(
                    self.live
                        .notices
                        .connection_reason()
                        .map(str::to_owned)
                        .unwrap_or(lost),
                );
                let why = Some(match why {
                    Some(why) => why.with_reason(reason),
                    None => Failure::Transport {
                        reason,
                        needs_user: needs_user(&self.live),
                    },
                });
                Link::Waiting {
                    retry_at: now + delay,
                    why,
                }
            }
            Transition::RetryAt(at) => match &self.link {
                Link::Disabled => Link::Disabled,
                Link::Detached { .. } => Link::Detached { retry_at: at, why },
                Link::Waiting { .. } => Link::Waiting { retry_at: at, why },
                Link::Dialing { .. } => Link::Dialing { retry_at: at, why },
                Link::Up { since, .. } => Link::Up {
                    retry_at: at,
                    since: *since,
                },
            },
            Transition::Trade(transport) => {
                // Observe the replacement's handshake, not the parked transport's advice.
                self.link = Link::Dialing {
                    retry_at,
                    why: None,
                };
                return self.enter(Transition::Observe {
                    now: Instant::now(),
                    transport,
                });
            }
            #[cfg(any(test, feature = "test-support"))]
            Transition::Fixture(link) => {
                self.link = link;
                return Action::None;
            }
        };
        self.link = next;
        Action::None
    }

    /// What a retirement leaves of the state the window shows: a retired attempt
    /// still waiting for a retry is not a user detach, so its daemon status reads
    /// connecting again. The window calls this once it has taken the state its
    /// bridge left, which is where that status came from.
    pub fn retired(&mut self) {
        if self.enabled() && !self.detached() {
            self.live.status = ConnectionStatus::Connecting;
        }
    }

    /// Whether the last failure is one only the user can fix, so retrying it
    /// changes nothing.
    pub fn waits_for_user(&self) -> bool {
        self.link.why().is_some_and(Failure::needs_user)
    }

    pub fn herdr_missing(&self) -> bool {
        matches!(self.link.why(), Some(Failure::Missing { .. }))
    }

    pub fn herdr_stopped(&self) -> bool {
        matches!(self.link.why(), Some(Failure::Stopped { .. }))
    }

    pub fn version_mismatch(&self) -> Option<&VersionMismatch> {
        match self.link.why() {
            Some(Failure::Version { mismatch, .. }) => Some(mismatch),
            _ => None,
        }
    }

    /// Why the last attempt ended, in the daemon's words or the app's own.
    pub fn outage(&self) -> Option<&str> {
        self.link.why().and_then(Failure::reason)
    }
}
