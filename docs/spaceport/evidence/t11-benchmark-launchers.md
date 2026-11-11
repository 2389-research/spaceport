# T11: Isolated native benchmark launchers

Kata `q9e3`, child of `a5ks`, follows the macOS window-selector repair in
`e30e41cfd`. Worktree `.worktrees/benchmark-launchers`, branch
`fix/isolated-benchmark-launchers`, now based on `ee8f18a55`.

`just test-perf` and Windows native CI launched the executable directly with the
caller's state environment. The Just recipe also assumed a local `target/`
directory. With an external `CARGO_TARGET_DIR` and no executable at that local
path, the old recipe reproduced exit 127 without launching an app.

Both launchers now select the existing `performance_gui` sandbox runner through
Cargo. The target supports macOS and Windows, obtains the actual executable from
Cargo and uses the shared cleared environment and private state paths. The
runner stops and reaps only its own child before removing its sandbox. A final
absence assertion checks successful cleanup. Linux still lacks native benchmark
event delivery; the recipe refuses it explicitly, as the existing CLI does.

The cached test requires release mode, both native scroll phases, cached/fresh
glyph equality, changed-cell/popup/cursor checks and a budget pass. The reference
test uses the same runner and requires its own mode and scroll/budget markers.
The Just recipe retains presence-based `HERDR_PERF_UNCACHED` selection, including
`0`; explicit cached CI selection cannot inherit reference mode. The local 30ms
and Windows CI 100ms budgets remain unchanged. Reference comparison uses the
existing 250ms release default. No timeout, retry or rendering policy changed.

## Verification

Local checks used Rust 1.99.0, empty `HERDR_BUILD_PR_NUMBER`, and
`.worktrees/sparkle-updates/target` as the external Cargo target directory on
macOS 27.2.1 arm64. Logs remain in `target/benchmark-launchers/` in this worktree.

- `just test-perf` passed the real cached native fixture at 30ms.
  It passed again on repaired base `ee8f18a55`: worst warm p95 4.780ms, both
  native scroll phases and all cached correctness markers present, cleanup
  assertion passed (`updated-base-native.log`).
- `HERDR_PERF_UNCACHED=0 just test-perf 250` passed real reference mode.
- Explicit cached Cargo selection with parent `HERDR_PERF_UNCACHED=1` passed at
  30ms and reported cached mode, proving the child does not inherit that flag.
- Both modes emitted native scroll markers and `PERF PASS`; cached mode also
  checked glyph equality, changed cells, popup and cursor. Each successful run
  checked sandbox removal after exact-child reaping.
- `cargo test --locked --release -p sheprd-gpui --features integration-test --test performance_gui sandbox::tests`
  passed both existing shared-sandbox checks.
- `cargo test --locked -p sheprd-gpui --bin spaceport --features integration-test cli::tests`
  passed 49 parsing tests.
- `cargo clippy --locked -p sheprd-gpui --all-targets --features integration-test -- -D warnings`
  passed. Existing debug linker compact-unwind and `block 0.1.6`
  future-compatibility warnings remain unsuppressed.
- `just format-check check-file-size check-sorted check-i18n`, recipe Bash
  syntax/ShellCheck, `actionlint .github/workflows/ci.yml` and whitespace checks
  passed. `just audit-workflows` reported no findings, with existing offline and
  unsupported-Python-shell warnings, and unchanged suppressions.
- Independent review found no blocker after correcting the platform ignore
  message and stale documentation about GPUI feature selection.

This changes fixture launch paths and integration-only CLI help, with no visible
product UI change. Native Windows execution and full platform CI remain pending;
the PR must pass them before merge. This does not resolve Windows timing
variability tracked in `h8g5`, or close broad T11/resource/accessibility gates.
