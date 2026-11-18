# Protocol edges

Date: 2026-11-17. Scope: `nz6w` under `9s44`.

Status decoding now checks the control protocol before decoding the status payload.
A newer status enum cannot hide version guidance behind a schema error.
The bounded socket reader still rejects malformed or oversized input.
The existing unsupported error and CLI exit code remain unchanged.
The protocol check moved from the CLI formatter into the transport decoder.

Terminal hello negotiation now retries at the advertised version once.
A second refusal returns its typed error and bounded reason.
The real peer regression previously attempted a third connection and lost that refusal.
All twelve terminal attach tests passed after the fix.

Unix control sockets now retain a stable lock file throughout their serving lifetime.
The lock serializes stale-address recovery and survives socket cleanup.
Cleanup carries the original socket's device and inode.
It removes that socket only when its identity still matches.
The app captures this cleanup handle and releases it on the background executor during quit.
Windows retains its native handle-based cleanup behavior.

Real Unix socket tests reproduced deletion of a replacement and ignored startup locks before the fix.
All four ownership tests now pass, including eight concurrent starters on one stale address.
The full control-protocol suite passed all 80 tests.
Windows cross-target Clippy passed for client and control-protocol crates, including all targets and features.
Native Clippy passed for both crates with the same target and feature coverage.
Native Windows execution remains pending CI.
Independent concurrency review found no material defects.
The ownership guarantee covers cooperating app starters within the private state directory.
It does not claim an atomic filesystem comparison against arbitrary external socket replacement.

The reported Core late-answer leak was already fixed in base commit `69eda3b53`.
Root inspected the retained cleanup and timeout ownership code.
All ten Core connection lifecycle tests passed, with one opt-in fixture ignored.
No new Core client change was needed.

Logs:

- `/tmp/spaceport-9s44-status-version-{red,green}.log`
- `/tmp/spaceport-9s44-terminal-hello-{red,green}.log`
- `/tmp/spaceport-9s44-socket-ownership-{red,green}.log`
- `/tmp/spaceport-9s44-protocol-edges-suite.log`
- `/tmp/spaceport-9s44-protocol-edges-windows-clippy.log`
- `/tmp/spaceport-9s44-protocol-edges-native-clippy.log`
- `/tmp/spaceport-9s44-core-call-lifecycle.log`
