// ABOUTME: Starts ACP agents with owned process trees and drained standard error.
// ABOUTME: Retains exit status while stopping owned Unix groups and Windows jobs.
//! The agent's process and descendants, including an npm shim's Node child.

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;
#[cfg(unix)]
use unix::Child;
#[cfg(windows)]
use windows::Child;

use std::{
    collections::VecDeque,
    ffi::OsString,
    io::{BufRead, BufReader},
    path::Path,
    process::{ChildStdin, ChildStdout, ExitStatus, Stdio},
    sync::{Arc, Mutex, PoisonError},
    thread,
    time::Duration,
};

use crate::{Error, Result};

/// Lines of the agent's standard error kept to explain a failure.
const STDERR_LINES: usize = 20;
const STDERR_LINE: usize = 400;
const POLL: Duration = Duration::from_millis(50);
#[cfg(unix)]
const GRACE: Duration = Duration::from_secs(2);

pub(crate) struct Spawned {
    pub input: ChildStdin,
    pub output: ChildStdout,
    pub process: AgentProcess,
}

/// A handle that can stop the agent from any thread.
#[derive(Clone)]
pub(crate) struct AgentProcess {
    child: Arc<Mutex<Child>>,
    stderr: Arc<Mutex<VecDeque<String>>>,
}

pub(crate) struct StopOnDrop(AgentProcess);

impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.stop();
    }
}

pub(crate) fn spawn(
    program: &OsString,
    args: &[OsString],
    env: &[(&str, &str)],
    cwd: &Path,
) -> Result<Spawned> {
    let mut command = sheprd_client::new_command(program);
    command
        .args(args)
        .envs(env.iter().copied())
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    let child = unix::spawn(command);
    #[cfg(windows)]
    let child = windows::spawn(command);
    let mut child = child.map_err(|source| Error::AgentSpawn {
        program: program.to_string_lossy().into_owned(),
        source,
    })?;
    let input = child.stdin.take().ok_or(Error::AgentPipe("input"))?;
    let output = child.stdout.take().ok_or(Error::AgentPipe("output"))?;
    let stderr = Arc::new(Mutex::new(VecDeque::new()));
    if let Some(errors) = child.stderr.take() {
        let tail = Arc::clone(&stderr);
        // Keeps the last lines only; the rest of standard error is discarded so
        // it never draws over the chat view.
        thread::Builder::new()
            .name("acp-stderr".to_owned())
            .spawn(move || {
                for line in BufReader::new(errors).lines() {
                    let Ok(line) = line else { return };
                    let mut tail = tail.lock().unwrap_or_else(PoisonError::into_inner);
                    if tail.len() == STDERR_LINES {
                        tail.pop_front();
                    }
                    tail.push_back(line.chars().take(STDERR_LINE).collect());
                }
            })?;
    }
    Ok(Spawned {
        input,
        output,
        process: AgentProcess {
            child: Arc::new(Mutex::new(child)),
            stderr,
        },
    })
}

impl AgentProcess {
    pub(crate) fn stop_on_drop(&self) -> StopOnDrop {
        StopOnDrop(self.clone())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Child> {
        self.child.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Waits for the agent to exit, polling so that `stop` can still reach it.
    pub(crate) fn wait(&self) -> std::io::Result<ExitStatus> {
        loop {
            if let Some(status) = self.lock().try_wait()? {
                return Ok(status);
            }
            thread::sleep(POLL);
        }
    }

    /// The last lines the agent wrote to standard error.
    pub(crate) fn stderr_tail(&self) -> Vec<String> {
        self.stderr
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .cloned()
            .collect()
    }

    /// Ends the owned process tree, including descendants of an exited shim.
    /// Unix groups receive TERM before KILL; Windows jobs terminate together.
    pub(crate) fn stop(&self) {
        #[cfg(unix)]
        self.lock().stop();
        #[cfg(windows)]
        {
            let mut child = self.lock();
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(all(test, unix))]
mod tests;

#[cfg(all(test, windows))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod windows_tests;
