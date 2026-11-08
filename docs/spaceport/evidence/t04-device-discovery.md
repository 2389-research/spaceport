# T04 Add Device discovery removal

This cut removes Add Device suggestions from Bonjour `_ssh._tcp`, local `tailscale status --json`, and `~/.ssh/config` aliases. It does not alter phone Bluetooth pairing, the Core machine picker, or the manual Add Device form. T04 remains `in_progress`; A04 and A30 remain `not_run`.

## Ownership and reachability

Before removal, opening the Add Device form started `menu/devices/discover.rs`, whose source modules read the network, Tailscale client and SSH config on background workers. The form displayed a Nearby devices list, completed typed targets, and accepted arrow/Tab selection; the rail showed an unsaved suggestion count. Its own GPUI tests, English Fluent catalog, error variants, README section, and GPUI `mdns-sd`/`if-addrs` dependency edges served that path.

The discovery module and its tests, form and rail entry points, dedicated messages/errors, docs and dependency edges are removed. The remaining `mdns-sd`/`if-addrs` workspace dependencies serve other crates, including the separate phone server; this cut does not claim those components are removed. `menu/devices/local_network.rs` remains because a manually entered SSH target can still need macOS Local Network permission. `menu/devices/core_catalog.rs` remains the Core machine display path. The manual form still accepts a target, label and session and keeps its existing host checks; T03 must later route machine setup through Core rather than the legacy Herdr 1 path.

The combined fork compile exposed one newly unused `SearchInput::set_completion` method: discovery had been its sole production caller. The completion field, ghost-text paint branch and tests for that now unreachable behavior were removed. `multiline_input` has a separate active chat completion path and remains in use. This closes the warning at its source without adding an allowance.

## Checks and limits

The new headless test `add_device_keeps_manual_target_and_has_no_discovery` first failed because the discovery panel rendered. After deletion it passed, with a typed target and label retained. The existing Core machine picker, manual form `en-XA` translation, and local network permission tests also passed using the same built test binary. `just format-check`, `just check-file-size`, `just check-sorted`, `just check-i18n`, and `git diff --check` passed. The package render/check and spec-tool tests validate the evidence package only.

After integration, `cargo +1.99.0 test --locked -p sheprd-gpui --bin sheprd search_input::` passed 25 tests and `core_terminals::` passed 65 tests with 3 private Core tests ignored. The first combined build failed because two new T03 test refusals lacked T02's `hint` field; both fixtures now use `hint: None`, and the rerun passed. `cargo +1.99.0 check --locked -p sheprd-gpui --bin sheprd` passed without the search completion warning. It still reports one warning for the T03 Archive/Delete/Rename UI variants that remain unwired; the test link prints the existing macOS compact-unwind warning and Cargo reports `block` future compatibility.

No native desktop QA, Windows/Linux build, full workspace suite, real daemon, Tailscale client, or personal SSH configuration was used. The old manual host-save path and other excluded discovery features elsewhere in the app need separate T03/T04 work. A04 remains open because this cut does not prove Core-owned machine add/reconnect.
