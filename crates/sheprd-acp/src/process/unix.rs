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
        let _ = self.stop_with(
            kill_process_group,
            Self::try_wait,
            others_remain,
            super::GRACE,
        );
    }

    fn stop_with(
        &mut self,
        mut signal: impl FnMut(Pid, Signal) -> rustix::io::Result<()>,
        mut observe: impl FnMut(&mut Self) -> io::Result<Option<ExitStatus>>,
        mut others_remain: impl FnMut(Pid) -> bool,
        grace: Duration,
    ) -> io::Result<()> {
        let Some(group) = self.group else {
            return Ok(());
        };
        let _ = signal(group, Signal::TERM);
        // An exited leader does not prove its descendants stopped handling TERM.
        // Give descendants the grace while the leader reserves the group identity,
        // and end it early once nothing but that reserved leader is left.
        let deadline = Instant::now() + grace;
        while Instant::now() < deadline {
            match observe(self) {
                Ok(Some(_)) if !others_remain(group) => break,
                Ok(_) => {}
                Err(error) => {
                    if self.group.is_none() {
                        return Err(error);
                    }
                    if error.kind() == io::ErrorKind::Interrupted {
                        continue;
                    }
                    // Observation errors must not skip final cleanup of an owned child.
                }
            }
            // The reserved leader keeps a signal probe of the group positive
            // after exit, so the group's members are listed instead.
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

/// Whether a process other than the reserved `leader` is still in its group,
/// as `pgrep -g` lists it: macOS's leaves zombies out, Linux's lists the
/// leader itself. True when the list cannot be read, so the caller keeps its
/// full grace.
fn others_remain(leader: Pid) -> bool {
    let leader = leader.as_raw_nonzero().to_string();
    let output = sheprd_client::new_command("pgrep")
        .args(["-g", &leader])
        .stdin(process::Stdio::null())
        .stderr(process::Stdio::null())
        .output();
    match output {
        // pgrep exits 1 when it matched nothing.
        Ok(output) if output.status.code() == Some(1) => false,
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .any(|pid| pid != leader),
        _ => true,
    }
}

impl Drop for Child {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests;
