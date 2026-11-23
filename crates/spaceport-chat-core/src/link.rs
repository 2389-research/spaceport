// ABOUTME: Runs bounded chat socket workers without blocking the UI thread.
// ABOUTME: Reports completed or uncertain writes before exposing connection loss.
//! One connection to a chat program. A reader thread connects, starts a
//! writer thread, and reads; the caller only ever polls and queues, through
//! bounded channels, so no socket I/O or thread join reaches a UI thread.
//! The endpoint is `spaceport_client::private_endpoint`: a socket on Unix, and on
//! Windows a pipe checked to be this user's before anything is sent. A chat
//! on an SSH host is reached through a child process instead, `ssh` running
//! `herdr-acp connect` there, whose standard streams carry the same frames.
//! A reader that gets ahead of the caller by [`MAX_EVENTS`] messages gives the
//! connection up rather than buffering more, and the caller reconnects from
//! its cursor.

use super::{Error, MAX_EVENTS, MAX_PENDING, Result};
use spaceport_chat_protocol::{
    ClientKind, ClientMessage, CommandId, Cursor, REVISION, ServerMessage, VERSION, read_frame,
    write_frame,
};
use spaceport_client::private_endpoint::Connection;
use std::{
    cell::Cell,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
    },
    thread,
    time::Duration,
};

/// How long connecting waits for a free pipe instance on Windows.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

/// Commands queued for the writer: room for every command a session may
/// have waiting, resent on a new connection, and as many more.
const MAX_COMMANDS: usize = 2 * MAX_PENDING;
/// A stalled platform write must not retain the reader or block reconnect.
const DRAIN_TIMEOUT: Duration = Duration::from_millis(100);

/// What one poll of a link found.
#[derive(Debug, Default)]
pub struct Polled {
    pub messages: Vec<ServerMessage>,
    pub writes: Vec<WriteOutcome>,
    /// Why the connection ended, once every message before it was taken.
    pub end: Option<Error>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteOutcome {
    Written(CommandId),
    Uncertain(CommandId),
}

#[derive(Default)]
enum Writing {
    #[default]
    Idle,
    Command(CommandId),
    Stopped,
}

#[derive(Default)]
struct WriterProgress {
    state: Mutex<Writing>,
}

enum Progress {
    Begin(CommandId),
    Complete(WriteOutcome),
    Stop,
}

impl WriterProgress {
    /// Fence receipt publication against timeout without holding I/O locks.
    fn transition(&self, event: Progress, written: &SyncSender<WriteOutcome>) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match (&*state, event) {
            (Writing::Idle, Progress::Begin(id)) => {
                *state = Writing::Command(id);
                true
            }
            (Writing::Command(id), Progress::Complete(outcome))
                if *id
                    == match outcome {
                        WriteOutcome::Written(id) | WriteOutcome::Uncertain(id) => id,
                    } =>
            {
                let sent = written.try_send(outcome).is_ok();
                *state = Writing::Idle;
                sent
            }
            (_, Progress::Stop) => {
                let sent = match *state {
                    Writing::Command(id) => written.try_send(WriteOutcome::Uncertain(id)).is_ok(),
                    Writing::Idle | Writing::Stopped => true,
                };
                *state = Writing::Stopped;
                sent
            }
            _ => false,
        }
    }
}

/// How a link reaches its program.
pub enum Dial {
    /// The program's private endpoint on this machine.
    Socket(PathBuf),
    /// A child whose stdin and stdout carry the connection, such as `ssh`
    /// running `herdr-acp connect` on the program's host.
    Process(Command),
}

impl From<PathBuf> for Dial {
    fn from(path: PathBuf) -> Self {
        Self::Socket(path)
    }
}

/// Ends a connection from any thread without waiting: shuts the socket
/// down, or kills the child, whose reader then reaps it.
enum Closer {
    Socket(spaceport_client::private_endpoint::Closer),
    Process(Arc<Mutex<Child>>),
}

impl Closer {
    fn try_clone(&self) -> std::io::Result<Self> {
        Ok(match self {
            Self::Socket(closer) => Self::Socket(closer.try_clone()?),
            Self::Process(child) => Self::Process(child.clone()),
        })
    }

    fn close(&self) {
        match self {
            Self::Socket(closer) => closer.close(),
            // Held only while the reader reaps a child already killed.
            Self::Process(child) => {
                if let Ok(mut child) = child.try_lock() {
                    let _ = child.kill();
                }
            }
        }
    }

    /// Reaps a killed child; called only from the reader thread.
    fn reap(&self) {
        if let Self::Process(child) = self
            && let Ok(mut child) = child.lock()
        {
            let _ = child.wait();
        }
    }
}

