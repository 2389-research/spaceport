//! Starting a separately installed core when none is running.

use crate::{Error, Result};
use std::{
    io::{self, Read},
    path::Path,
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
    let status = loop {
        if let Some(status) = child.try_wait().map_err(start_error)? {
            break status;
        }
        if Instant::now() >= deadline {
            // Only the start command dies here; a core it started keeps running.
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::StartTimeout(timeout));
        }
        std::thread::sleep(Duration::from_millis(25));
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
fn drain_stderr(pipe: impl Read + Send + 'static, deadline: Instant) -> String {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut pipe = pipe.take(MAX_STDERR);
        let mut buffer = [0; 1024];
        loop {
            match pipe.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    if tx.send(buffer[..read].to_vec()).is_err() {
                        break;
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
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
