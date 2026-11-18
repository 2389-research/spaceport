# Core command backpressure

Date: 2026-11-17. Scope: `38y7` under `9s44`.

## Acceptance and changes

Core input rejection must reach the view instead of only a log.
Rejected size and focus changes must remain pending.
They must retry when capacity returns, even without terminal output.
Accepted commands must retain FIFO ordering and bounded queue storage.
Input must never replay automatically.

`Link::send` now reports queue acceptance.
The view records size and focus only after acceptance.
Its fields explicitly name queued state instead of implying runtime acknowledgement.
Rejected input reaches the screen mailbox as a visible status.
Disconnected workers also publish a visible status without replacing a more specific existing failure.
The rejection path does not log the rejected command's private input text.

The first full-queue rejection wakes the view once.
A later dequeue wakes it again when capacity returns.
The first wake covers a drain racing the blocked-flag assignment.
Repeated rejection while blocked does not continuously invalidate the view.
Worker drop retains its existing stop flag and never joins on the UI thread.

## Verification

Three actual baseline regressions failed before the change.
A rejected resize was recorded as sent.
Saturated input produced no visible failure.
Draining a saturated queue produced no no-output retry wakeup.

The focused Core suite passed 138 tests; eight explicit-runtime tests remained ignored.
Independent concurrency review inspected send/dequeue races, mailbox ownership, UI polling, render retries, FIFO ordering, and drop paths.
No material findings remained.
The final consumer assertion passed: the Core view consumes and displays a rejected-command status.

Logs use `/tmp/spaceport-9s44-core-{resize,input,retry}-red.log` and `core-backpressure-green.log`.
Cargo used the pinned toolchain, shared task lock and target, and `--locked`.
The existing compact-unwind and `block` future-incompatibility notices remain.
Native Core runtime presentation and platform CI were not run for this change.