type Parts = (Box<dyn Read + Send>, Box<dyn Write + Send>, Closer);

/// Connects as `dial` says, into a reader, a writer, and their closer.
fn open(dial: Dial) -> std::io::Result<Parts> {
    match dial {
        Dial::Socket(path) => {
            let (reader, writer, closer) = Connection::connect(&path, CONNECT_TIMEOUT)?.split()?;
            Ok((Box::new(reader), Box::new(writer), Closer::Socket(closer)))
        }
        Dial::Process(mut command) => {
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()?;
            let streams = child.stdout.take().zip(child.stdin.take());
            let closer = Closer::Process(Arc::new(Mutex::new(child)));
            let Some((stdout, stdin)) = streams else {
                closer.close();
                closer.reap();
                return Err(std::io::ErrorKind::BrokenPipe.into());
            };
            Ok((Box::new(stdout), Box::new(stdin), closer))
        }
    }
}

pub struct Link {
    commands: SyncSender<ClientMessage>,
    events: Receiver<ServerMessage>,
    writes: Receiver<WriteOutcome>,
    end: Arc<Mutex<Option<Error>>>,
    socket: Arc<Socket>,
    state: Cell<State>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Active,
    Draining,
}

/// Dropping the link closes its transport without joining on the UI thread.
#[derive(Default)]
struct Socket {
    closed: Arc<AtomicBool>,
    closer: Mutex<Option<Closer>>,
}

impl Socket {
    /// Keeps `closer`, closing at once if the link was dropped while it
    /// connected. The flag is read after the store, so a close that missed
    /// the connection is seen here.
    fn attach(&self, closer: &Closer) {
        if let Ok(clone) = closer.try_clone()
            && let Ok(mut slot) = self.closer.lock()
        {
            *slot = Some(clone);
        }
        if self.closed.load(Ordering::SeqCst) {
            closer.close();
        }
    }

    /// Never waits: a connection being attached meanwhile sees the flag.
    fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        if let Ok(slot) = self.closer.try_lock()
            && let Some(closer) = slot.as_ref()
        {
            closer.close();
        }
    }
}

impl Link {
    /// Starts connecting to the program `dial` reaches, resuming after
    /// `since`. Commands are queued only after the session validates its epoch.
    pub fn start(dial: impl Into<Dial>, since: Option<Cursor>) -> Result<Self> {
        let dial = dial.into();
        let (commands, queued) = mpsc::sync_channel(MAX_COMMANDS);
        let (sender, events) = mpsc::sync_channel(MAX_EVENTS);
        let (written, writes) = mpsc::sync_channel(MAX_COMMANDS);
        let end = Arc::new(Mutex::new(None));
        let socket = Arc::new(Socket::default());
        let (thread_end, thread_socket) = (end.clone(), socket.clone());
        thread::Builder::new()
            .name("herdr-gui-chat".into())
            .spawn(move || {
                let error = run(dial, since, queued, &sender, written, &thread_socket);
                if let Ok(mut slot) = thread_end.lock() {
                    slot.get_or_insert(error);
                }
                // The sender drops after the reason is stored, so a window
                // that sees the channel close always finds it.
                drop(sender);
            })
            .map_err(Error::Spawn)?;
        Ok(Self {
            commands,
            events,
            writes,
            end,
            socket,
            state: Cell::new(State::Active),
        })
    }

    /// Queues `message` for the writer without waiting.
    pub fn send(&self, message: ClientMessage) -> Result<()> {
        if self.socket.closed.load(Ordering::SeqCst) {
            return Err(Error::NotAttached);
        }
        self.commands
            .try_send(message)
            .map_err(|error| match error {
                TrySendError::Full(_) => Error::Busy,
                TrySendError::Disconnected(_) => Error::NotAttached,
            })
    }

    pub fn stopped(&self) -> bool {
        self.socket.closed.load(Ordering::SeqCst)
    }

    /// Stops both workers. Their outcomes drain before the final error appears.
    pub fn stop(&self, error: Error) {
        self.state.set(State::Draining);
        if let Ok(mut end) = self.end.lock() {
            end.get_or_insert(error);
        }
        self.socket.close();
    }

    /// Takes at most `budget` messages, and the reason the connection ended
    /// once none are left before it.
    pub fn poll(&self, budget: usize) -> Polled {
        let mut polled = Polled::default();
        while polled.messages.len() < budget {
            match self.events.try_recv() {
                Ok(message) if self.state.get() == State::Active => polled.messages.push(message),
                Ok(_) => {}
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    let end = self.end.try_lock().ok().and_then(|mut end| end.take());
                    polled.end = Some(end.unwrap_or(Error::Closed));
                    break;
                }
            }
        }
        // A final end means the reader joined or fenced its writer. No
        // receipt can arrive after this drain, even if shutdown raced a write.
        while let Ok(outcome) = self.writes.try_recv() {
            polled.writes.push(outcome);
        }
        polled
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        self.socket.close();
    }
}

