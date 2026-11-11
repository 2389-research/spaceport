# T11: Native macOS performance fixture selects Spaceport

Kata `dh46`, under `a5ks`, is a bounded native-fixture correction. Base:
`d0e076e046ae490c3d942bdd7a914570a88440b7`, branch
`fix/macos-performance-selector`, worktree `.worktrees/macos-perf-selector`.

The AppKit adapter searched its own process's windows for `Sheprd`, while startup
used `constants::WINDOW_TITLE` (`Spaceport`). The selector now reads that same
constant. Event construction, renderer, timing budget and unsafe scope are unchanged.

The existing `performance_gui` test now uses the shared Sandbox with a cleared,
private environment. Only the optional performance budget is forwarded. A guard
reaps its exact child before sandbox removal, including on failure. Successful
runs assert directory cleanup. The test requires release mode and verifies the
existing scroll, glyph-cache, changed-cell/popup/cursor and completion markers.
Shared daemon-only helpers and their precondition assertions have scoped test
lint expectations; the harness's own code retains strict linting.

## Native result

macOS 27.2.1, arm64, Rust 1.99.0, optimized release build. Command:

```sh
HERDR_PERF_P95_MS=30 cargo test --locked --release -p sheprd-gpui --features integration-test --test performance_gui -- --ignored --nocapture
```

`RUSTUP_TOOLCHAIN=1.99.0`, empty `HERDR_BUILD_PR_NUMBER` and the existing
`.worktrees/sparkle-updates/target` Cargo cache were selected explicitly.

- TDD baseline: first frame painted, then `PERF FAIL: fixture window not found`.
- Final native test passed at the unchanged 30ms budget. Warm p95 milliseconds:
  hover 3.810, scroll 5.765, terminal 1.531. Both lists moved; 140 cached/fresh
  glyph layouts matched; changed cells, centered popup and cursor checks passed.
- Both shared sandbox unit tests passed: private-directory cleanup on unwind,
  and the cleared allowlisted child environment.
- Strict app/native-test Clippy, formatting, source-size, sorting and whitespace
  checks passed. The existing `block 0.1.6` future-compatibility warning remains.
- Specification integrity/schema checks passed. Fresh-eyes review found no
  blocker in target selection, environment handling or child/sandbox cleanup.

One added marker assertion initially expected boolean phase labels; the driver
uses indices 0/1. That failed test is retained separately; the assertion was
corrected from source and the complete test rerun. Each rerun followed a code
change; no automatic retry or budget change was added.

Logs: this worktree's ignored `target/macos-perf-selector/`, including
`native-red.log`, `phase-assertion-failure.log`, `native-green.log`,
`sandbox-tests.log` and `clippy.log`. The fixture used no daemon or personal app.
This is local macOS evidence; Windows performance variability (`h8g5`), full CI,
T11 and broader resource/platform acceptance remain open.
