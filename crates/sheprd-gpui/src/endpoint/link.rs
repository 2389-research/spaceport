// ABOUTME: Owns endpoint link transitions and the lifetime of each transport attempt.
// ABOUTME: Keeps retry advice with the link until recovery or an explicit retarget.
use super::*;

pub(super) enum Link {
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
pub(super) enum Failure {
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

    fn retry_at(&self) -> Option<Instant> {
        match self {
            Self::Detached { retry_at, .. }
            | Self::Waiting { retry_at, .. }
            | Self::Dialing { retry_at, .. }
            | Self::Up { retry_at, .. } => Some(*retry_at),
            Self::Disabled => None,
        }
    }
}

pub(super) enum Transition<'a> {
    Stop,
    Detach,
    Retarget {
        target: ConnectTarget,
        enabled: bool,
    },
    Connect {
        options: ConnectOptions,
        active: bool,
    },
    Observe(Instant),
    Update {
        state: &'a mut LiveState,
        now: Instant,
    },
    Lost(Instant),
    RetryAt(Instant),
    ResetAttempts,
    Trade(&'a mut ConnectionBridge, &'a mut LiveState),
    #[cfg(test)]
    Fixture(Link),
}

impl Endpoint {
    /// The only writer of `link`; transport ownership changes enter here too.
    pub(super) fn enter(&mut self, transition: Transition<'_>) {
        let retry_at = self.link.retry_at().unwrap_or_else(Instant::now);
        let why = self.link.why().cloned();
        let next = match transition {
            Transition::Stop | Transition::Detach => {
                let detached = matches!(transition, Transition::Detach) || self.detached();
                self.toasts.entries.clear();
                self.connection.detach(false);
                self.initial_surface = false;
                self.focus_requested = false;
                self.generation += 1;
                self.live = self.connection.take_update().unwrap_or_default();
                // A retired attempt still waiting for a retry is not a user detach.
                if self.enabled() && !detached {
                    self.live.status = ConnectionStatus::Connecting;
                }
                if !self.enabled() {
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
                }
            }
            Transition::Retarget { target, enabled } => {
                self.enter(Transition::Stop);
                self.connection = ConnectionBridge::new(target);
                self.attempts = 0;
                self.recovery = None;
                self.live = LiveState::default();
                Link::initial(enabled)
            }
            Transition::Connect { options, active } => {
                self.enter(Transition::Stop);
                self.initial_surface = active;
                self.attempts = self.attempts.saturating_add(1);
                self.connection.reconnect(
                    options,
                    false,
                    active,
                    crate::daemon::Start::WhenMissing,
                );
                self.live = self.connection.take_update().unwrap_or_default();
                Link::Dialing {
                    retry_at: Instant::now() + self.retry_delay(),
                    why,
                }
            }
            Transition::Observe(now) => {
                if !self.enabled() || self.detached() {
                    return;
                }
                let mut retry_at = retry_at;
                // Only a retired handle starts an outage. A worker diagnostic
                // can refine that outage after its delayed inbox update arrives.
                let reason = self.outage().map(|reason| {
                    if self.connection.handle.is_none() && !self.live.status.is_connected() {
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
                } else if self.live.server_stopped
                    && (crate::daemon::may_start(&self.connection.target)
                        || matches!(
                            self.connection.target,
                            ConnectTarget::Ssh { .. } | ConnectTarget::Wsl { .. }
                        ))
                {
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
                if self.connection.handle.is_none()
                    && why.as_ref().is_some_and(Failure::needs_user)
                    && !self.link.why().is_some_and(Failure::needs_user)
                {
                    retry_at = retry_at.max(now + MAX_RETRY_DELAY);
                }
                if self.live.status.is_connected() && (self.serving() || why.is_none()) {
                    let since = if self.serving() {
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
                } else if self.connection.handle.is_some() {
                    Link::Dialing { retry_at, why }
                } else {
                    Link::Waiting { retry_at, why }
                }
            }
            Transition::Update { state, now } => {
                std::mem::swap(&mut self.live, state);
                self.enter(Transition::Observe(now));
                return;
            }
            Transition::ResetAttempts => {
                self.attempts = 0;
                return;
            }
            Transition::Lost(now) => {
                self.connection.handle = None;
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
                        .unwrap_or_else(|| fl!("endpoint-connection-lost")),
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
            Transition::Trade(connection, live) => {
                std::mem::swap(&mut self.connection, connection);
                std::mem::swap(&mut self.live, live);
                self.initial_surface = true;
                // Observe the replacement's handshake, not the parked transport's advice.
                self.link = Link::Dialing {
                    retry_at,
                    why: None,
                };
                self.enter(Transition::Observe(Instant::now()));
                return;
            }
            #[cfg(test)]
            Transition::Fixture(link) => link,
        };
        self.link = next;
    }

    pub(crate) fn enabled(&self) -> bool {
        !matches!(self.link, Link::Disabled)
    }
    pub(super) fn detached(&self) -> bool {
        matches!(self.link, Link::Detached { .. })
    }
    pub(super) fn retry_at(&self) -> Instant {
        self.link.retry_at().unwrap_or_else(Instant::now)
    }
    pub(super) fn waits_for_user(&self) -> bool {
        self.link.why().is_some_and(Failure::needs_user)
    }
    pub(crate) fn herdr_missing(&self) -> bool {
        matches!(self.link.why(), Some(Failure::Missing { .. }))
    }
    pub(crate) fn herdr_stopped(&self) -> bool {
        matches!(self.link.why(), Some(Failure::Stopped { .. }))
    }
    pub(crate) fn version_mismatch(&self) -> Option<&VersionMismatch> {
        match self.link.why() {
            Some(Failure::Version { mismatch, .. }) => Some(mismatch),
            _ => None,
        }
    }
    pub(crate) fn outage(&self) -> Option<&str> {
        self.link.why().and_then(Failure::reason)
    }
    pub(super) fn retry_delay(&self) -> Duration {
        Duration::from_millis(500u64 << self.attempts.min(8)).min(MAX_RETRY_DELAY)
    }
}

#[cfg(test)]
mod fixtures;
