// ABOUTME: Owns bounded FIFO scheduling for user-requested Core terminals.
// ABOUTME: Consumes each submitted request once, including uncertain worker outcomes.

use super::{Purpose, infos::Infos, purpose_matches_core};
use spaceport_backend::{Spawn, TerminalInfo};
use spaceport_core_client::CoreBackend;
use std::{collections::VecDeque, sync::mpsc};

/// Waiting requests, separate from the one already submitted to Core.
const CAPACITY: usize = 64;
type Completion = Result<TerminalInfo, super::Error>;
type Started = mpsc::Receiver<Completion>;
type SpawnWorker = fn(CoreBackend, Spawn, mpsc::SyncSender<Completion>) -> std::io::Result<()>;
pub(super) struct Request {
    expected_core: Option<String>,
    purpose: Purpose,
    spawn: Spawn,
    report: Option<super::Reporter>,
}

#[derive(Default)]
pub(super) enum Spawning {
    #[default]
    Idle,
    Queued(VecDeque<Request>),
    Running {
        /// The Core that ran this request, so placement can tell whether the
        /// window still shows that Core's terminals.
        core: String,
        purpose: Purpose,
        started: Started,
        queued: VecDeque<Request>,
        report: Option<super::Reporter>,
    },
}

pub(super) enum Event<'a> {
    Queue(Purpose, Box<Spawn>, Option<String>, Option<super::Reporter>),
    Start(&'a CoreBackend, &'a Infos, SpawnWorker),
    Poll,
    Unavailable,
}

pub(super) enum Outcome {
    None,
    Accepted,
    Full,
    Stale,
    Cleared,
    Finished(String, Purpose, Box<Completion>, Option<super::Reporter>),
}

impl Spawning {
    /// The only writer of the spawn lifecycle, including queued requests.
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 113 lines")]
    pub(super) fn transition(&mut self, event: Event<'_>) -> Outcome {
        match event {
            Event::Queue(purpose, spawn, core_id, report) => {
                let request = Request {
                    expected_core: core_id,
                    purpose,
                    spawn: *spawn,
                    report,
                };
                match self {
                    Self::Idle => *self = Self::Queued(VecDeque::from([request])),
                    Self::Queued(queued) | Self::Running { queued, .. } => {
                        if queued.len() == CAPACITY {
                            return Outcome::Full;
                        }
                        queued.push_back(request);
                    }
                }
                Outcome::Accepted
            }
            Event::Start(backend, infos, launch) => {
                let Self::Queued(queued) = self else {
                    return Outcome::None;
                };
                let Some(Request {
                    purpose,
                    mut spawn,
                    expected_core,
                    report,
                }) = queued.pop_front()
                else {
                    *self = Self::Idle;
                    return Outcome::None;
                };
                let queued = std::mem::take(queued);
                *self = if queued.is_empty() {
                    Self::Idle
                } else {
                    Self::Queued(queued)
                };
                let actual = &backend.core().hello().core.id;
                if expected_core
                    .as_ref()
                    .is_some_and(|expected| expected != actual)
                    || !purpose_matches_core(&purpose, actual)
                {
                    return Outcome::Stale;
                }
                if spawn.cwd.is_none()
                    && let Purpose::Split { at, .. } = &purpose
                {
                    spawn.cwd = infos.get(at).and_then(|info| info.cwd.clone());
                }
                let backend = backend.clone();
                let (send, started) = mpsc::sync_channel(1);
                if let Err(error) = launch(backend, spawn, send.clone()) {
                    tracing::warn!(%error, "Could not start a herdr-core spawn worker");
                    // No worker exists; this new one-slot mailbox is empty.
                    let _ = send.send(Err(super::Error::Worker(error)));
                }
                let queued = match std::mem::take(self) {
                    Self::Queued(queued) => queued,
                    _ => VecDeque::new(),
                };
                *self = Self::Running {
                    core: actual.clone(),
                    purpose,
                    started,
                    queued,
                    report,
                };
                Outcome::None
            }
            Event::Poll => {
                let Self::Running { started, .. } = self else {
                    return Outcome::None;
                };
                let outcome = match started.try_recv() {
                    Ok(outcome) => outcome,
                    Err(mpsc::TryRecvError::Empty) => return Outcome::None,
                    // Submission may have happened. Never put this request back in the queue.
                    Err(mpsc::TryRecvError::Disconnected) => {
                        Err(super::Error::Core(spaceport_core_client::Error::Lost))
                    }
                };
                let Self::Running {
                    core,
                    purpose,
                    queued,
                    report,
                    ..
                } = std::mem::take(self)
                else {
                    return Outcome::None;
                };
                if !queued.is_empty() {
                    *self = Self::Queued(queued);
                }
                Outcome::Finished(core, purpose, Box::new(outcome), report)
            }
            Event::Unavailable => {
                let cleared = match self {
                    Self::Idle => false,
                    Self::Queued(_) => {
                        *self = Self::Idle;
                        true
                    }
                    Self::Running { queued, .. } => !std::mem::take(queued).is_empty(),
                };
                if cleared {
                    Outcome::Cleared
                } else {
                    Outcome::None
                }
            }
        }
    }
}

/// Launches blocking Core work away from the UI thread.
pub(super) fn spawn_worker(
    backend: CoreBackend,
    spawn: Spawn,
    send: mpsc::SyncSender<Completion>,
) -> std::io::Result<()> {
    std::thread::Builder::new()
        .name("herdr-core-spawn".into())
        .spawn(move || {
            let _ = send.send(backend.spawn(&spawn).map_err(super::Error::from));
        })
        .map(drop)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
