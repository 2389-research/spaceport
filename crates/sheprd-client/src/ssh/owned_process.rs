// ABOUTME: Keeps private SSH children within their owning application's lifetime.
// ABOUTME: Uses a pipe watchdog on Unix and a kill-on-close job on Windows.
#[cfg(unix)]
use std::process::Child;
use std::{
    io,
    path::Path,
    process::{ChildStdout, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
pub(crate) mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
type Process = Child;
#[cfg(windows)]
type Process = windows::Child;

const POLL: Duration = Duration::from_millis(25);
const STOP_GRACE: Duration = Duration::from_secs(5);

#[cfg(all(test, unix))]
#[allow(clippy::unwrap_used)]
mod tests;

/// One private SSH child. Stopping never waits; dropping reaps off-thread.
pub struct OwnedChild {
    child: Option<Process>,
    #[cfg(unix)]
    keepalive: Option<std::process::ChildStdin>,
    #[cfg(unix)]
    guarded: bool,
}

impl OwnedChild {
    /// Starts a private child whose listener cannot survive application exit.
    /// `directory` is an owner-only control directory, created by this caller.
    /// Commands must inherit the environment; explicit overrides are preserved.
    /// Unix wrapping cannot preserve `Command::env_clear` or custom stream settings.
    /// Call off the UI thread. Diagnostics are discarded because they may hold secrets.
    pub fn spawn(command: Command, directory: Option<&Path>, stdout: Stdio) -> io::Result<Self> {
        #[cfg(unix)]
        {
            let mut command = unix::command(&command, directory)?;
            let mut child = command.stdout(stdout).stderr(Stdio::null()).spawn()?;
            let keepalive = child.stdin.take();
            Ok(Self {
                child: Some(child),
                keepalive,
                guarded: true,
            })
        }
        #[cfg(windows)]
        {
            let _ = directory;
            let mut command = command;
            command
                .stdin(Stdio::null())
                .stdout(stdout)
                .stderr(Stdio::null());
            Ok(Self {
                child: Some(windows::spawn(command)?),
            })
        }
    }

    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.as_mut().and_then(|child| child.stdout.take())
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        match self.child.as_mut() {
            Some(child) => child.try_wait(),
            None => Ok(None),
        }
    }

    /// Stops only this owned child. Safe on the UI thread and when repeated.
    pub fn stop(&mut self) {
        #[cfg(unix)]
        if self.guarded {
            drop(self.keepalive.take());
            return;
        }
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
        }
    }

    /// Stops and reaps this child. Call only on a worker thread.
    pub fn finish(mut self) {
        self.stop();
        if let Some(child) = self.child.take() {
            reap(child);
        }
    }
}

// Existing lifecycle fixtures also exercise direct children without a watchdog.
#[cfg(all(unix, any(test, feature = "test-support")))]
impl From<Child> for OwnedChild {
    fn from(child: Child) -> Self {
        Self {
            child: Some(child),
            keepalive: None,
            guarded: false,
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        self.stop();
        let Some(child) = self.child.take() else {
            return;
        };
        if let Err(error) = thread::Builder::new()
            .name("spaceport-ssh-reaper".into())
            .spawn(move || reap(child))
        {
            tracing::warn!(category = "ssh", %error, "could not reap an owned SSH child");
        }
    }
}

fn reap(mut child: Process) {
    let deadline = Instant::now() + STOP_GRACE;
    while matches!(child.try_wait(), Ok(None)) && Instant::now() < deadline {
        thread::sleep(POLL);
    }
    if child.kill().is_ok() {
        let _ = child.wait();
    }
}
