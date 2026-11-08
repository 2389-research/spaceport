# T04/RM-CLOUD: Container stop and destroy cut

Based on `69eb27a5` and the T04 context packet. T04 remains blocked on G-LINK, A30 stays `not_run`, and RM-CLOUD is incomplete. No real engine, saved device file, user config, or Core state was read or changed.

## Removed reachability

- Normal `app.rs` startup no longer installs `cloud.rs`'s repeating Container idle check. `container/idle.rs` and its agent-status/stop worker are gone.
- Settings' Container card no longer offers Stop or an auto-stop preference. It still displays a saved row's observed engine state and offers exact-row Forget. The old `[container] auto_stop` key remains parsed and preserved on an engine-setting save, but does not affect `Settings::resolve` or start a timer.
- The provider registry no longer offers machine deletion. The host menu's Remove path now calls only `cloud::remove(provider, id)` for the exact saved row. The delete/unsynced scan dialog, provider destroy helper, engine stop/delete commands, and their orphaned messages/tests are gone. Removing a saved row does not stop or delete its container or volume.
- The old per-device stop-intent state and UI `stopped` endpoint state are gone. The saved connector keeps a per-account/ID start lock so concurrent views do not race an engine start.

## Retained graph and next cut

`sheprd-client::ConnectTarget::Cloud` and `CloudProvider::Container` remain active. `connection.rs` still routes saved Container targets through `cloud::connect` and `container/connect.rs`, which may start a stopped container and invoke the legacy Herdr bridge. `endpoint/catalog.rs` still loads known Container rows from `cloud-devices.json`; Settings still reads the engine for saved-row status and exposes exact-row Forget. The default `container` feature remains enabled. The generic `cloud_setup`, command, image/catalog, Add Device provider rail and Settings cloud section also remain compiled. None of this is Core machine parity. Removing the sole provider enum variant now would strand shared cloud UI (`CloudProvider::ALL[0]`) and the legacy endpoint/catalog route; a later coherent cut must retire those routes while preserving old JSON as foreign data and Core/manual SSH targets. No container or volume was deleted for this fork cut.

Red source guard: `scripts/release/tests/test_container_stop_absence.py` failed 3/3 before the edit on the startup timer, Settings Stop/provider destroy, and host-menu deletion. The prior provisioning absence guard then failed because it required the now-removed idle module and startup timer; it now checks the retained connector and the timer's absence. The headless Settings test changed from requiring Stop to requiring its absence alongside a retained saved-row Remove control; the cloud host-menu test requires the exact forget-only note. The orphaned retired host-menu destroy test file was also removed after diff review.

## Checks

- Source absence and prior provisioning guards: 7 passed after the cut.
- `cargo +1.99.0 test -p sheprd-remote --features container --locked -- --quiet`: 151 passed, 3 opt-in live tests ignored; no engine was contacted.
- `cargo +1.99.0 test -p sheprd-gpui --all-features --locked cloud_devices -- --quiet`: 10 passed. The cloud host-menu exact saved-row test passed separately (1/1).
- `cargo +1.99.0 check -p sheprd-remote --all-targets --features container --locked` and `cargo +1.99.0 check -p sheprd-gpui --all-targets --locked` passed. The app's all-feature `--all-targets` check passed after removing a stale suspended-state caller and two stale error-message fixtures.
- `just check-i18n`, `just check-file-size`, `just check-sorted`, `cargo +1.99.0 fmt --all -- --check`, `tools/spec.py check` from the package root, and `git diff --check` passed. The package check covers schema and links, not application conformance.
- Cargo reported the pinned `block v0.1.6` future-incompat notice; the large debug GUI test binary also caused a macOS linker compact-unwind warning. No new Rust source warning was reported.

Native UI, real Container engine behavior, and cross-platform CI were not run. The retained connector still has legacy behavior and needs its own removal unit before RM-CLOUD can close.
