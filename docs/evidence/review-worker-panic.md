# Connection worker panic recovery

Verified on macOS with Rust 1.99.0 on 2026-11-17.
This covers review finding `t45y`, under `9s44`.

## Failure and repair

A connector panic closed the event channel without delivering `Disconnected`.
The worker now owns a drop guard that delivers its terminal event.
Delivery still uses the bounded, cancellable event queue.
The guard marks the handle stopped after delivery.
Cancelled connections still omit terminal events.

The event reader now owns its terminal transition.
Its drop guard cancels the worker and fails pending mailbox requests.
It preserves an existing disconnect reason and explicit shutdown advice.
It publishes final state before marking the reader drained.
The guard is constructed inside the background closure.
No UI path joins a worker or waits for an inbox lock.

Poison recovery replaces partially applied live state with disconnected state.
It cancels audio and preserves the explicit server shutdown marker.
Terminal state rejects queued welcomes from the failed generation.
Polling recovers poison through a nonblocking lock attempt.
Retired readers retain their original inboxes and completion flags.

## Regression evidence

The connector regression failed before the fix with `Err(Disconnected)` from the event receiver.
It now receives the terminal event before channel closure.

Three GUI regressions failed before their fixes:

- Reader EOF left the connection `Connected`.
- A poisoned inbox hid its update from polling.
- Reader unwind never marked the reader drained.

A further regression proved poison recovery erased explicit shutdown advice.
The repaired recovery preserves that marker.

The tests also cover cancelled setup, pending answers, lock contention, and inbox replacement.
A reader unwind test keeps its connector live until the reader cancels it.
A paired socket test exercises real handshake, snapshot, shutdown, and disconnect ordering.
These tests never discover or start a personal daemon.

## Verification

Cargo commands used the shared build lock and `CARGO_INCREMENTAL=0`.
They used the shared `upstream-fixes/target` directory.

| Command | Result |
| --- | --- |
| `cargo test -p sheprd-client --lib worker_panic --locked` | 2 passed |
| `cargo test -p sheprd-client --lib --locked` | 388 passed; 2 ignored |
| `cargo test -p sheprd-gpui --bin spaceport connection:: --locked` | 28 passed |
| `cargo clippy -p sheprd-client --lib --locked -- -D warnings` | Passed |
| `cargo clippy -p sheprd-gpui --bin spaceport --locked -- -D warnings` | Passed |
| `rustfmt --edition 2024 --config skip_children=true --check` on changed Rust files | Passed |
| `git diff --check` | Passed |
| `just check-file-size` | Passed |
| `just check-sorted` | Passed |

The GUI test linker repeated the existing large unwind-table warning.
Cargo also reported the existing future-compatibility notice for `block 0.1.6`.
Independent review inspected the guards, poison reset, callers, and retry ordering.
It found no material issues.

Full workspace gates, all-feature coverage, and platform CI remain deferred to CI.
No native desktop check ran for this state and transport change.
The task made no commits, pushes, PR changes, or Kata writes.
