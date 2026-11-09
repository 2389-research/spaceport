// ABOUTME: Stops background scans without blocking their owner.
// ABOUTME: Wakes sleeping workers so dropping a scanner takes effect at once.
use std::{
    sync::{Condvar, Mutex, PoisonError},
    time::Instant,
};

/// Tells a worker to stop, waking it from its wait between scans.
#[derive(Default)]
pub(crate) struct Stop {
    stopped: Mutex<bool>,
    changed: Condvar,
}

impl Stop {
    pub(crate) fn stop(&self) {
        *self.stopped.lock().unwrap_or_else(PoisonError::into_inner) = true;
        self.changed.notify_all();
    }

    /// Waits until `due`; false once stopped.
    pub(crate) fn wait(&self, due: Instant) -> bool {
        let mut stopped = self.stopped.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            if *stopped {
                return false;
            }
            let now = Instant::now();
            if now >= due {
                return true;
            }
            stopped = self
                .changed
                .wait_timeout(stopped, due - now)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
    }
}

#[cfg(test)]
mod tests;
