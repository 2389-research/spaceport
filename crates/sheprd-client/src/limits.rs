//! Bounds the connection worker enforces: queue depths, poll cadence, and the
//! timeouts that turn a stalled peer into a typed error rather than a hang,
//! plus the one bounded wait for a child process.

use std::time::Duration;

pub(crate) const COMMAND_CAPACITY: usize = 64;
pub(crate) const EVENT_CAPACITY: usize = 8;
/// How often a bounded wait re-reads the condition it waits on. An idle worker,
/// a child process, and a socket nobody notifies give no event to wait for.
pub(crate) const POLL: Duration = Duration::from_millis(10);
/// Longest an idle worker blocks. Deadlines and wakes end it sooner; this only
/// bounds the cost of a wake that was lost.
pub(crate) const IDLE_WAIT: Duration = Duration::from_secs(5);
/// How long an SSH link may stay silent before a health ping.
pub(crate) const QUIET_PING: Duration = Duration::from_secs(5);
pub(crate) const TIMEOUT: Duration = Duration::from_secs(10);
/// How long a quiet SSH link may leave a health ping unanswered.
pub(crate) const PING_TIMEOUT: Duration = Duration::from_secs(10);
/// How long a link asked to prove itself, as after the machine woke, may take.
pub(crate) const LIVENESS_TIMEOUT: Duration = Duration::from_secs(3);
pub(crate) const COMMAND_TIMEOUT: Duration = Duration::from_secs(60);
// Completed API round trips over 250 ms are noteworthy; queue wait is excluded.
pub(crate) const SLOW_REQUEST: Duration = Duration::from_millis(250);
pub(crate) const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

/// The bounded wait for a child process this client or the app started: the
/// deadline, the kill on a missed deadline or a stop, and the reap, in one
/// place so no caller can leave a child running or unreaped.
pub mod wait {
    use super::POLL;
    use std::{
        io,
        process::{Child, ExitStatus},
        thread,
        time::Instant,
    };

    /// Why [`exit`] stopped waiting before the child ended.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Stop {
        /// The caller's deadline passed.
        TimedOut,
        /// The caller asked the wait to stop.
        Cancelled,
    }

    /// How a waited child ended.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Exit {
        /// The child ended on its own, and is reaped.
        Exited(ExitStatus),
        /// The wait stopped first; the child was killed and reaped.
        Stopped(Stop),
    }

    /// Waits for `child` to end, at most until `deadline`, killing and reaping
    /// it when the deadline passes or `cancelled` returns true. A child's exit
    /// has no notification to wait on, so this re-reads it every [`POLL`]; the
    /// caller keeps the child, its pipes, and its own bound. Blocking: call it
    /// from a worker, never the UI thread.
    pub fn exit(
        child: &mut Child,
        deadline: Instant,
        cancelled: impl Fn() -> bool,
    ) -> io::Result<Exit> {
        loop {
            if let Some(status) = child.try_wait()? {
                return Ok(Exit::Exited(status));
            }
            let stop = if cancelled() {
                Some(Stop::Cancelled)
            } else if Instant::now() >= deadline {
                Some(Stop::TimedOut)
            } else {
                None
            };
            if let Some(stop) = stop {
                // Only this exact child is signalled, and it is reaped here so
                // a stopped wait never leaves a zombie.
                let _ = child.kill();
                let _ = child.wait();
                return Ok(Exit::Stopped(stop));
            }
            thread::sleep(POLL);
        }
    }
}

#[cfg(all(test, unix))]
mod tests;
