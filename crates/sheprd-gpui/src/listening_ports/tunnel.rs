// ABOUTME: Opens and owns bounded SSH tunnels for remote port links.
// ABOUTME: Fences results by attempt and cancels exact owned children.
//! SSH tunnels that make a remote server reachable from a browser tab here
//! at `127.0.0.1`. One `ssh -N -L` child per remote port, started when its
//! port is clicked and kept while the port is still listed, so reopening the
//! page reuses it.

use crate::Result;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    net::{IpAddr, Ipv4Addr, TcpListener},
    num::NonZeroU16,
    process::{Child, ExitStatus, Stdio},
    sync::{Arc, Mutex, MutexGuard, PoisonError, mpsc},
    thread,
    time::{Duration, Instant},
};

/// Why an SSH tunnel did not open.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SSH ended before the tunnel opened ({0}).")]
    Exited(ExitStatus),
    #[error("No free local port for an SSH tunnel.")]
    Port(#[source] std::io::Error),
    #[error("Could not start ssh for a tunnel.")]
    Start(#[source] std::io::Error),
    #[error("The SSH tunnel did not open in time.")]
    Timeout,
}

/// SSH connects, authenticates, and binds within its own `ConnectTimeout`.
const OPEN_TIMEOUT: Duration = Duration::from_secs(15);
const POLL_INTERVAL: Duration = Duration::from_millis(100);
/// Under `-N`, SSH's stdout carries only the readiness line, perhaps after
/// whatever the user's shell prints when it runs `LocalCommand`.
const READY_OUTPUT: u64 = 64 * 1024;
/// Tunnels are opened by hand, one click each; this only bounds a runaway.
const TUNNEL_LIMIT: usize = 32;

/// A remote port, by the SSH target its host is dialled with.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Key {
    pub target: String,
    /// The one address the server listens on, or None for one listening on
    /// every address or on loopback, which the remote's `localhost` reaches.
    pub address: Option<IpAddr>,
    pub port: u16,
}

/// The SSH child of a tunnel, shared so quitting or closing the port can kill
/// it at once while the background task alone waits for it, and while its
/// finished [`Tunnel`] is still on its way to the UI thread. Neither side
/// holds the lock across a blocking call.
#[derive(Clone, Default)]
pub(crate) struct Attempt(Arc<Mutex<Slot>>);

#[derive(Default)]
struct Slot {
    cancelled: bool,
    child: Option<Child>,
}

impl Attempt {
    fn slot(&self) -> MutexGuard<'_, Slot> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Takes the freshly spawned child. One that was cancelled meanwhile is
    /// killed at once, which the waiting task then sees as an exit.
    pub(super) fn adopt(&self, child: Child) {
        let mut slot = self.slot();
        let cancelled = slot.cancelled;
        slot.child = Some(child);
        if cancelled {
            kill(&mut slot);
        }
    }

    /// The child's exit, if it has one yet; never blocks.
    pub(super) fn try_wait(&self) -> std::io::Result<Option<ExitStatus>> {
        match self.slot().child.as_mut() {
            Some(child) => child.try_wait(),
            None => Ok(None),
        }
    }

    /// Kills the child without waiting for it; the task that owns the
    /// attempt reaps it. Safe on the UI thread and while quitting.
    pub(crate) fn cancel(&self) {
        let mut slot = self.slot();
        slot.cancelled = true;
        kill(&mut slot);
    }
}

fn kill(slot: &mut Slot) {
    if let Some(child) = &mut slot.child {
        // A child already reaped is not signalled again.
        let _ = child.kill();
    }
}

/// A running `ssh -N -L` child and the local port it forwards. The child
/// stays in its [`Attempt`], so cancelling that attempt still kills it.
pub(crate) struct Tunnel {
    attempt: Attempt,
    local: u16,
}

impl Tunnel {
    pub(super) fn new(attempt: &Attempt, local: u16) -> Self {
        Self {
            attempt: attempt.clone(),
            local,
        }
    }

    /// Whether the child is still forwarding. Never blocks.
    fn alive(&self) -> bool {
        self.attempt
            .slot()
            .child
            .as_mut()
            .is_some_and(|child| matches!(child.try_wait(), Ok(None)))
    }
}

