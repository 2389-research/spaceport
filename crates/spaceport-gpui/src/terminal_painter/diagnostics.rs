//! How long a paint took and how many of its primitives failed, reported at
//! most once per `REPORT_INTERVAL`.
//!
//! gpui-free: a future `spaceport-cells` crate takes this file as it stands.

use std::time::{Duration, Instant};

/// A paint's errors are reported, and its timings logged, this often.
pub(super) const REPORT_INTERVAL: Duration = Duration::from_secs(5);
/// A paint slower than this is worth a warning rather than a debug line.
pub(super) const SLOW_PAINT: Duration = Duration::from_millis(16);

#[derive(Default)]
pub(super) struct PaintTiming {
    pub(super) count: u64,
    pub(super) total: Duration,
    pub(super) max: Duration,
    pub(super) slow_count: u64,
}

pub(super) struct PaintDiagnostics {
    since: Instant,
    timing: PaintTiming,
    last_error: Option<Instant>,
    errors: u64,
}

impl PaintDiagnostics {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            since: now,
            timing: PaintTiming::default(),
            last_error: None,
            errors: 0,
        }
    }

    pub(super) fn record(&mut self, now: Instant, elapsed: Duration) -> Option<PaintTiming> {
        self.timing.count += 1;
        self.timing.total += elapsed;
        self.timing.max = self.timing.max.max(elapsed);
        self.timing.slow_count += u64::from(elapsed > SLOW_PAINT);
        if now.duration_since(self.since) < REPORT_INTERVAL {
            return None;
        }
        self.since = now;
        Some(std::mem::take(&mut self.timing))
    }

    pub(super) fn take_errors(&mut self, now: Instant, errors: u64) -> Option<u64> {
        self.errors = self.errors.saturating_add(errors);
        if self.errors == 0
            || self
                .last_error
                .is_some_and(|last| now.duration_since(last) < REPORT_INTERVAL)
        {
            return None;
        }
        self.last_error = Some(now);
        Some(std::mem::take(&mut self.errors))
    }
}
