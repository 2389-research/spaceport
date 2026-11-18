// ABOUTME: Counts bytes submitted to the real protection parser in scoped tests.
// ABOUTME: Keeps streaming work measurements separate for each test thread.
use std::cell::Cell;

thread_local! {
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn record(bytes: usize) {
    BYTES.with(|count| count.set(count.get().saturating_add(bytes)));
}

pub(in crate::chat::rich) fn measure<T>(run: impl FnOnce() -> T) -> (T, usize) {
    let previous = BYTES.replace(0);
    let result = run();
    let bytes = BYTES.replace(previous);
    (result, bytes)
}
