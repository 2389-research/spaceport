// ABOUTME: Keeps private SSH children in kill-on-close Windows jobs.
// ABOUTME: Assigns suspended children before process-wrap resumes their first thread.
use process_wrap::std::{ChildWrapper, CommandWrap, CommandWrapper, CreationFlags, JobObject};
use std::{
    io,
    os::windows::io::AsRawHandle,
    process::{ChildStdout, Command, ExitStatus},
    sync::Arc,
};
use win32job::{ExtendedLimitInfo, Job};
use windows::Win32::System::Threading::PROCESS_CREATION_FLAGS;

pub(super) struct Child {
    pub(super) stdout: Option<ChildStdout>,
    process: Box<dyn ChildWrapper>,
    // No child inherits this handle. A parent crash closes its last copy.
    _job: Arc<Job>,
}

#[derive(Debug)]
struct ParentJob(Arc<Job>);

impl CommandWrapper for ParentJob {
    fn post_spawn(
        &mut self,
        _command: &mut Command,
        child: &mut std::process::Child,
        _core: &CommandWrap,
    ) -> io::Result<()> {
        // JobObject's pre_spawn suspends the child. All post_spawn hooks run
        // before its wrap_child assigns the inner job and resumes that child.
        if let Err(error) = self.0.assign_process(child.as_raw_handle() as isize) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::other(error));
        }
        Ok(())
    }
}

pub(super) fn spawn(command: Command) -> io::Result<Child> {
    let mut limits = ExtendedLimitInfo::new();
    limits.limit_kill_on_job_close();
    let job = Arc::new(Job::create_with_limit_info(&limits).map_err(io::Error::other)?);
    let mut process = CommandWrap::from(command)
        .wrap(CreationFlags(PROCESS_CREATION_FLAGS(
            crate::CREATE_NO_WINDOW,
        )))
        .wrap(ParentJob(job.clone()))
        .wrap(JobObject)
        .spawn()?;
    Ok(Child {
        stdout: process.stdout().take(),
        process,
        _job: job,
    })
}

impl Child {
    pub(super) fn kill(&mut self) -> io::Result<()> {
        self.process.start_kill()
    }

    pub(super) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.process.try_wait()
    }

    pub(super) fn wait(&mut self) -> io::Result<ExitStatus> {
        // Do not consume job completion notifications twice after try_wait.
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(status);
            }
            std::thread::sleep(super::POLL);
        }
    }
}
