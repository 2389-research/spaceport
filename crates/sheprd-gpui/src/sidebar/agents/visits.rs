// ABOUTME: Counts actual snapshot row visits during deterministic lookup-budget tests.
// ABOUTME: Keeps instrumentation local to each test thread and absent from production builds.
use std::cell::Cell;

thread_local! {
    static VISITS: Cell<usize> = const { Cell::new(0) };
}

pub(in crate::sidebar) fn record() {
    VISITS.set(VISITS.get() + 1);
}

pub(in crate::sidebar) fn measure<T>(work: impl FnOnce() -> T) -> (T, usize) {
    VISITS.set(0);
    let result = work();
    (result, VISITS.replace(0))
}