/// The reader thread: connects, greets through the writer, and reads until
/// the connection ends, returning why.
fn run(
    dial: Dial,
    since: Option<Cursor>,
    queued: Receiver<ClientMessage>,
    events: &SyncSender<ServerMessage>,
    written: SyncSender<WriteOutcome>,
    socket: &Socket,
) -> Error {
    let parts = open(dial).and_then(|(reader, writer, closer)| {
        let writer_closer = closer.try_clone()?;
        Ok((reader, writer, closer, writer_closer))
    });
    let (reader, writer, closer, writer_closer) = match parts {
        Ok(parts) => parts,
        Err(error) => return Error::Connect(error),
    };
    socket.attach(&closer);
    let hello = ClientMessage::Hello {
        client: ClientKind::Gui,
        revision: REVISION,
        since,
        version: VERSION,
    };
    let stopped = socket.closed.clone();
    let progress = Arc::new(WriterProgress::default());
    let writer_progress = progress.clone();
    let writer_written = written.clone();
    let (finished, completion) = mpsc::sync_channel(1);
    let writer = match thread::Builder::new()
        .name("herdr-gui-chat-writer".into())
        .spawn(move || {
            write(
                writer,
                &writer_closer,
                hello,
                &queued,
                &writer_written,
                &stopped,
                &writer_progress,
            );
            let _ = finished.try_send(());
        }) {
        Ok(writer) => writer,
        Err(error) => {
            closer.close();
            return Error::Spawn(error);
        }
    };
    let mut reader = BufReader::new(reader);
    let end = loop {
        match read_frame::<_, ServerMessage>(&mut reader) {
            Ok(Some(message)) => match events.try_send(message) {
                Ok(()) => {}
                Err(TrySendError::Full(_)) => break Error::Overflow,
                Err(TrySendError::Disconnected(_)) => break Error::NotAttached,
            },
            Ok(None) => break Error::Closed,
            Err(error) => break Error::Read(error),
        }
    };
    socket.close();
    closer.close();
    // The dedicated reader waits at most DRAIN_TIMEOUT. A platform write
    // that cannot be interrupted remains detached, fenced against replay.
    drop(drain_writer(writer, &completion, &progress, &written));
    closer.reap();
    end
}

fn drain_writer(
    writer: thread::JoinHandle<()>,
    completion: &Receiver<()>,
    progress: &WriterProgress,
    written: &SyncSender<WriteOutcome>,
) -> Option<thread::JoinHandle<()>> {
    if completion.recv_timeout(DRAIN_TIMEOUT).is_ok() {
        let _ = writer.join();
        None
    } else {
        // Receipt publication and this uncertainty notice share one lock.
        // A late completion cannot race past the final connection end.
        progress.transition(Progress::Stop, written);
        Some(writer)
    }
}

/// The writer thread: the greeting, then each queued command in order,
/// until the link is dropped or a write fails.
fn write(
    writer: impl Write,
    closer: &Closer,
    hello: ClientMessage,
    queued: &Receiver<ClientMessage>,
    written: &SyncSender<WriteOutcome>,
    stopped: &AtomicBool,
    progress: &WriterProgress,
) {
    write_frames(writer, hello, queued, written, stopped, progress);
    closer.close();
}

fn write_frames(
    writer: impl Write,
    hello: ClientMessage,
    queued: &Receiver<ClientMessage>,
    written: &SyncSender<WriteOutcome>,
    stopped: &AtomicBool,
    progress: &WriterProgress,
) {
    let mut writer = BufWriter::new(writer);
    if write_frame(&mut writer, &hello).is_ok() {
        while !stopped.load(Ordering::SeqCst) {
            let message = match queued.recv_timeout(Duration::from_millis(20)) {
                Ok(message) => message,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            };
            if stopped.load(Ordering::SeqCst) {
                break;
            }
            if let ClientMessage::Command { id, .. } = &message
                && !progress.transition(Progress::Begin(*id), written)
            {
                break;
            }
            let result = write_frame(&mut writer, &message);
            if let ClientMessage::Command { id, .. } = message {
                let outcome = if result.is_ok() {
                    WriteOutcome::Written(id)
                } else {
                    WriteOutcome::Uncertain(id)
                };
                // At most MAX_PENDING commands wait for acknowledgement.
                // The larger receipt queue cannot fill before UI polling.
                if !progress.transition(Progress::Complete(outcome), written) {
                    break;
                }
            }
            if result.is_err() {
                break;
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
