// ABOUTME: Reserves a Unix agent's identity until its process group is stopped.
// ABOUTME: Observes exit without reaping and retires signaling before the final reap.
use rustix::process::{Pid, Signal, WaitId, WaitIdOptions, kill_process_group, waitid};
use std::{
    io,
    os::unix::process::{CommandExt, ExitStatusExt},
    process::{self, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus},
    time::{Duration, Instant},
};

pub(super) struct Child {
    pub(super) stdin: Option<ChildStdin>,
    pub(super) stdout: Option<ChildStdout>,
    pub(super) stderr: Option<ChildStderr>,
    // Only this owner may reap the leader. The shared mutex covers every
    // signal and wait; NOWAIT reserves its identity until cleanup.
    group: Option<Pid>,
    inner: process::Child,
    status: Option<ExitStatus>,
}

pub(super) fn spawn(mut command: Command) -> io::Result<Child> {
    command.process_group(0);
    let mut inner = command.spawn()?;
    Ok(Child {
        group: Some(Pid::from_child(&inner)),
        stdin: inner.stdin.take(),
        stdout: inner.stdout.take(),
        stderr: inner.stderr.take(),
        inner,
        status: None,
    })
}

impl Child {
    #[cfg(test)]
    pub(super) fn id(&self) -> u32 {
        self.inner.id()
    }

    pub(super) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        if self.status.is_some() {
            return Ok(self.status);
        }
        let group = self.group.ok_or(rustix::io::Errno::CHILD)?;
        let observed = match waitid(
            WaitId::Pid(group),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
        ) {
            Ok(status) => status,
            Err(error) => {
                // Missing ownership forbids further signals. No other code may reap this child.
                if error == rustix::io::Errno::CHILD {
                    self.group = None;
                }
                return Err(error.into());
            }
        };
        if let Some(status) = observed {
            // waitid reports si_status, not waitpid's packed status word.
            let raw = match (status.exit_status(), status.terminating_signal()) {
                (Some(code), _) => code << 8,
                (_, Some(signal)) => signal | if status.dumped() { 0x80 } else { 0 },
                _ => return Err(io::Error::other("waitid returned a non-exit status")),
            };
            self.status = Some(ExitStatus::from_raw(raw));
        }
        Ok(self.status)
    }

    pub(super) fn stop(&mut self) {
        let _ = self.stop_with(kill_process_group, Self::try_wait, super::GRACE);
    }

    fn stop_with(
        &mut self,
        mut signal: impl FnMut(Pid, Signal) -> rustix::io::Result<()>,
        mut observe: impl FnMut(&mut Self) -> io::Result<Option<ExitStatus>>,
        grace: Duration,
    ) -> io::Result<()> {
        let Some(group) = self.group else {
            return Ok(());
        };
        let _ = signal(group, Signal::TERM);
        let deadline = Instant::now() + grace;
        while Instant::now() < deadline {
            if let Err(error) = observe(self) {
                if self.group.is_none() {
                    return Err(error);
                }
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                // Observation errors must not skip final cleanup of an owned child.
            }
            // The reserved leader can keep the group probe positive even after exit.
            std::thread::sleep(super::POLL.min(deadline.saturating_duration_since(Instant::now())));
        }
        let _ = signal(group, Signal::KILL);
        // No group signal may follow the reap, even if waiting returns an error.
        self.group = None;
        let _ = self.inner.kill();
        let status = self.inner.wait()?;
        self.status = Some(status);
        Ok(())
    }
}

impl Drop for Child {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests;