impl Drop for Tunnel {
    /// Kills the child and reaps it on a thread of its own, so closing a
    /// tunnel never waits on the UI thread.
    fn drop(&mut self) {
        let Some(mut child) = self.attempt.slot().child.take() else {
            return;
        };
        let _ = child.kill();
        let reaped = thread::Builder::new()
            .name("herdr-tunnel-reaper".into())
            .spawn(move || {
                let _ = child.wait();
            });
        if let Err(error) = reaped {
            tracing::warn!(category = "listening-ports", %error, "could not reap an SSH tunnel");
        }
    }
}

/// Starts a tunnel to `key` and waits until SSH says its forward is bound,
/// which blocks for up to [`OPEN_TIMEOUT`]: call it off the UI thread.
/// `preferred` is the local port [`Tunnels::begin`] chose, kept when it is
/// still free, so the server keeps its own number or a page reopened after
/// a dropped connection keeps its address.
/// `attempt` lets the caller kill the SSH child while this still waits.
pub(crate) fn open(key: &Key, preferred: Option<u16>, attempt: &Attempt) -> Result<Tunnel> {
    match start(key, preferred, attempt) {
        // Someone took the preferred port meanwhile; any free one will do,
        // unless the attempt was cancelled, which also looks like an exit.
        Err(crate::Error::Tunnel(Error::Exited(_)))
            if preferred.is_some() && !attempt.slot().cancelled =>
        {
            start(key, None, attempt)
        }
        result => result,
    }
}

fn start(key: &Key, preferred: Option<u16>, attempt: &Attempt) -> Result<Tunnel> {
    let local = free_port(preferred).map_err(Error::Port)?;
    let mut command = sheprd_client::forward_command(&key.target, local, key.address, key.port)?;
    // Only the readiness line is read; diagnostics may carry secrets.
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command.spawn().map_err(Error::Start)?;
    let stdout = child.stdout.take();
    attempt.adopt(child);
    let result = match stdout {
        Some(stdout) => ready(|| attempt.try_wait(), stdout, Instant::now() + OPEN_TIMEOUT),
        None => Err(Error::Start(std::io::Error::other("no stdout pipe")).into()), // i18n: data
    };
    // From here on, dropping the tunnel kills the child, on failure too.
    let tunnel = Tunnel::new(attempt, local);
    result.map(|()| tunnel)
}

