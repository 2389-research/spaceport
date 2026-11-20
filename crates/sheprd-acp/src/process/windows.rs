// ABOUTME: Owns the Windows job containing an ACP agent and all its descendants.
// ABOUTME: Preserves piped streams and direct-agent exit status while stopping the job.
use process_wrap::std::{ChildWrapper, CommandWrap, CreationFlags, JobObject};
use std::{
    io,
    process::{ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus},
};
use windows::Win32::System::Threading::PROCESS_CREATION_FLAGS;

pub(super) struct Child {
    pub(super) stdin: Option<ChildStdin>,
    pub(super) stdout: Option<ChildStdout>,
    pub(super) stderr: Option<ChildStderr>,
    job: Box<dyn ChildWrapper>,
}

pub(super) fn spawn(command: Command) -> io::Result<Child> {
    // JobObject creates the child suspended, assigns its job, then resumes it.
    // Failed assignment or resume kills and reaps that child before returning.
    let mut job = CommandWrap::from(command)
        .wrap(CreationFlags(PROCESS_CREATION_FLAGS(
            sheprd_client::CREATE_NO_WINDOW,
        )))
        .wrap(JobObject)
        .spawn()?;
    Ok(Child {
        stdin: job.stdin().take(),
        stdout: job.stdout().take(),
        stderr: job.stderr().take(),
        job,
    })
}

impl Child {
    pub(super) fn kill(&mut self) -> io::Result<()> {
        self.job.start_kill()
    }

    pub(super) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.job.try_wait()
    }

    pub(super) fn wait(&mut self) -> io::Result<ExitStatus> {
        // Preserve the agent's status, even when a descendant outlives its shim.
        // Polling avoids the wrapper's completion-port wait after try_wait may
        // have consumed a job notification. Termination still targets the job.
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(status);
            }
            std::thread::sleep(super::POLL);
        }
    }
}

impl Drop for Child {
    fn drop(&mut self) {
        // Keep setup failures and the last owner dropping from leaking agents.
        if self.kill().is_ok() {
            let _ = self.wait();
        }
    }
}
