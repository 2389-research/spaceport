//! Starting a separately installed core when none is running.

use crate::{Error, Result};
use std::{
    io::{self, Read},
    path::Path,
    process::ChildStderr,
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

/// The core's executable name; `hc` is only a link to it.
pub const PROGRAM: &str = if cfg!(windows) {
    "herdr-core.exe"
} else {
    "herdr-core"
};

/// Output of a failed start kept for diagnostics.
const MAX_STDERR: u64 = 4 << 10;

/// Runs `binary server start`, which returns once the session's core
/// listens, and waits at most `timeout`. `configure` sets the environment,
/// such as the login shell's. Blocking: never call it on the UI thread.
pub fn start(binary: &Path, configure: impl FnOnce(&mut Command), timeout: Duration) -> Result<()> {
    let mut command = sheprd_client::new_command(binary);
    command
        .args(["server", "start", "--json"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    configure(&mut command);
    let start_error = |source| Error::Start {
        binary: binary.to_owned(),
        source,
    };
    let mut child = command.spawn().map_err(start_error)?;
    let deadline = Instant::now() + timeout;
    let status =
        match sheprd_client::wait::exit(&mut child, deadline, || false).map_err(start_error)? {
            sheprd_client::wait::Exit::Exited(status) => status,
            // Only the start command dies here; a core it started keeps running.
            sheprd_client::wait::Exit::Stopped(_) => return Err(Error::StartTimeout(timeout)),
        };
    if status.success() {
        return Ok(());
    }
    let stderr = child
        .stderr
        .take()
        .map(|pipe| drain_stderr(pipe, deadline))
        .unwrap_or_default();
    Err(Error::StartFailed { status, stderr })
}

/// What a failed start wrote to stderr, read until EOF or `deadline`. A core
/// the start command started keeps the pipe open after that command exits, so
/// only the deadline ends the drain.
///
/// The reader also stops at the deadline: it polls the pipe with what is left
/// of the deadline before each read, so a descendant holding the write end
/// open cannot keep the thread and its descriptor alive after `start` has
/// returned.
fn drain_stderr(pipe: ChildStderr, deadline: Instant) -> String {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut pipe = pipe;
        let mut buffer = [0; 1024];
        let mut served = 0u64;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            #[cfg(unix)]
            if poll_readable(&pipe, remaining).is_err() {
                break;
            }
            #[cfg(not(unix))]
            let _ = remaining;
            match pipe.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    served += read as u64;
                    if served >= MAX_STDERR || tx.send(buffer[..read].to_vec()).is_err() {
                        break;
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error)
                    if error.kind() == io::ErrorKind::WouldBlock
                        || error.kind() == io::ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(_) => break,
            }
        }
    });
    let mut kept = Vec::new();
    while let Ok(chunk) = rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        kept.extend_from_slice(&chunk);
    }
    String::from_utf8_lossy(&kept).into_owned()
}

/// Waits until `pipe` is readable or `remaining` has passed. Errors on a pipe
/// that cannot be polled; the read then still ends at the caller's deadline
/// because the values it keeps are bounded by the channel timeout.
#[cfg(unix)]
fn poll_readable(pipe: impl rustix::fd::AsFd, remaining: Duration) -> io::Result<()> {
    use rustix::event::{PollFd, PollFlags};
    let fd = pipe.as_fd();
    let mut fds = [PollFd::new(&fd, PollFlags::IN)];
    let timeout = rustix::time::Timespec::try_from(remaining).unwrap_or(rustix::time::Timespec {
        tv_sec: i64::MAX,
        tv_nsec: 0,
    });
    let count = rustix::event::poll(&mut fds, Some(&timeout))?;
    if count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "stderr drain deadline",
        ));
    }
    Ok(())
}
