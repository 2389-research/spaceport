// ABOUTME: Stores one shared accessibility snapshot for motion and theme consumers.
// ABOUTME: Keeps native probes and window refreshes outside the settings crate.
use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
#[cfg(test)]
mod tests;

const REDUCE_MOTION: u8 = 1;
const HIGH_CONTRAST: u8 = 2;
static POLICY: AtomicU8 = AtomicU8::new(0);

/// Missing preferences leave the last known policy unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Preferences {
    pub high_contrast: Option<bool>,
    pub reduce_motion: Option<bool>,
}

#[cfg(any(test, feature = "test-support"))]
thread_local! {
    // Injected UI tests must not change another test's background palette reads.
    static OVERRIDE: std::cell::Cell<Option<u8>> = const { std::cell::Cell::new(None) };
}

fn snapshot() -> u8 {
    #[cfg(any(test, feature = "test-support"))]
    if let Some(policy) = OVERRIDE.get() {
        return policy;
    }
    POLICY.load(Ordering::Relaxed)
}

/// Updates available preferences and reports whether high contrast changed.
/// The app's policy owner calls this before refreshing its windows.
pub fn store(preferences: Preferences) -> bool {
    let previous = snapshot();
    let mut policy = previous;
    for (flag, value) in [
        (REDUCE_MOTION, preferences.reduce_motion),
        (HIGH_CONTRAST, preferences.high_contrast),
    ] {
        if let Some(value) = value {
            policy = if value { policy | flag } else { policy & !flag };
        }
    }
    #[cfg(any(test, feature = "test-support"))]
    if OVERRIDE.get().is_some() {
        OVERRIDE.set(Some(policy));
        return (previous ^ policy) & HIGH_CONTRAST != 0;
    }
    POLICY.store(policy, Ordering::Relaxed);
    (previous ^ policy) & HIGH_CONTRAST != 0
}

pub fn reduce_motion() -> bool {
    snapshot() & REDUCE_MOTION != 0
}

pub fn high_contrast() -> bool {
    snapshot() & HIGH_CONTRAST != 0
}
