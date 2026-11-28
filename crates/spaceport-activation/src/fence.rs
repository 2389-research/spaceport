// ABOUTME: Owns surface activation submission, acknowledgement, and retry transitions.
// ABOUTME: Retries only barriers known to be unqueued and fences input until coherent acknowledgement.
//!
//! One activation is: activate a terminal surface with an ordered barrier, and
//! keep input off until the daemon acknowledges that barrier and the projection
//! it names is the one the snapshot agrees with. [`Fence`] is that lifecycle and
//! [`Transition`] is its only writer, so a receipt that arrives late cannot
//! revive a barrier that already failed or was superseded.
//!
//! Both clients drive it. The distinction that matters is between a barrier the
//! ordered queue refused — it never entered the queue, so it is safe to send
//! again — and one that was submitted and then went unanswered, which is not.
//! Losing that distinction is what makes one client disconnect a healthy
//! connection where another simply retries.
use crate::OwnedNavigationTarget;
use spaceport_client::protocol::{ClientShellSnapshot, PaneSurfaceFrame};
use std::time::{Duration, Instant};

/// The budget one barrier gets, from submission to acknowledgement.
///
/// One barrier, not one navigation: a client that reaches a destination in
/// several requests gives each of them this, and bounds the whole walk itself
/// ([`crate::ROUTE_BUDGET`]).
pub const TIMEOUT: Duration = Duration::from_secs(5);

