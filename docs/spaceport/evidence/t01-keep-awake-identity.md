# T01: Keep-awake OS identity

Kata `b1f6`, a bounded child of `rxkr`, addresses SP-022. Base source:
`6f30f47072d7c2ea67470cf67f26227775053acc`, branch
`fix/spaceport-keep-awake`, worktree `.worktrees/keep-awake-branding`.

The real power request still named Sheprd. Its app name and both reason strings
now derive from `constants::WINDOW_TITLE`. Scope, idle/display flags, reverse
domain, guard ownership and worker behavior are unchanged. Pinned keepawake
0.6.1 uses the reason on macOS/Linux and the app name on Linux.

## Verification

Cargo used `RUSTUP_TOOLCHAIN=1.99.0`, `HERDR_BUILD_PR_NUMBER=` and the existing
`.worktrees/sparkle-updates/target` dependency cache as `CARGO_TARGET_DIR`.

- Baseline `cargo test --locked -p sheprd-gpui --bin spaceport caffeine::tests`
  passed 20 tests. These state/worker tests use stand-in guards, not the OS.
- The new opt-in test failed before the fix: this test process owned a real
  `PreventUserIdleSystemSleep` assertion named `Sheprd keeps the system awake`.
  The guard dropped before the name assertion, even in the failing case.
- After the fix, the exact native test passed:
  `cargo test --locked -p sheprd-gpui --bin spaceport caffeine::tests::power_assertions::spaceport_power_assertions_follow_scope_and_release -- --ignored --exact`.
  System scope has one idle-system assertion; display scope has idle-system
  and idle-display assertions. Names match Spaceport; guard drop removes them.
- The normal Caffeine suite passed 20 tests, with this native case explicitly
  ignored there and run separately above.
- Strict app/test Clippy (`-D warnings`), formatting, source-size, sorting,
  i18n, specification schema/integrity and whitespace checks passed.
- Fresh-eyes review found no blocker: the change stays on the worker path,
  preserves power flags and uses the existing product-name constant.

The test calls the real keep-awake implementation and `pmset -g assertions`.
It filters results to its exact process ID before checks or diagnostics. It
does not open the personal app, contact a daemon or alter another process's
holds. Logs live under this worktree's ignored `target/keep-awake-branding/`.

Baseline linker compact-unwind and `block 0.1.6` future-compatibility warnings
remain; neither was suppressed. Linux/Windows native behavior and full platform
CI were not run locally. This is native power-API evidence, not full GUI or
side-by-side installation acceptance. Parent `rxkr`, T01 and A42 remain open.
