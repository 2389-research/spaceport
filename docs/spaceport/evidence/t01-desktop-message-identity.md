# T01: Desktop messages name their actual product owner

Kata `jc6s`, child of `rxkr`, addresses SP-022 from base `0eaed5cf5` in
`.worktrees/desktop-message-identity`, branch `fix/desktop-message-identity`.

## Source ownership

The vendored protocol's recorded Herdr source is commit
`856b64b9bfd41b9d3d82e2375ed5b4bf941fda28` (`crates/sheprd-protocol/NOTICE.md`).
Read-only inspection of that exact source establishes the release-data owner:

- [`src/update.rs`](https://github.com/herdrdev/herdr/blob/856b64b9bfd41b9d3d82e2375ed5b4bf941fda28/src/update.rs)
  checks Herdr's manifest and chooses Herdr/package-manager update commands.
- [`src/server/client_shell.rs`](https://github.com/herdrdev/herdr/blob/856b64b9bfd41b9d3d82e2375ed5b4bf941fda28/src/server/client_shell.rs)
  copies daemon update availability and install-command state into the snapshot.
- [`src/product_announcements.rs`](https://github.com/herdrdev/herdr/blob/856b64b9bfd41b9d3d82e2375ed5b4bf941fda28/src/product_announcements.rs)
  chooses announcements using the daemon's own build version.

These headings therefore identify Herdr. Labeling their versions as Spaceport
would misstate the data's owner. The settings fallback and missing-state-directory
error describe Spaceport instead. Shared chat rejection text can say “this
client” without making the protocol crate depend on a desktop brand.

This scope changes presentation text only. Snapshot ownership, install commands,
notice dismissal, release selection and serialized rejection values stay intact.
It does not enable an updater or satisfy a Spaceport release/update gate.

## Verification

Rust commands used `RUSTUP_TOOLCHAIN=1.99.0`, empty `HERDR_BUILD_PR_NUMBER` for
the app, and `.worktrees/sparkle-updates/target` as `CARGO_TARGET_DIR`.

- TDD: the three `desktop_identity` cases failed on the old UI labels, and the
  protocol product-identity case failed on its old display text. Catalog tests
  use the runtime formatter because the message IDs are test data; production
  `fl!` calls still compile-check every argument against the catalog schema.
- `cargo test --locked -p sheprd-gpui --bin spaceport desktop_identity`: three
  passed, covering Herdr release labels/version data, Spaceport fallback errors
  and product-neutral rejection text in English and both pseudo-locales where
  identifiers are present.
- `cargo test --locked -p sheprd-chat-protocol`: all 59 tests passed. The new
  case checks the same `unsupported` JSON value and round trip; existing unknown
  enum, framing and reducer checks remain intact.
- `cargo test --locked -p sheprd-gpui --bin spaceport announcement`: ten passed,
  including headless card/sheet dismissal and preserved wire parameters.
- The exact `every_devices_variant_has_a_message` and
  `every_chat_rejection_has_a_message` filters each passed one parity test.
  An earlier singular device filter selected zero tests and is not proof.
- `cargo test --locked -p sheprd-gpui --bin spaceport about`: 11 passed. Moving
  the existing Herdr name into app constants preserves attribution text.
- `cargo test --locked -p sheprd-gpui --test removed_updater`: both passed,
  including real compiled product/runtime names and both release-refusal cases.
- `the_release_notes_sheet_shows_only_messages_under_en_xa`: passed one
  headless translated-sheet check.
- Strict app/protocol all-targets Clippy, canonical format/file-size/sorting/i18n,
  specification integrity and whitespace checks passed.

Logs remain in this worktree's `target/desktop-message-identity/`. Existing
macOS compact-unwind linker and `block 0.1.6` future-compatibility warnings remain
unsuppressed. These are catalog, protocol and headless window checks using the
existing protocol peer; no real runtime update or native end-to-end behavior is
claimed. The desktop remains locked, so native glyph/layout/IME checks and
screenshots are pending alongside platform CI. Broad T01/A42 and `rxkr` stay open.
