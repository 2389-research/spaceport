# State machines

This registry tracks lifecycle conversions delivered under Kata qnz9.
Each entry names its state owner, transitions, side effects, and regression tests.
Keep lifecycle writes inside the named transition function.
Keep wire and saved representations separate from these private states.

The [implementation plan](plans/2026-11-17-explicit-state-machines.md) lists all 15 conversions and their dependencies.
The [earlier SIFT progress](sift-audit-2026-11-13-progress.md) records preceding conversions.

## Chat file work — xtjm

[`chat/files.rs`](../crates/sheprd-gpui/src/chat/files.rs) owns each pane's unfinished file operations.
`FileWork` holds a private `State` enum with `Idle` and `Pending(operations)` variants.
Only `FileWork::transition(Event)` writes that state, its operation list, or cancellation signals.

| Event | Transition | Effect |
| --- | --- | --- |
| Begin | Idle becomes Pending; Pending gains the operation | Cancel stale work, then retain the exact new ticket |
| Synchronize | Keep operations for the current target and transcript epoch | Cancel removed operations; return to Idle when none remain |
| Finish | Remove the exact current ticket once | Permit its completion without debiting another operation |
| Cancel | Pending becomes Idle | Signal cancellation for every unfinished operation |

The ticket captures its target, transcript cursor, file count, and cancellation identity.
Sequence advancement within the same transcript epoch preserves the ticket.
Target replacement or a new epoch invalidates it.
Pending file counts derive from current operations instead of an independent counter.
Dialogs use zero-count tickets so their later callbacks have the same ownership fence.

Pane destruction dispatches Cancel through the transition owner.
The owner wraps the private enum so replacing state cannot recursively run its destructor.
Unclaimed SSH upload results use background cleanup against their captured original host.
Cleanup remains best effort under the existing transfer API and application lifetime.

### Verification

- [Pure owner tests](../crates/sheprd-gpui/src/chat/files/tests.rs) cover counts, exact completion, invalidation, and cancellation.
- [Upload and dialog tests](../crates/sheprd-gpui/src/chat/tests/uploads.rs) exercise actual headless callbacks and completion handling.
- `cargo test --locked -p sheprd-gpui --bin spaceport chat::` passed 492 tests.
- Three regression tests failed before the conversion.

Live SSH cleanup and native window teardown remain separate integration checks.
