# T03 CLI new-window target for Core groups

Source base: integration `a13a511b9`. `spaceport window new` still read
`endpoints[0]`, although a Core-only window has no legacy endpoint. A
headless test with a selected Core group and zero endpoints first failed in
the test fixture because GPUI was asked to read a window already on its
update stack. After moving the call outside that update, it failed at the
actual production `endpoints[0]` read: index out of bounds, length zero.

The CLI server now selects a typed `WindowLaunch`. A named window or the
frontmost window contributes its exact selected `CoreGroupKey` when Core is
active, or its existing legacy launch target while that path still exists. An explicit
caller daemon socket keeps its exact legacy target. The Core selection must
match the fresh observed hub and group before the request queues a new
window; stale or same-ID/different-hub observations return `NotFound`. The
existing Core constructor then joins with `Start::Join`. The response still
says only that opening was asked for, because a later window failure is not
an acknowledgement. The command and docs keep their existing identity and
output shape; no backend RPC was added.

The corrected new test passed 1/1; the full headless app-control group passed
9/9, including legacy exact-socket and Core window-list checks. The real CLI
integration group passed 20/20. Default-feature and all-target/all-feature
app checks passed with pinned Rust 1.99.0 and `--locked --offline`. `just
format-check`, `check-file-size`, `check-sorted`, `check-i18n`, package
integrity and its 21 tests, and `git diff --check` passed. The first format
check identified one ordinary rustfmt change; it passed after formatting.
Only the inherited macOS compact-unwind linker warning and `block v0.1.6`
future-compatibility notice remained.

This does not prove native window creation or a live Core join. A Core pane
without `--window` has no proved environment identity for group selection;
the existing frontmost-window rule applies. Core tab tear-off, remote
native group parity, Windows and Linux tests remain open. T03 remains
`in_progress`; A09, A24 and A33 remain `not_run`.
