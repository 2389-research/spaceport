// ABOUTME: Owns one window's opt-in Core connection and exact runtime identity.
// ABOUTME: Joins or starts on a worker without stopping runtime-owned processes.
//! The window's link to herdr-core: found running, or started from a separate
//! installation when the window is allowed to, on a thread of its
//! own. The window reads the outcome on its tick.

use super::Error;
use sheprd_core_client::{Core, CoreBackend, Endpoint, PROGRAM};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Scrollback lines each herdr-core terminal keeps in the app.
const SCROLLBACK: usize = 10_000;
/// How long a started core has to listen.
const START_TIMEOUT: Duration = Duration::from_secs(20);
/// How long a failed join waits before restored tabs try again, so a core
/// that is not running costs one attempt now and then, not one per tick.
const JOIN_RETRY: Duration = Duration::from_secs(2);

/// Where the window's herdr-core connection stands.
#[derive(Default)]
pub(crate) enum State {
    #[default]
    Idle,
    Connecting {
        outcome: Attempt,
        start: Start,
        /// A later explicit request may upgrade only this join's failure.
        upgrade: bool,
    },
    Ready(CoreBackend),
    /// Shared so the window can show it without moving it.
    Unavailable {
        error: Arc<Error>,
        at: Instant,
    },
}

/// Whether the window may start a core when none is running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Start {
    /// Only join a core that already runs.
    Join,
    /// Start an installed `herdr-core`. The window asks
    /// for this only when no Herdr 1 is installed, or the user did.
    IfMissing,
}

/// Why the hub holds no live connection to one exact Core. Each caller
/// matches it, so what a site accepts is written at that site.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum Mismatch {
    #[error("no Core connection is ready")]
    Absent,
    /// The connection is to that Core, and has dropped.
    #[error("the Core connection dropped")]
    Lost,
    /// The connection is to another Core, dropped or not.
    #[error("the connection is to another Core")]
    Other,
}

/// How an attempt joins: the real one dials herdr-core; tests stand in.
type Join = fn(Start) -> Result<CoreBackend, Error>;
type Attempt = std::sync::mpsc::Receiver<Result<CoreBackend, Error>>;
type Spawn = fn(Join, Start) -> std::io::Result<Attempt>;

pub(crate) struct Hub {
    enabled: bool,
    /// Persistent Core identity for an exact existing-runtime view.
    expected_core: Option<String>,
    join: Join,
    spawn: Spawn,
    state: State,
}

/// Inputs accepted by the hub's single lifecycle owner.
enum Event {
    BindExact(String),
    ClearExact,
    Connect(Start),
    Poll,
    #[cfg(test)]
    SetState(State),
}

impl Default for Hub {
    fn default() -> Self {
        Self::with_join(join)
    }
}

