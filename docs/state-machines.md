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

## Settings persistence — y0p7

[`persistence/lifecycle.rs`](../crates/sheprd-gpui/src/settings_window/persistence/lifecycle.rs) owns active writes, drain goals, and pending work.
`Lifecycle::transition(Event)` is the only writer of those fields.
Active `Save` states are Idle, Loading, and Running(Kind).
Running kinds capture Ordinary, Theme(revision), Layout(mode), or Sidebar(intent).
The orthogonal `Goal` states are Editing, Flushing, Closing(window), and Quitting.

| Events | Transition or effect |
| --- | --- |
| BeginLoad / FinishLoad | Idle to Loading and back; resume accepted work |
| BeginSave / FinishSave | Capture the exact running intent, then return it for reconciliation |
| Queue / Next / TakePending | Accept at most 64 waiting ordinary edits; drain FIFO or transfer ownership to shutdown |
| Flush / StopFlush | Enter or leave automatic draft draining |
| Close / TakeClose | Drain accepted work before removing the captured window |
| Quit | Preserve the running kind while transferring pending writes to shutdown |
| QueueSidebar / TakeSidebar | Keep the committed sidebar intent separate from later previews |
| AdvanceShared | Promote matching queued snapshots through an exact owned successful write |
| AbortDrain | Stop a failed theme resolution without dropping an accepted sidebar commit |
| SaveFailed | Stop automatic draft draining and preserve live drafts for explicit retry |

Theme and layout completion clear only the captured intent.
A newer draft therefore survives an earlier save or quit.
Queue saturation refuses the newest request with a localized error.
A later queued success cannot erase that failure notice.
The managed-skill action retains its existing refusal while busy.

[`persistence/operation.rs`](../crates/sheprd-gpui/src/settings_window/persistence/operation.rs) retains typed shared edits until dispatch.
A successful guarded write carries its exact before and after snapshots.
Reconciliation must match that after snapshot before promoting requests with the matching before snapshot.
Normal and quit drains use the same promotion rule.
External writes remain conflicts, including changes after reconciliation reads.

### Verification

- [Save intent tests](../crates/sheprd-gpui/src/settings_window/tests/save_intents.rs) cover theme, layout, sidebar, queue, and shutdown races.
- [Real-file queue tests](../crates/sheprd-gpui/src/settings_window/tests/shared_save_queue.rs) cover owned write chains and external conflicts.
- `cargo test --locked -p sheprd-gpui --bin spaceport settings_window::`: 241 passed.
- `cargo test --locked -p sheprd-gpui --bin spaceport herdr_settings::`: 48 passed.
- Default and integration-feature Clippy, formatting, size, sorting, i18n, and whitespace checks passed.

Native controls and cross-platform execution remain separate checks.
