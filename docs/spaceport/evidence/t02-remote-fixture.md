# T02 private Core-to-Core machine fixture

T02 is in progress. A04 remains not run as a complete product scenario.

On macOS, the opt-in integration test started two copies of pinned Core `cd12ddd88bc11490228343291dc1096c9ed3bf77` (`0.1.18`) in distinct private homes and sockets. The local Core used Core's `HC_SSH` test hook to run a private pipe script that executed the remote Core's `connect` command. The script carried bytes; it did not use real SSH, Tailscale, credentials, or a personal daemon. The Spaceport client called the real `machine.add` RPC without an install field. Core reported the second Core in `machine.list`, ran a terminal on it, and returned a machine-qualified address that `terminal.get` and the aggregate terminal list resolved exactly once.

The test then killed the exact remote `serve` child it had created, after killing its own remote test terminal. A first assertion expected a visible down state and failed after ten seconds. Source tracing showed that `herdr-core connect` calls Core's `connect_or_start`; Core restarted its private remote server and could keep the catalog connected. The corrected test verifies the original child exited, a new Core answered at the same private socket with the same ID, and a fresh terminal launched through the local Core. This tests Core-owned recovery without adding client retry or process ownership.

`RUSTUP_TOOLCHAIN=1.99.0 CARGO_TARGET_DIR=/tmp/spaceport-t00-target just test-core /tmp/spaceport-t00-target/debug/herdr-core` passed all 3 opt-in live tests. `cargo +1.99.0 test --locked -p sheprd-core-client --lib` passed 38 active tests with 3 ignored. The test does not prove SSH host-key refusal, authentication, Tailscale routing, Windows/Linux native behavior, GUI duplicate suppression, or the final Spaceport product path. Those remain open.

## Core backend machine target

`sheprd_backend::Spawn` now has an optional saved machine name. The Core backend passes it unchanged as `terminal.run`'s `on` parameter; an absent name keeps Core's local default. A scripted-peer test failed to compile before the field existed, then checked the exact `on = "lab"` request and `lab/t6` result. The private two-Core fixture opened a second client connection to the local Core, spawned on `lab` through `CoreBackend`, received a qualified address, and killed that exact terminal. It did not start an SSH manager in the client. The real-Core suite passed three opt-in tests; the client library passed 41 active tests with three ignored. The GUI does not yet route remote selection through this field, so A04 remains not run as a complete Spaceport scenario.

## Refused machine is not saved

A separate private Core fixture set Core's test-only `HC_SSH` hook to a script that printed the standard changed-host-identification warning and exited 255. Spaceport's `machine_add` returned Core's `unreachable` refusal, and `machine.list` stayed empty. All four opt-in real-Core tests passed. This proves refusal propagation and no saved machine for that script outcome. It does not exercise OpenSSH's host-key check, an actual known-hosts file, authentication prompts, or any real remote host. A05 remains not run.
