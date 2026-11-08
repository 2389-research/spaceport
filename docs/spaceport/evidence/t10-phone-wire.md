# Retired phone chat greeting

At `ee75a851`, `sheprd-chat-protocol/src/wire.rs` still declared `ClientKind::Remote` for the deleted phone daemon. A source search found no retained constructor of that variant. The current GUI sends `ClientKind::Gui` in `sheprd-gpui/src/chat/link.rs`; `sheprd-acp/src/attach.rs` reads the hello fields it needs and ignores the client kind. The enum's `#[serde(other)] Other` case already handles unknown JSON names. The saved `[remote]` settings table is inert and reported as unknown by the config test; the old `remote` CLI and control requests have rejection tests. No phone push/server endpoint constant or live package input appeared in the inspected crates and release sources. This source inventory cannot prove that no old personal service is running.

The cut removes only `ClientKind::Remote` and its phone comment. An old `"client":"remote"` JSON hello still decodes as `ClientKind::Other`. By source inspection, the attach handler ignores that field, so its version check should behave as before; an old phone client was not run. GUI and local CLI kind names still serialize as `"gui"` and `"cli"`. No chat wire version bump is needed for this decoding behavior. The removed Rust enum variant is an intentional source API deletion for the retired phone client; no retained caller used it. No service, saved config, Core method or local IPC implementation changed.

- Source absence regression: `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest scripts.release.tests.test_phone_absence.PhoneAbsenceTests.test_chat_wire_has_no_phone_client_kind -v` failed with the old `Remote,` variant, then the full phone absence module passed 5/5 after removal.
- Typed wire regression: with the old variant temporarily restored, `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/spaceport-t10-phone-wire-target cargo +1.99.0 test --locked -p sheprd-chat-protocol --lib retired_phone_kind_decodes_as_other -- --nocapture` failed because the old hello decoded as `Remote`. After removing it, the protocol library suite passed 68/68, including the new old-hello and retained GUI/CLI kind tests. A first run with `--exact` and the short filter selected zero tests; the corrected filter produced the red result.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/spaceport-t10-phone-wire-target cargo +1.99.0 clippy --locked -p sheprd-chat-protocol --all-targets -- -D warnings` and `cargo +1.99.0 fmt --all -- --check`: passed.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/spaceport-t10-phone-wire-target cargo +1.99.0 test --locked -p sheprd-acp --bin sheprd-acp attach::tests::`: 11 passed, covering the retained attach server's hello, command and revision behavior with its private test socket. No old phone process ran.

A31 remains `not_run`: a native process/package/network inventory and retained desktop CLI scenario are still required. RM-PHONE and T10 remain `in_progress`; this test does not claim native chat or remote Core parity.

At merged revision `8510b166`, the chat protocol library passed all 68 tests,
including old `remote` hello decoding as `Other`, and the phone source absence
module passed all five tests. The default desktop app check passed with only
Cargo's existing `block v0.1.6` future-compatibility notice. This is a merged
macOS code check; the native A31 and remote chat gates stay open.
