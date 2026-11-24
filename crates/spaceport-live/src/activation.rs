// ABOUTME: Owns surface activation submission, acknowledgement, and retry transitions.
// ABOUTME: Retries only barriers known to be unqueued and fences input until coherent acknowledgement.
use crate::navigation::OwnedNavigationTarget;
use crate::state::SurfaceActivation;
use spaceport_client::protocol::{ClientShellSnapshot, PaneSurfaceFrame};
use std::time::{Duration, Instant};

pub const TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Fence {
    #[default]
    Open,
    Releasing {
        deadline: Instant,
    },
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
    Queue,
    Rejected,
    InvalidResponse,
    BootChanged,
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
            Transition::Settled => {
                if let Self::Acknowledged { request, .. } = self {
                    request.focus = None;
                }
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
                    && request.focus.as_ref().is_none_or(|target| match target {
                        crate::navigation::NavigationTarget::Workspace(id) => {
                            snapshot.focused_workspace_id.as_ref() == Some(id)
                        }
                        crate::navigation::NavigationTarget::Tab(id) => {
                            snapshot.focused_tab_id.as_ref() == Some(id)
                        }
                        crate::navigation::NavigationTarget::Pane(id) => {
                            snapshot.focused_pane_id.as_ref() == Some(id)
                        }
                    })
            }
            _ => false,
        }
    }
}
