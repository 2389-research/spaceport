# T01: Setup relay diagnostics name Spaceport

Kata `ab9s`, child of `rxkr`, addresses SP-022 on base `ee8f18a55` in
`.worktrees/setup-relay-branding`, branch `fix/setup-relay-branding`.

The setup relay's closed-setup error, failed command output and session helper
diagnostics still named Sheprd. They now read the existing window-title constant,
which comes from the shared client product name. Fluent receives the name as data
so both pseudo-locales preserve it. Error variants, sources, status 127, socket
claims, terminal/session ownership and launch behavior retain their contracts.

## Verification

Rust commands selected `RUSTUP_TOOLCHAIN=1.99.0`, empty `HERDR_BUILD_PR_NUMBER`
and `.worktrees/sparkle-updates/target` as `CARGO_TARGET_DIR`.

- New formatting/localization checks and the updated real failed-launch check
  first failed on Sheprd. The external helper test also first failed on its
  actual stderr: `Sheprd could not prepare the setup terminal`.
- `cargo test --locked -p sheprd-gpui --bin spaceport setup_relay` then passed
  16 tests. These cover typed/root formatting, English and both pseudo-locales,
  actual local socket/PTY relay behavior, malformed frames, failed launch,
  resize, timeout, cancellation, private claims and cleanup. The existing
  all-variant English Display/localization parity check passed too.
- `cargo test --locked -p sheprd-gpui --test cli setup_relay` passed the real
  helper subprocess case: null stdin refuses terminal preparation, exits 127,
  prints the Spaceport diagnostic and never starts GPUI or a setup command.
- `cargo clippy --locked -p sheprd-gpui --all-targets -- -D warnings` passed.
- `just format-check check-file-size check-sorted check-i18n`, specification
  integrity checks and `git diff --check` passed.

Logs remain under this worktree's `target/setup-relay-branding/`. The existing
debug linker compact-unwind and `block 0.1.6` future-compatibility warnings remain
unsuppressed. A test-only unused import appeared in the initial red run and was
corrected before the green run and strict lint.

These are real formatter, local relay and executable checks; no host was
contacted or permission changed. Native dialog glyph/IME inspection and full
other-platform checks remain unproved locally. Broad T01/A42 and `rxkr` remain
open. Final review found no blocker in this bounded message change.
