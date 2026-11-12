# T01: Version mismatch guidance names Spaceport

Kata `zcbd`, child of `rxkr`, addresses SP-022 from base `640f3c204` in
`.worktrees/version-guidance-branding`, branch `fix/version-guidance-branding`.

The endpoint/SSH refusal diagnostics and client-update notice still named
Sheprd. The existing client product-name constant now supplies the update
advice; Fluent receives it as data for the three diagnostics and notice title.
There is still one product-name definition. Only a strictly newer protocol
generation asks for an app update. Older or equal generations retain the
existing Herdr command. No handshake classification, connection, runtime
lifecycle, update execution or retry policy changed.

## Verification

Cargo used Rust 1.99.0, empty `HERDR_BUILD_PR_NUMBER`, and the existing
`.worktrees/sparkle-updates/target` as `CARGO_TARGET_DIR`.

- TDD: `cargo test --locked -p sheprd-client generation` failed both newer-server
  checks on the old Sheprd advice. The first app `version_guidance` run failed
  the localized-error and notice-title brand checks; the older/equal-generation
  Herdr advice check already passed.
- `cargo test --locked -p sheprd-client version_mismatch`: eight passed,
  preserving classification, version handling and refusal/session behavior.
- `cargo test --locked -p sheprd-client ssh::tests::incompatible`: three passed.
  These use a scripted bridge peer, not a live host or an end-to-end SSH test.
- `cargo test --locked -p sheprd-gpui --bin spaceport version_guidance`: three
  passed. English and both pseudo-locales preserve the brand, reported version
  and host data; older/equal-generation guidance retains `herdr update`.
- `cargo test --locked -p sheprd-gpui --bin spaceport version_mismatch`: seven
  passed, including the notice data test above and headless window checks for
  one-time announcement, command copying, rearming and absent client-update
  actions when no such action exists.
- `cargo test --locked -p sheprd-gpui --bin spaceport every_client_variant_has_a_message`:
  passed the all-variant English Display/localized-message parity check.
- `cargo clippy --locked -p sheprd-client -p sheprd-gpui --all-targets -- -D warnings`,
  `just format-check check-file-size check-sorted check-i18n`, specification
  integrity and whitespace checks passed.

Logs remain in this worktree's `target/version-guidance-branding/`. Existing
debug linker compact-unwind and `block 0.1.6` future-compatibility warnings
remain unsuppressed. The local checks are formatter, protocol-peer and headless
window checks. macOS reported `CGSSessionScreenIsLocked=Yes`, so no native dialog
glyph/layout/IME checks or screenshots were taken. Platform CI and native manual
QA remain pending. Broad T01/A42 and `rxkr` stay open.
