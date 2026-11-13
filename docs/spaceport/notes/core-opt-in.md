# Herdr Core opt-in

## Approved design

Doctor Biz approved this design through Q&A on November 13, 2026.
Herdr Core defaults off for new and existing installs until explicitly enabled.
Settings > General > Experimental contains “Enable Herdr Core”.
Helper text says “Adds Core terminals and groups. Requires restart.”
Changes save immediately and take effect after restarting Spaceport.
The effective setting belongs to the app session and stays consistent across windows.
While off, hide Core UI and prevent Core connection and runtime-start attempts.
Core-only CLI actions return an explicit disabled error.
Preserve existing Core preferences, presentation documents and terminal processes.
Regular Herdr remains available.

Doctor Biz approved implementation after reviewing this design.
This note follows the canonical Spaceport plan location rather than creating a parallel task graph.
The initial feature approval excluded release, signing, runtime installation and data migration.

On November 13, Doctor Biz also authorized committing, merging and tagging a new version.
This approval covers the Core opt-in delivery through the existing macOS preview workflow.
The separate ACP helper naming work is excluded from this release cut.

## Implementation and verification

- [x] Add the saved `herdr_core_enabled` preference, default false, and immutable app-session gate.
- [x] Cover false defaults, invalid values, saved edits and restart-only effective state with regression tests.
- [x] Gate Core polling, creation, restoration, fallback and Core-only control routes.
- [x] Hide Core UI and search/menu entries while the app-session gate is off.
- [x] Add the Experimental setting using existing persistence and CLI config support.
- [x] Preserve saved Core view references when disabled; never stop Core or its terminals.
- [x] Exercise disabled startup against a private listening fixture and prove no connection.
- [x] Exercise opt-in and restart semantics with private real producers and existing Core fixtures.
- [x] Verify native settings and absence of Core UI, including narrow and pseudo-locale cases.
- [x] Run quick canonical checks, focused tests and one independent lifecycle/contract review.
- [x] Record exact evidence and platform limits; commit/push only within the requested delivery scope.

Use Rust 1.99.0, `--locked`, `CARGO_INCREMENTAL=0` and the existing display-copy target.
Keep full workspace and platform checks in hosted CI.

## Results

The implementation lives on `feat/core-opt-in` in `.worktrees/core-opt-in`.
Delivery proceeds through a reviewed PR and successful main CI before the new version tag.
[Evidence and native screenshots](../evidence/core-opt-in/README.md) describe the isolated fixtures and their limits.

The following commands passed locally with the environment above:

- `just format-check check-file-size check-sorted check-i18n`: all checks passed; 20 tooling tests passed.
- `cargo test --locked -p sheprd-gpui --features integration-test --bin spaceport`: 3,885 passed, 10 ignored.
- `cargo test --locked -p sheprd-gpui --features integration-test --test cli`: 22 passed.
- `cargo clippy --locked -p sheprd-gpui --all-targets --features integration-test -- -D warnings`: passed.
- `cargo clippy --locked -p sheprd-gpui --all-targets -- -D warnings`: passed.
- `cargo test --locked -p sheprd-gpui --features integration-test --test live_gui native_local_controls -- --ignored --exact`: passed.
- `cargo test --locked -p sheprd-gpui --features integration-test --test live_gui sidebar_routing::native_host_routing -- --ignored --exact --nocapture --test-threads=1`: passed.
- From `docs/spaceport/`: `uv run --no-project --with-requirements tools/requirements.txt python tools/spec.py check`: passed.
- `git diff --check`: passed.

Logs remain under this worktree's ignored `target/core-opt-in/` directory.
`green-suite.log`, `cli.log`, `clippy-feature.log` and `clippy-default.log` hold the final suite and lint results.
`static-green.log` and `spec-final.log` hold the static checks.
`native-final.log` and `native-routing.log` hold final native results.
The earlier interrupted suite runs do not count as passing evidence.
The existing local-network permission test passed after its 60-second cleanup wait.
The existing `block 0.1.6` future-compatibility warning and debug linker unwind warning remain.

Independent review found two visible bypasses in the status bar and saved-tab renderer.
Actual rendering regressions reproduced each bypass before its fix.
Both regressions pass in the final suite.
Review found no additional runtime lifecycle or saved-document contract issue.

Full workspace CI, `macos-updater` builds and Linux validation are deferred to hosted CI.
Windows CI stays paused under D66.
Native installed-Core lifecycle, IME and OS focus checks were not run.
