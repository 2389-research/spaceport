// ABOUTME: Counts bytes submitted to real Markdown parsers in scoped tests.
// ABOUTME: Restores nested measurements and isolates concurrent test threads.
use std::cell::Cell;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Counts {
    pub(super) protection_bytes: usize,
    pub(super) markdown_bytes: usize,
}

thread_local! {
    static BYTES: Cell<Counts> = const { Cell::new(Counts { protection_bytes: 0, markdown_bytes: 0 }) };
}

pub(super) fn record_protection(bytes: usize) {
    BYTES.with(|count| {
        let mut value = count.get();
        value.protection_bytes = value.protection_bytes.saturating_add(bytes);
        count.set(value);
    });
}

pub(super) fn record_markdown(bytes: usize) {
    BYTES.with(|count| {
        let mut value = count.get();
        value.markdown_bytes = value.markdown_bytes.saturating_add(bytes);
        count.set(value);
    });
}

struct Restore(Counts);

impl Drop for Restore {
    fn drop(&mut self) {
        BYTES.set(self.0);
    }
}

pub(super) fn measure<T>(run: impl FnOnce() -> T) -> (T, Counts) {
    let restore = Restore(BYTES.replace(Counts::default()));
    let result = run();
    let bytes = BYTES.get();
    drop(restore);
    (result, bytes)
}
