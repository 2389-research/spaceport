# T01: Local-network guidance names Spaceport

Kata `z4a0`, a bounded child of `rxkr`, addresses SP-022. Base source:
`b331add6fbdceab53bb89f6e3f4d37c473d7ef73`, branch
`fix/spaceport-local-network-copy`, worktree `.worktrees/local-network-branding`.

The SSH denial diagnostic and Add Device waiting message still told people to
allow Sheprd in macOS Settings. The product name now has one owner,
`sheprd_client::APP_NAME`: client diagnostics need it without depending on GPUI.
The existing window-title constant reads it, and Fluent receives it as data so
both pseudo-locales preserve Spaceport. Host data and isolation marks remain.

The dialog also bypassed localization after a denied wait. That branch now uses
the existing `Localize` implementation. Permission handling, host claims, retry
timing, cancellation, host-key policy and daemon behavior are unchanged.

## Verification

Cargo used `RUSTUP_TOOLCHAIN=1.99.0`, `HERDR_BUILD_PR_NUMBER=` and the existing
`.worktrees/sparkle-updates/target` cache as `CARGO_TARGET_DIR`.

- Baseline `cargo test --locked -p sheprd-gpui --bin spaceport local_network`
  passed three dialog-state tests. TDD then failed the new product-name checks
  and the existing dialog assertion on the old Sheprd text.
- A separate dialog regression failed after the strings were corrected: the
  refusal still rendered plain English under `en-XA`. It passed after the
  actual dialog used `failure.localize()`.
- The final `local_network` filter passed six tests: brand/host preservation in
  English, `en-XA` and `en-XB`, plus wait, cancellation and denied/allowed states.
- `every_ssh_variant_has_a_message` passed, preserving English Display/localized
  error parity for every SSH variant and wrapper.
- `menu::devices::tests::en_xa` passed three dialog/picker translation checks;
  `constants::tests` passed two identity checks.
- `cargo test --locked -p sheprd-client ssh::failure::tests` passed nine tests.
- Strict client/app/test Clippy, format, source-size, sorting, i18n,
  specification schema/integrity and whitespace checks passed.
- Fresh-eyes review found no blocker; the shared name follows the existing
  client-to-GPUI dependency direction and the dialog uses its typed formatter.

Logs are retained under this worktree's ignored `target/local-network-branding/`.
These are real formatting and headless dialog checks with isolated fixture
claims. They do not connect to a host, grant permissions or open Settings.
Native permission-dialog/glyph checks and full platform CI remain separate;
parent `rxkr`, T01 and A42 stay open. Existing linker compact-unwind and
`block 0.1.6` future-compatibility warnings remain unsuppressed.