impl Hub {
    pub(crate) fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Self::default()
        }
    }

    pub(crate) fn enabled(&self) -> bool {
        self.enabled
    }

    /// Fixes the expected persistent Core ID before a join without dialing.
    /// A start already in flight cannot be made join-only after submission.
    pub(crate) fn bind_exact(&mut self, core_id: &str) -> Result<(), Error> {
        self.check_identity(core_id)?;
        // A start already running cannot be turned into a join-only attempt.
        if matches!(
            self.state,
            State::Connecting {
                start: Start::IfMissing,
                ..
            }
        ) {
            return Err(Error::StartPending);
        }
        if let State::Ready(backend) = &self.state {
            let actual = &backend.core().hello().core.id;
            if actual != core_id {
                return Err(Error::WrongCore {
                    expected: core_id.to_owned(),
                    actual: actual.clone(),
                });
            }
        }
        self.transition(Event::BindExact(core_id.to_owned()));
        Ok(())
    }

    /// Releases the view's exact target when it leaves that Core group.
    pub(crate) fn clear_exact(&mut self) {
        self.transition(Event::ClearExact);
    }

    fn check_identity(&self, core_id: &str) -> Result<(), Error> {
        match self.expected_core.as_deref() {
            Some(expected) if expected != core_id => Err(Error::WrongCore {
                expected: expected.to_owned(),
                actual: core_id.to_owned(),
            }),
            _ => Ok(()),
        }
    }

    fn with_join(join: Join) -> Self {
        Self {
            enabled: false,
            expected_core: None,
            join,
            spawn: spawn_attempt,
            state: State::Idle,
        }
    }

    pub(crate) fn state(&self) -> &State {
        &self.state
    }

    pub(crate) fn backend(&self) -> Option<&CoreBackend> {
        match &self.state {
            State::Ready(backend) => Some(backend),
            _ => None,
        }
    }

    /// The live connection to exactly `core_id`. Identity is compared
    /// first, so `Lost` always means that Core's own connection dropped.
    pub(crate) fn backend_for(&self, core_id: &str) -> Result<&CoreBackend, Mismatch> {
        let backend = self.backend().ok_or(Mismatch::Absent)?;
        if backend.core().hello().core.id != core_id {
            return Err(Mismatch::Other);
        }
        if backend.core().is_lost() {
            return Err(Mismatch::Lost);
        }
        Ok(backend)
    }

    #[cfg(all(test, unix))]
    pub(super) fn set_backend_for_test(&mut self, backend: CoreBackend) {
        self.transition(Event::SetState(State::Ready(backend)));
    }

    #[cfg(test)]
    pub(crate) fn set_unavailable_for_test(&mut self, error: Error) {
        self.transition(Event::SetState(State::Unavailable {
            error: Arc::new(error),
            at: Instant::now(),
        }));
    }

    #[cfg(test)]
    pub(crate) fn set_connecting_for_test(&mut self) {
        self.transition(Event::SetState(State::Connecting {
            outcome: std::sync::mpsc::channel().1,
            start: Start::Join,
            upgrade: false,
        }));
    }

    #[cfg(test)]
    pub(crate) fn set_join_for_test(&mut self, join: Join) {
        self.join = join;
    }

    /// Connects on a background thread unless already connected or trying.
    /// A join-only ask waits a moment after a failure; asking to start
    /// always tries, and upgrades a join-only attempt in flight.
    pub(crate) fn connect(&mut self, start: Start) {
        self.transition(Event::Connect(start));
    }

    /// Takes the connection's outcome; true when the state changed. A lost
    /// connection reads as idle again, so the next ask reconnects.
    pub(crate) fn poll(&mut self) -> bool {
        self.transition(Event::Poll)
    }

    /// Owns connection lifecycle writes, including exact-target start suppression.
    fn transition(&mut self, event: Event) -> bool {
        match event {
            Event::BindExact(core_id) => {
                self.expected_core = Some(core_id);
                if let State::Connecting { upgrade, .. } = &mut self.state {
                    *upgrade = false;
                }
            }
            Event::ClearExact => self.expected_core = None,
            #[cfg(test)]
            Event::SetState(state) => self.state = state,
            Event::Connect(start) => {
                if !self.enabled {
                    return false;
                }
                // An exact Core view may rejoin, but never start a replacement.
                let start = if self.expected_core.is_some() {
                    Start::Join
                } else {
                    start
                };
                match &mut self.state {
                    State::Ready(_) => return false,
                    State::Connecting {
                        start: attempt,
                        upgrade,
                        ..
                    } => {
                        *upgrade |= start == Start::IfMissing && *attempt == Start::Join;
                        return false;
                    }
                    State::Unavailable { at, .. }
                        if start == Start::Join && at.elapsed() < JOIN_RETRY =>
                    {
                        return false;
                    }
                    State::Idle | State::Unavailable { .. } => {}
                }
                self.state = match (self.spawn)(self.join, start) {
                    Ok(outcome) => State::Connecting {
                        outcome,
                        start,
                        upgrade: false,
                    },
                    Err(error) => State::Unavailable {
                        error: Arc::new(Error::Worker(error)),
                        at: Instant::now(),
                    },
                };
            }
            Event::Poll => match &self.state {
                State::Connecting {
                    outcome, upgrade, ..
                } => match outcome.try_recv() {
                    Ok(Ok(backend)) => {
                        let actual = &backend.core().hello().core.id;
                        // Success consumes this attempt's start permission too.
                        self.state = match self.check_identity(actual) {
                            Ok(()) => State::Ready(backend),
                            Err(error) => State::Unavailable {
                                error: Arc::new(error),
                                at: Instant::now(),
                            },
                        };
                    }
                    Ok(Err(_)) if *upgrade => {
                        self.state = State::Idle;
                        self.transition(Event::Connect(Start::IfMissing));
                        return false;
                    }
                    Ok(Err(error)) => {
                        self.state = State::Unavailable {
                            error: Arc::new(error),
                            at: Instant::now(),
                        };
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => return false,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => self.state = State::Idle,
                },
                State::Ready(backend) if backend.core().is_lost() => self.state = State::Idle,
                _ => return false,
            },
        }
        true
    }
}

fn spawn_attempt(join: Join, start: Start) -> std::io::Result<Attempt> {
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("herdr-core-connect".into())
        .spawn(move || {
            let _ = send.send(join(start));
        })?;
    Ok(receive)
}

/// Blocking: joins the session's core, starting one first when allowed.
fn join(start: Start) -> Result<CoreBackend, Error> {
    let endpoint = Endpoint::resolve(sheprd_client::login_env::var)?;
    let first = match Core::connect(&endpoint) {
        Ok(core) => return Ok(CoreBackend::new(core, SCROLLBACK)),
        Err(error) => error,
    };
    if start == Start::Join || !matches!(first, sheprd_core_client::Error::Connect { .. }) {
        return Err(first.into());
    }
    let binary = crate::bundled::locate_installed(PROGRAM).ok_or(Error::NotInstalled)?;
    sheprd_core_client::start(&binary, sheprd_client::login_env::apply, START_TIMEOUT)?;
    // `server start` returns once the core listens; a slow file system can
    // still refuse the first dial.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match Core::connect(&endpoint) {
            Ok(core) => return Ok(CoreBackend::new(core, SCROLLBACK)),
            Err(sheprd_core_client::Error::Connect { .. }) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests;
