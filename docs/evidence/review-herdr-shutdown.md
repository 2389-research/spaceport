# Intentional Herdr shutdown

Review child `bs4s`, under `9s44`.

The client previously converted `ServerShutdown` directly into a disconnect error.
The endpoint lost the distinction between shutdown and transport failure.
Its automatic reconnect could then start Herdr again.

The client now forwards shutdown before its disconnect event.
The existing endpoint reducer retains shutdown advice across worker state replacement.
Local, SSH, and WSL targets require an explicit reconnect after shutdown.
Explicit sockets remain attach-only and keep their ordinary reconnect behavior.
Ordinary transport loss retains automatic reconnect.

Automatic redial waits until ordered events drain and their final inbox update is consumed.
This prevents a contended event reader from losing the shutdown fence.
Parked local connections never start Herdr.
Parked remote connections wait for their primary endpoint to serve.

SSH and WSL bridges retain upstream start-on-attach behavior.
The primary guard prevents retries after observed shutdown.
It does not provide an atomic remote attach-only operation during an external concurrent stop.

## Verification

All Cargo commands used Rust 1.99.0, `--locked`, and `CARGO_INCREMENTAL=0`.
Commands shared the lock `/tmp/spaceport-qnz9/cargo.lock` and the existing upstream-fixes target cache.

- The client shutdown-order regression failed before the fix.
- The endpoint shutdown regression failed before the fix.
- Independent review identified delayed-reader and remote-bridge gaps.
- Both added regressions failed before their corrections.
- Six shutdown and restart-card tests passed.
- The broader integration-feature endpoint filter passed 249 tests.
- The client unit suite passed 386 tests, with two existing opt-in ignores.
- A real isolated named Herdr session delivered shutdown before disconnect and exited cleanly.
- Native local-control fixtures passed in both themes and at 800px and 360px widths.

Root inspected the actual native captures.
The card, note, and explicit restart button fit both widths.
The independent reviewer found no remaining required findings after correction.

Logs use `/tmp/spaceport-9s44-` prefixes.
The named-session test selected `/Users/harper/.local/bin/herdr` explicitly.
It used the shared private sandbox and stopped only its own named session.
No personal daemon, transcript, or credential participated.

Native captures are in [review-squad/shutdown](review-squad/shutdown/).
Windows, Linux, updater-feature builds, and full workspace CI remain unrun locally.
The baseline macOS linker unwind warning and `block` future-compatibility notice remain.