/// The activation one barrier was asked for: its request, the daemon boot it
/// was asked of, the destination it was aimed at, and which way it goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceActivation {
    pub request: String,
    pub boot: String,
    pub focus: Option<OwnedNavigationTarget>,
    pub active: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Fence {
    #[default]
    Open,
    Releasing {
        deadline: Instant,
    },
    /// A barrier the queue refused. It never entered the queue, so it may be
    /// sent again; the destination it was aimed at is still waiting.
    Retry {
        boot: String,
        focus: Option<OwnedNavigationTarget>,
    },
    Sent {
        request: SurfaceActivation,
        deadline: Instant,
    },
    Acknowledged {
        request: SurfaceActivation,
        revision: u64,
        deadline: Instant,
    },
    Ready {
        request: SurfaceActivation,
        revision: u64,
    },
    Failed {
        request: Option<SurfaceActivation>,
        cause: Failure,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    /// The barrier never entered the ordered queue, and not because it was
    /// full: the connection refused it outright.
    Queue,
    /// The daemon answered the barrier with an error.
    Rejected,
    /// The daemon answered, but not with the acknowledgement the barrier asked
    /// for.
    InvalidResponse,
    /// The daemon restarted under the activation.
    BootChanged,
    /// The barrier was submitted and went unacknowledged for [`TIMEOUT`].
    Timeout,
}

pub enum Transition<'a> {
    #[cfg(any(test, feature = "test-support"))]
    Reset(Option<Instant>),
    Select(Instant),
    Wait(Instant),
    Renew(Instant),
    Queued {
        result: Result<String, spaceport_client::SendError>,
        boot: String,
        focus: Option<OwnedNavigationTarget>,
        active: bool,
        now: Instant,
    },
    Rejected(&'a str),
    Response {
        request: &'a str,
        result: Option<&'a serde_json::Value>,
    },
    Boot(&'a str),
    /// A create request named the tab that its ordered barrier must observe.
    CreatedTab(String),
    Settled,
    Opened,
    Expire(Instant),
}

impl Fence {
    #[cfg(any(test, feature = "test-support"))]
    pub fn for_test(request: SurfaceActivation, revision: Option<u64>) -> Self {
        let mut fence = Self::Open;
        let id = request.request.clone();
        let active = request.active;
        let settled = request.focus.is_none();
        fence.transition(Transition::Queued {
            result: Ok(request.request),
            boot: request.boot,
            focus: request.focus,
            active,
            now: Instant::now(),
        });
        if let Some(revision) = revision {
            fence.transition(Transition::Response { request: &id, result: Some(&serde_json::json!({
                "type": "client_shell_surface_set", "active": active, "projection_revision": revision,
            })) });
            if settled {
                fence.transition(Transition::Opened);
            }
        }
        fence
    }

    /// The only writer for the activation lifecycle, on both inbox and UI clones.
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 141 lines")]
    pub fn transition(&mut self, event: Transition<'_>) {
        match event {
            #[cfg(any(test, feature = "test-support"))]
            Transition::Reset(deadline) => {
                *self = deadline.map_or(Self::Open, |deadline| Self::Releasing { deadline })
            }
            Transition::Select(budget) => match self {
                Self::Open | Self::Releasing { .. } => {
                    *self = Self::Releasing { deadline: budget };
                }
                Self::Sent { request, deadline }
                | Self::Acknowledged {
                    request, deadline, ..
                } if !request.active => {
                    // A new selection owns its budget, while the source release
                    // still owns the inactive request and its acknowledgement.
                    *deadline = budget;
                }
                Self::Ready { request, revision } if !request.active => {
                    *self = Self::Acknowledged {
                        request: request.clone(),
                        revision: *revision,
                        deadline: budget,
                    };
                }
                _ => {}
            },
            Transition::Wait(now) => {
                if matches!(self, Self::Open) {
                    *self = Self::Releasing {
                        deadline: now + TIMEOUT,
                    };
                }
            }
            Transition::Renew(now) => match self {
                Self::Releasing { deadline }
                | Self::Sent { deadline, .. }
                | Self::Acknowledged { deadline, .. } => *deadline = now + TIMEOUT,
                _ => {}
            },
            Transition::Queued {
                result,
                boot,
                focus,
                active,
                now,
            } => {
                *self = match result {
                    Ok(request) => Self::Sent {
                        request: SurfaceActivation {
                            request,
                            boot,
                            focus,
                            active,
                        },
                        deadline: now + TIMEOUT,
                    },
                    // Full proves this barrier never entered the ordered queue.
                    Err(spaceport_client::Error::Full) if active => Self::Retry { boot, focus },
                    Err(_) => Self::Failed {
                        request: None,
                        cause: Failure::Queue,
                    },
                };
            }
            Transition::Rejected(id) => {
                if let Some(request) = self
                    .request()
                    .filter(|request| request.request == id)
                    .cloned()
                {
                    *self = Self::Failed {
                        request: Some(request),
                        cause: Failure::Rejected,
                    };
                }
            }
            Transition::Response {
                request: id,
                result,
            } => {
                // Late receipts cannot revive a failed or superseded barrier.
                let Self::Sent { request, deadline } = self else {
                    return;
                };
                if request.request != id {
                    return;
                }
                let revision = result
                    .filter(|result| {
                        result["type"] == "client_shell_surface_set"
                            && result["active"] == request.active
                    })
                    .and_then(|result| result["projection_revision"].as_u64());
                let request = request.clone();
                let deadline = *deadline;
                *self = match revision {
                    Some(revision) => Self::Acknowledged {
                        request,
                        revision,
                        deadline,
                    },
                    None => Self::Failed {
                        request: Some(request),
                        cause: Failure::InvalidResponse,
                    },
                };
            }
            Transition::Boot(boot) => {
                let stale = match self {
                    Self::Retry { boot: expected, .. } => expected != boot,
                    _ => self.request().is_some_and(|request| request.boot != boot),
                };
                if stale {
                    *self = Self::Failed {
                        request: self.request().cloned(),
                        cause: Failure::BootChanged,
                    };
                }
            }
            Transition::Expire(now) => {
                if self.deadline().is_some_and(|deadline| now >= deadline) {
                    *self = Self::Failed {
                        request: self.request().cloned(),
                        cause: Failure::Timeout,
                    };
                }
            }
            Transition::CreatedTab(tab) => self.set_focus(Some(OwnedNavigationTarget::Tab(tab))),
            Transition::Settled => {
                // Focus is evidence for completing one navigation, not a
                // permanent constraint. A caller whose focus request was
                // refused settles while its barrier is still waiting, because
                // the barrier alone establishes which pane owns input.
                self.set_focus(None);
            }
            Transition::Opened => {
                if let Self::Acknowledged {
                    request, revision, ..
                } = self
                {
                    let mut request = request.clone();
                    request.focus = None;
                    *self = Self::Ready {
                        request,
                        revision: *revision,
                    };
                }
            }
        }
    }

    fn set_focus(&mut self, target: Option<OwnedNavigationTarget>) {
        match self {
            Self::Sent { request, .. } | Self::Acknowledged { request, .. } => {
                request.focus = target
            }
            Self::Retry { focus, .. } => *focus = target,
            _ => {}
        }
    }

    pub fn request(&self) -> Option<&SurfaceActivation> {
        match self {
            Self::Sent { request, .. }
            | Self::Acknowledged { request, .. }
            | Self::Ready { request, .. } => Some(request),
            Self::Failed { request, .. } => request.as_ref(),
            _ => None,
        }
    }

    pub fn revision(&self) -> Option<u64> {
        match self {
            Self::Acknowledged { revision, .. } | Self::Ready { revision, .. } => Some(*revision),
            _ => None,
        }
    }

    pub fn deadline(&self) -> Option<Instant> {
        match self {
            Self::Releasing { deadline }
            | Self::Sent { deadline, .. }
            | Self::Acknowledged { deadline, .. } => Some(*deadline),
            _ => None,
        }
    }

    pub fn failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    pub fn fenced(&self) -> bool {
        !matches!(self, Self::Open | Self::Ready { .. })
    }

    pub fn pending(&self) -> bool {
        match self {
            Self::Open => false,
            Self::Ready { request, .. } | Self::Acknowledged { request, .. } => {
                !request.active || request.focus.is_some()
            }
            _ => true,
        }
    }

    /// Whether the projection a client is showing is the one this activation
    /// asked for: the acknowledged activation is active, belongs to this daemon
    /// boot, presents the revision it was acknowledged at or a later one, and
    /// holds the destination it was aimed at.
    ///
    /// The caller owns coherence: a snapshot and a surface that name different
    /// revisions belong to each other only once they agree, and until then
    /// there is nothing here to be ready against.
    pub fn ready(&self, snapshot: &ClientShellSnapshot, surface: &PaneSurfaceFrame) -> bool {
        match self {
            Self::Open => true,
            Self::Acknowledged {
                request, revision, ..
            }
            | Self::Ready { request, revision } => {
                request.active
                    && request.boot == snapshot.boot_id
                    && surface.projection_revision >= *revision
                    && request
                        .focus
                        .as_ref()
                        .is_none_or(|target| target.is_focused_in(snapshot))
            }
            _ => false,
        }
    }
}