/// Waits for the child, which `exited` polls, to print
/// [`sheprd_client::FORWARD_READY`]. The port
/// itself is never probed: a process that bound it before SSH could would
/// answer a probe, and a page opened then would show that process instead.
/// SSH prints the line only once it holds the port.
pub(super) fn ready(
    mut exited: impl FnMut() -> std::io::Result<Option<ExitStatus>>,
    stdout: impl Read + Send + 'static,
    deadline: Instant,
) -> Result<()> {
    let (sender, found) = mpsc::sync_channel(1);
    thread::Builder::new()
        .name("herdr-tunnel-ready".into())
        .spawn(move || {
            let ready = BufReader::new(stdout.take(READY_OUTPUT))
                .lines()
                .map_while(std::io::Result::ok)
                .any(|line| line.trim_end() == sheprd_client::FORWARD_READY);
            let _ = sender.send(ready);
        })
        .map_err(Error::Start)?;
    let mut closed = false;
    loop {
        if closed {
            thread::sleep(POLL_INTERVAL);
        } else {
            match found.recv_timeout(POLL_INTERVAL) {
                Ok(true) => return Ok(()),
                // Output ended without the line: the child is exiting.
                Ok(false) | Err(mpsc::RecvTimeoutError::Disconnected) => closed = true,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
        }
        if let Some(status) = exited().map_err(Error::Start)? {
            return Err(Error::Exited(status).into());
        }
        if Instant::now() >= deadline {
            return Err(Error::Timeout.into());
        }
    }
}

/// A loopback port nobody listens on, `preferred` when it is free. The
/// listener is closed before `ssh` binds the port; another process taking it
/// in between makes `ssh` exit before it reports ready, so [`open`] fails
/// rather than opening the other process's page.
pub(super) fn free_port(preferred: Option<u16>) -> std::io::Result<u16> {
    pick(preferred, |port| {
        TcpListener::bind((Ipv4Addr::LOCALHOST, port))?
            .local_addr()
            .map(|address| address.port())
    })
}

/// `preferred` when `bind` takes it, else whatever port `bind(0)` gets.
pub(super) fn pick(
    preferred: Option<u16>,
    mut bind: impl FnMut(u16) -> std::io::Result<u16>,
) -> std::io::Result<u16> {
    match preferred.map(&mut bind) {
        Some(Ok(port)) => Ok(port),
        _ => bind(0),
    }
}

/// One port's lifecycle. A retry keeps the existing tunnel until it succeeds.
#[derive(Default)]
enum State {
    // keep-sorted start
    #[default]
    Idle,
    Open(Tunnel),
    Opening(Attempt),
    Reopening {
        attempt: Attempt,
        tunnel: Tunnel,
    },
    // keep-sorted end
}

impl State {
    fn tunnel(&self) -> Option<&Tunnel> {
        match self {
            Self::Open(tunnel) | Self::Reopening { tunnel, .. } => Some(tunnel),
            Self::Idle | Self::Opening(_) => None,
        }
    }

    fn attempt(&self) -> Option<&Attempt> {
        match self {
            Self::Opening(attempt) | Self::Reopening { attempt, .. } => Some(attempt),
            Self::Idle | Self::Open(_) => None,
        }
    }

    fn count(&self) -> usize {
        usize::from(self.tunnel().is_some()) + usize::from(self.attempt().is_some())
    }
}

#[derive(Default)]
struct Entry {
    state: State,
    /// Port history survives closing, but is not an active tunnel.
    used: Option<u16>,
}

enum Event<'a> {
    // keep-sorted start
    Begin,
    Close,
    Fail(&'a Attempt),
    Finish(&'a Attempt, Tunnel),
    Forget,
    #[cfg(all(test, unix))]
    Insert(Tunnel),
    Prune,
    // keep-sorted end
}

#[derive(Default)]
struct Outcome {
    accepted: bool,
    local: Option<u16>,
}

/// This window's tunnel lifecycles and remembered local ports.
#[derive(Default)]
pub(crate) struct Tunnels {
    entries: HashMap<Key, Entry>,
}

impl Tunnels {
    /// The sole owner of lifecycle and port-history writes.
    fn transition(&mut self, key: &Key, event: Event<'_>) -> Outcome {
        if matches!(event, Event::Begin)
            && (self
                .entries
                .values()
                .map(|entry| entry.state.count())
                .sum::<usize>()
                >= TUNNEL_LIMIT
                || self.attempt(key).is_some())
        {
            return Outcome::default();
        }
        let mut entry = self.entries.remove(key).unwrap_or_default();
        let mut outcome = Outcome::default();
        match event {
            Event::Begin => {
                outcome.accepted = true;
                outcome.local = entry.used.or_else(|| {
                    NonZeroU16::new(key.port)
                        .map(sheprd_client::preferred_local_port)
                        .filter(|port| {
                            !self.entries.values().any(|entry| entry.used == Some(*port))
                        })
                });
                let attempt = Attempt::default();
                entry.state = match entry.state {
                    State::Open(tunnel) => State::Reopening { tunnel, attempt },
                    _ => State::Opening(attempt),
                };
            }
            Event::Finish(attempt, tunnel) => {
                if entry
                    .state
                    .attempt()
                    .is_some_and(|current| Arc::ptr_eq(&current.0, &attempt.0))
                {
                    let local = tunnel.local;
                    // A page left on an old address must not acquire another server.
                    self.entries.retain(|_, other| {
                        if other.used == Some(local) {
                            other.used = None;
                        }
                        other.state.count() != 0 || other.used.is_some()
                    });
                    entry.used = Some(local);
                    entry.state = State::Open(tunnel);
                    outcome = Outcome {
                        accepted: true,
                        local: Some(local),
                    };
                }
            }
            Event::Fail(attempt) => {
                if entry
                    .state
                    .attempt()
                    .is_some_and(|current| Arc::ptr_eq(&current.0, &attempt.0))
                {
                    outcome.accepted = true;
                    entry.state = match entry.state {
                        State::Reopening { tunnel, .. } => State::Open(tunnel),
                        _ => State::Idle,
                    };
                }
            }
            Event::Close | Event::Forget => {
                outcome.accepted = entry.state.count() != 0;
                if let Some(attempt) = entry.state.attempt() {
                    attempt.cancel();
                }
                entry.state = State::Idle;
                if matches!(event, Event::Forget) {
                    entry.used = None;
                }
            }
            Event::Prune => {
                entry.state = match entry.state {
                    State::Open(tunnel) if !tunnel.alive() => State::Idle,
                    State::Reopening { tunnel, attempt } if !tunnel.alive() => {
                        State::Opening(attempt)
                    }
                    state => state,
                };
                outcome.local = entry.state.tunnel().map(|tunnel| tunnel.local);
            }
            #[cfg(all(test, unix))]
            Event::Insert(tunnel) => entry.state = State::Open(tunnel),
        }
        if entry.state.count() != 0 || entry.used.is_some() {
            self.entries.insert(key.clone(), entry);
        }
        outcome
    }

    /// The local port of a live tunnel to `key`; a dead one is dropped.
    pub(crate) fn local(&mut self, key: &Key) -> Option<u16> {
        self.transition(key, Event::Prune).local
    }

    /// The local port of a live tunnel, without changing its lifecycle.
    pub(crate) fn live(&self, key: &Key) -> Option<u16> {
        self.entries
            .get(key)?
            .state
            .tunnel()
            .filter(|tunnel| tunnel.alive())
            .map(|tunnel| tunnel.local)
    }

    /// Starts an attempt and returns its preferred local port. A retry may
    /// keep an existing open tunnel. Active tunnels and attempts share a bound.
    pub(crate) fn begin(&mut self, key: &Key) -> Option<Option<u16>> {
        let outcome = self.transition(key, Event::Begin);
        outcome.accepted.then_some(outcome.local)
    }

    /// The handle on the attempt [`Tunnels::begin`] started for `key`.
    pub(crate) fn attempt(&self, key: &Key) -> Option<Attempt> {
        self.entries.get(key)?.state.attempt().cloned()
    }

    /// Accepts only the current attempt's result. Stale tunnels are dropped.
    pub(crate) fn finish(&mut self, key: Key, attempt: &Attempt, tunnel: Tunnel) -> Option<u16> {
        self.transition(&key, Event::Finish(attempt, tunnel)).local
    }

    /// Whether a tunnel is open. Dead children are pruned on the next scan.
    pub(crate) fn is_open(&self, key: &Key) -> bool {
        self.entries
            .get(key)
            .is_some_and(|entry| entry.state.tunnel().is_some())
    }

    /// Cancels a port's activity while keeping its remembered local port.
    pub(crate) fn close(&mut self, key: &Key) -> bool {
        self.transition(key, Event::Close).accepted
    }

    /// Reports a failure only while that exact attempt is still wanted.
    pub(crate) fn fail(&mut self, key: &Key, attempt: &Attempt) -> bool {
        self.transition(key, Event::Fail(attempt)).accepted
    }

    /// Drops unlisted ports and dead tunnels. Returns whether an open tunnel
    /// went away, so callers redraw the close button only when needed.
    pub(crate) fn retain(&mut self, mut listening: impl FnMut(&Key) -> bool) -> bool {
        let before = self.open_count();
        let keys: Vec<_> = self.entries.keys().cloned().collect();
        for key in keys {
            let event = if listening(&key) {
                Event::Prune
            } else {
                Event::Forget
            };
            self.transition(&key, event);
        }
        self.open_count() != before
    }

    /// Cancels every attempt and drops tunnels without waiting for children.
    pub(crate) fn close_all(&mut self) {
        let keys: Vec<_> = self.entries.keys().cloned().collect();
        for key in keys {
            self.transition(&key, Event::Forget);
        }
    }

    fn open_count(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| entry.state.tunnel().is_some())
            .count()
    }

    #[cfg(all(test, unix))]
    pub(crate) fn insert(&mut self, key: Key, tunnel: Tunnel) {
        self.transition(&key, Event::Insert(tunnel));
    }

    #[cfg(all(test, unix))]
    pub(crate) fn len(&self) -> usize {
        self.open_count()
    }
}

impl Drop for Tunnels {
    /// A window that goes away takes its connecting tunnels with it.
    fn drop(&mut self) {
        self.close_all();
    }
}

#[cfg(all(test, unix))]
impl Tunnel {
    /// A tunnel around any child, standing in for `ssh` in tests.
    pub(crate) fn around(child: Child, local: u16) -> Self {
        let attempt = Attempt::default();
        attempt.adopt(child);
        Self::new(&attempt, local)
    }
}
