# T01: Chat startup failures name Spaceport

Kata `z9wm`, child of `rxkr`, addresses SP-022 from base `0eaed5cf5` in
`.worktrees/chat-helper-discovery`, branch `fix/chat-helper-discovery`.

The disconnected, helper-missing and helper-lookup-pending errors still named
Sheprd. They now use the existing shared product name. The preferred helper
filename comes from the existing bundled-name mapping, extracted into one
function used by both lookup and presentation. Fluent receives both identifiers
as data so pseudo-locales preserve them, including `.exe` on Windows.

The legacy `herdr-acp` lookup already prefers the shipped `sheprd-acp` before its
fallback; this was not a discovery bug. A new chat-level filesystem check passed
before the message changes, proving current packaged-name selection and bundled
legacy precedence over PATH. No lookup order, supported name, remote-host script,
tab/session ownership, permission or runtime lifecycle policy changed.

## Verification

Cargo used Rust 1.99.0, empty `HERDR_BUILD_PR_NUMBER` and the existing
`.worktrees/sparkle-updates/target` cache as `CARGO_TARGET_DIR`.

- TDD `start_branding`: the two message checks first failed on Sheprd, while the
  shipped-helper lookup check passed before any production edit.
- `cargo test --locked -p sheprd-gpui --bin spaceport chat::tests::start`:
  14 passed, including English/pseudo-localized branding and existing start,
  refusal, command/environment and replaced-connection behavior.
- `cargo test --locked -p sheprd-gpui --bin spaceport bundled::tests`: six passed,
  covering preferred and legacy names, bundle/PATH precedence and external Core
  boundaries after extracting the unchanged mapping.
- `cargo test --locked -p sheprd-gpui --bin spaceport every_chat_start_variant_has_a_message`:
  passed all-variant English Display/localized-message parity.
- `cargo run --locked -p sheprd-acp -- --help`: built and ran the actual
  `sheprd-acp` executable successfully. The help path starts no agent or daemon.
- `cargo clippy --locked -p sheprd-gpui --all-targets -- -D warnings`,
  `just format-check check-file-size check-sorted check-i18n`, specification
  integrity and whitespace checks passed.

Logs remain under this worktree's `target/chat-helper-discovery/`. Existing
macOS compact-unwind linker and `block 0.1.6` future-compatibility warnings remain
unsuppressed. Filesystem probes test lookup only; their files are never executed.
Existing startup window tests use the headless protocol peer. Only the helper
help smoke check runs a real executable; none proves a real agent conversation.
The desktop reports locked, so native chat-dialog glyph/IME checks and screenshots
remain pending, along with platform CI. Broad T01/T06/A42 and `rxkr` remain open.
