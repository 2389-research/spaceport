# wx47: a pane's empty OSC 52 clear no longer wipes a fresh copy

Worktree: `pi-worktree-d68f038b-da0a-4ba6-ba58-0bd45ed31579-s0-0`
Base: `65c532dc4` (origin/main)
Branch: `feat/wx47-osc52-empty-clear`

## The bug

A program in a pane can set the clipboard with OSC 52; Spaceport decodes the
payload and writes it in `apply_clipboard_writes`
(`crates/spaceport-gpui/src/endpoint/polling.rs`). An *empty* payload is a
deliberate clipboard **clear** — `crates/spaceport-osc52/src/tests.rs` pins
`decode("")  == Some("")` — but the window wrote it like any other payload and
raised the `copied to clipboard` flash anyway. A pane program's clear that
arrived in the same drain as the user's own copy-on-select write therefore
wiped the copy they had just made, while the flash still said it was copied.

## The fix

Supervisor's policy, kept minimal: a pane-initiated empty payload never writes
and never toasts. No time-window clobber guard — that was considered and
rejected as over-engineered for now.

- `apply_clipboard_writes` skips empty payloads, logging each at debug level
  ("dropped the terminal's empty clipboard clear"), and returns early when
  nothing non-empty was written. The return value still means "something was
  written", so the window repaints for the flash exactly as before.
- The terminal's copy is now worded apart from the user's own selection copy:
  `endpoint-copied` in `crates/spaceport-gpui/locales/en-US/endpoint.ftl` reads
  `copied from the terminal`, while `window-find-copied` stays
  `copied to clipboard`. English is the only language directory, so no other
  locale file mirrors the key.
- `crates/spaceport-gpui/README.md`'s OSC 52 paragraph matches the behavior: the
  flash reads `copied from the terminal`, and an empty write is dropped.

## Tests

New topic file `crates/spaceport-gpui/src/endpoint/tests/clipboard_writes.rs`
(registered in `endpoint/tests.rs`), which also takes over the existing
pasteboard/toast test so all clipboard-write tests live in one file:

- `daemon_clipboard_writes_reach_the_pasteboard_and_report` (moved, unchanged in
  substance): a copy reaches the pasteboard and reports itself; a disabled toast
  stays silent; an empty iterator writes nothing.
- `empty_clipboard_clears_never_write_or_report`: seeded clipboard, only empty
  payloads -> no write, no flash, returns `false`.
- `an_empty_clear_between_copies_leaves_the_text_and_one_flash`: text plus a
  trailing clear in one drain -> the text is what the clipboard holds and the
  one flash is `endpoint-copied`.
- `a_terminal_copy_is_worded_apart_from_the_users_own_selection_copy`: pins both
  English wordings so the two copies cannot collapse back together.

`decode("") == Some("")` in `spaceport-osc52` is untouched and still passes.

Both new window tests were checked against the unfixed code first: with the
empty-payload skip removed, `empty_clipboard_clears_never_write_or_report` and
`an_empty_clear_between_copies_leaves_the_text_and_one_flash` fail (the second
on `left: None, right: Some("copied by the pane")` — the clear had wiped the
copy). With the fix restored, both pass.

## What was run

```
python3 scripts/release/sparkle_framework.py acquire target/sparkle-sdk
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 \
  SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk \
  cargo check -p spaceport-gpui --all-targets --features integration-test   # ok
… cargo clippy -p spaceport-gpui --all-targets --features integration-test -- -D warnings   # ok
… cargo test -p spaceport-gpui --bin spaceport clipboard                    # 55 passed
… cargo test -p spaceport-gpui --bin spaceport endpoint::                   # 255 passed
… cargo test -p spaceport-osc52                                             # 3 passed
cargo fmt --all -- --check                                                  # clean
python3 scripts/check-i18n.py                                               # 2114 messages valid and used
just check-file-size                                                        # within 1000 lines
just check-sorted                                                           # sorted
```

Not run: `just ci`, the full workspace test matrix, and native GUI checks —
those stay with CI. No desktop check was needed: the change is which payloads
are written and which flash text is used, both covered headlessly.

## Residual risks

- A pane program that legitimately wants to *clear* the clipboard cannot do so
  through this client any more. That is the approved trade: the clipboard here
  belongs to the user's gestures, and a clear arriving milliseconds after their
  own copy is indistinguishable from a clobber.
- A non-UTF-8 or oversized payload was already dropped before this change; the
  only new drop is the decodable empty one.
