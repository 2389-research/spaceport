# Child output deadlines

Date: 2026-11-17. Scope: `fn64` under `9s44`.

SSH probes and device commands shared an unbounded reader join after child exit.
An inherited pipe writer could keep that join waiting forever.
Both paths now use the same cancellable, bounded pipe capture.
Unix readers use nonblocking pipes. Windows readers check available bytes before reading.
Each capture owns its descriptor, cancellation flag, and reader thread.
Dropping a capture closes its reader and joins it off the UI thread.
It does not wait for another process to close its writer.

Child waiting and output collection share one deadline.
SSH cancellation also interrupts output collection.
Reader failures retain their I/O source.
Device commands retain bounded stdout prefixes and stderr tails.
The stderr tail preserves the final error after progress output.
The device child guard also reaps the child when reader startup fails.

Real kernel-pipe regressions failed with retained writers before the fix.
A follow-up regression caught the first channel-only fix retaining its blocked reader.
The final tests prove deadline handling, cancellation, descriptor release, retention limits, and I/O failures.
An isolated executable fixture checks the actual device command's final stderr line.
It uses private HOME and XDG paths.

The client library passed 409 tests, with four opt-in tests ignored.
Native and Windows cross-target client Clippy checks passed for all targets and features.
The retained GPUI Clippy log did not establish all-feature coverage.
A fresh all-target, all-feature check then passed with the validated pinned Sparkle SDK.
That SDK is `/private/tmp/spaceport-cli-sdk.7S7nty/sdk`.
Its acquisition and validation record is `/tmp/spaceport-qnz9/cli-chat-sdk-validated.txt`.
The final device output run passed three tests after centralizing the shared test sandbox.
Its ignored child fixture ran explicitly inside the parent test.
Independent lifecycle and concurrency review found no material defects.
Native Windows execution remains pending CI.

Logs:

- `/tmp/spaceport-9s44-probe-reader-cleanup-red.log`
- `/tmp/spaceport-9s44-probe-capture-green.log`
- `/tmp/spaceport-9s44-device-stderr-red.log`
- `/tmp/spaceport-9s44-device-output-isolated.log`
- `/tmp/spaceport-9s44-device-output-central-sandbox.log`
- `/tmp/spaceport-9s44-pipe-client-suite.log`
- `/tmp/spaceport-9s44-pipe-capture-native-clippy.log`
- `/tmp/spaceport-9s44-pipe-capture-windows-clippy.log`
- `/tmp/spaceport-9s44-pipe-gpui-clippy.log`
- `/tmp/spaceport-qnz9/cli-chat-clippy-all.log`
