# State machines

This registry tracks lifecycle conversions delivered under Kata qnz9.
Each entry names its state owner, transitions, side effects, and regression tests.
Keep lifecycle writes inside the named transition function.
Keep wire and saved representations separate from these private states.

The [implementation plan](plans/2026-11-17-explicit-state-machines.md) lists all 15 conversions and their dependencies.
The [earlier SIFT progress](sift-audit-2026-11-13-progress.md) records preceding conversions.

## Core terminal spawning — 1w6g

[`core_terminals/spawning.rs`](../crates/sheprd-gpui/src/core_terminals/spawning.rs) owns `Spawning`.
Its states are Idle, Queued(FIFO), and Running(purpose, receiver, FIFO).
Only `Spawning::transition(Event)` mutates the state or queue.

| Event | Transition or effect |
| --- | --- |
| Queue | Retain the complete request and captured Core identity; refuse the newest when 64 requests wait |
| Start | Remove the oldest request; validate its Core and purpose; submit one worker |
| Poll | Consume a completed or disconnected worker once; preserve its waiting successors |
| Unavailable | Clear waiting requests while retaining an already submitted worker |

Completion placement always attempts to start the next request.
Success, placement refusal, and errors therefore cannot strand the FIFO.
The worker retains its original backend and performs blocking work off the UI thread.
A disconnected receiver has an uncertain outcome and never requeues its request.
A stale Core request is refused before submission.
Core terminals remain runtime-owned when a view disappears.

### Verification

- [Queue integration tests](../crates/sheprd-gpui/src/core_terminals/tests/spawn_queue.rs) observe framed requests and post-placement draining.
- [Owner tests](../crates/sheprd-gpui/src/core_terminals/spawning/tests.rs) cover bounds, Core mismatch, unavailable cleanup, and uncertain completion.
- `cargo test --locked -p sheprd-gpui --features integration-test --bin spaceport core_terminals::`: 114 passed, eight opt-in tests ignored.
- Seven chat Core tests and applicable quick checks passed.

Native GUI and isolated real-runtime spawning remain separate integration checks.

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

## Herdr activity reporting — dre7

[`herdr/activity.rs`](../crates/sheprd-acp/src/herdr/activity.rs) owns `ActivityReport`.
Its states are Unreported, QueuedInitial, Queued, Reporting, Retry, and Acknowledged.
Only `ActivityReport::transition(Event)` mutates that bounded mailbox lifecycle.

| Event | Transition or effect |
| --- | --- |
| Queue | Keep the latest activity; preserve one newer value during an in-flight report |
| Begin | Move queued or retry activity into Reporting for the single worker |
| Complete | Keep newer queued work; otherwise acknowledge success or retain Retry on failure |

The worker performs socket I/O outside the mailbox lock.
A failed report waits on the existing Condvar retry interval.
Acknowledgement includes the existing authority check and reclaim sequence.
A newer activity supersedes the older value, whether its in-flight report succeeds or fails.
An attempted report may have reached Herdr despite a failed reply.
The attempted-state projection therefore preserves announcement and withdrawal behavior.
Closing wakes the worker, skips stale retries, and retains the prior handling of already queued reports.

### Verification

- [Worker tests](../crates/sheprd-acp/src/herdr/tests/activity.rs) use an isolated socket peer for failed submission, authority checks, supersession, and shutdown.
- [Portable owner tests](../crates/sheprd-acp/src/herdr/tests/activity_state.rs) cover acknowledgement and latest-value retention.
- Two retry regressions failed before the conversion.
- `cargo test --locked -p sheprd-acp`: 411 unit, one branding, and four process tests passed.
- Scoped Clippy, formatting, size, sorting, and whitespace checks passed.

Live-Herdr and other-platform execution remain separate integration checks.

## Agent session opening — ha8y

Codex and Pi each own opening state beneath their existing bridge.
The bridge translates effects into existing harness messages and ACP replies.
Turn ownership and setting-command state stay in their existing owners.

| Provider | Owner | Stored states | Transition function |
| --- | --- | --- | --- |
| Codex | [`codex/bridge/opening.rs`](../crates/sheprd-acp/src/codex/bridge/opening.rs) | Handshake: Waiting, Ready, Failed. Models: NotAsked, Waiting, Ready. Opening: None, Asked, Thread, Open. | `opening::State::transition(Event)` |
| Pi | [`pi/bridge/opening.rs`](../crates/sheprd-acp/src/pi/bridge/opening.rs) | Handshake: Waiting, Ready, Failed. Opening: None, Asked, Open. | `opening::State::transition(Event)` |

### Codex transitions

- `Ask` retains the ACP request until initialization completes.
- A second pending opening fails without replacing the first request.
- Successful initialization requests models once and starts or resumes the requested thread.
- Failed initialization persists and answers both pending and later opening requests.
- A valid thread waits for model discovery before becoming Open.
- Failed model discovery preserves the existing optional-model behavior.
- Thread failure or an empty thread identity fails the pending opening.
- A completed opening permits another opening.
- Pending and refused replacements retain the active session identity.
- Only a validated successful replacement changes prompt, cancellation, and notification routing.

### Pi transitions

- State and model discovery may finish in either order.
- A discovery failure persists and answers pending and later opening requests.
- Later discovery success cannot replace a recorded failure.
- Both successful discovery results permit resolving the pending ACP opening.
- The requested loaded identity takes precedence over the discovered identity.
- An empty identity fails instead of creating an Open session.
- A second pending request cannot replace the first opening or its loaded identity.
- Later openings preserve current model and thinking settings.

### Verification

The tests exercise real bridge inputs and emitted harness/ACP messages with synthetic protocol data.
The existing helper checks also run isolated executable processes.
Live provider sessions remain a separate integration check.

- [Codex opening tests](../crates/sheprd-acp/src/codex/bridge/tests/opening.rs).
- [Codex reopening tests](../crates/sheprd-acp/src/codex/bridge/tests/reopening.rs).
- [Pi opening tests](../crates/sheprd-acp/src/pi/bridge/tests/opening.rs).
- `cargo test --locked -p sheprd-acp bridge::tests::opening`.
- `cargo test --locked -p sheprd-acp --all-features`.

The original regression run failed six cases before the conversion.
Sequential opening and retained-setting regressions guard behavior preserved during the conversion.

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

## Chat tool liveness — ed3g

[`Transcript::tool_live`](../crates/sheprd-chat-protocol/src/model.rs) owns the shared liveness projection.
The stored enums remain Status, ToolStatus, and ActivityState.
`Transcript::apply` remains their shared transition function; no wire state was added.
The projection is pure and never rewrites a tool's reported result.

| Observed state | Liveness |
| --- | --- |
| Failed or exited session | Stopped |
| Direct running activity | Live across turn boundaries |
| Direct ended activity | Stopped |
| Completed or failed ordinary tool | Stopped |
| Unfinished child with a known parent activity | Follow that parent's activity |
| Unfinished ordinary tool in the current working turn | Live |
| Idle session, older turn, or unknown current membership | Stopped |

GPUI cards, Terminal and Timeline markers, expanded child steps, and the working list use that projection.
A private ToolState maps it to consistent labels and colors.
The TUI line cache tracks liveness alongside wrapped content.
Status-only updates and turn boundaries therefore refresh cached progress marks.
Completed children do not become live merely because their parent still runs.

### Verification

- [Protocol tests](../crates/sheprd-chat-protocol/src/tests/tool_liveness.rs) cover session, turn, and parent-activity ownership.
- [TUI tests](../crates/sheprd-acp/src/tui/view/tests/tool_liveness.rs) reduce actual attach messages through the line cache.
- [Paint tests](../crates/sheprd-gpui/src/chat/tests/tool_liveness.rs) inspect the actual Timeline marker.
- Two TUI regressions, a marker regression, and a parent-activity regression failed before their corrections.
- Clean-member checks passed 71 protocol, 406 ACP unit, five process/branding, and 482 chat tests.
- Scoped all-target Clippy and repository quick checks passed.
- [Native evidence](evidence/tool-liveness.md) includes 24 private before/after preview windows in light/dark and wide/narrow layouts.

Live-provider event delivery remains a separate integration check.

## Endpoint links — 77dt

[`endpoint/link.rs`](../crates/sheprd-gpui/src/endpoint/link.rs) stores Disabled, Detached, Waiting, Dialing, and Up.
Retry deadlines and connection stability time belong to those states.
Failure distinguishes Transport, Missing, and Version advice.
`Endpoint::enter(Transition)` owns all link, attempt, bridge, and handle lifecycle writes.

| Transition | Effect |
| --- | --- |
| Stop / Detach | Retire the current transport and preserve the appropriate retry advice |
| Retarget | Replace the exact target and clear the old target's diagnostic state |
| Connect | Start the eligible transport attempt |
| Observe / Update | Project current worker facts into stored link state |
| Lost | Record failure and retry policy without reusing an old inbox |
| RetryAt / ResetAttempts | Own retry scheduling and stability accounting |
| Trade | Transfer the bridge and observed state together |

LiveState remains the independently reduced worker projection.
Stored link phase readers use accessors rather than reconstructing flags.
Initial surface registration and activation receipts keep their separate owners.
A stopped handle cannot complete the stability interval from a staged connected snapshot.
Fresh attempts without an outage reason do not inherit a reconnecting label.

### Verification

- [Catalog tests](../crates/sheprd-gpui/src/endpoint/tests/catalog_link.rs) pin diagnostic reset on retargeting.
- [Real-handle tests](../crates/sheprd-gpui/src/endpoint/lifecycle_tests/catalog_link.rs) fence retired inboxes.
- [Stability tests](../crates/sheprd-gpui/src/endpoint/lifecycle_tests/link_stability.rs) reject stopped-handle success.
- Clean default checks passed 4,007 tests; integration-feature endpoint checks passed 239.
- Scoped Clippy, formatting, size, sorting, i18n, whitespace, and independent lifecycle review passed.

Native catalog interactions and other-platform execution remain separate checks.

## Core call and stream ownership — 674p

[`connection.rs`](../crates/sheprd-core-client/src/connection.rs) stores each Call as Registered, Sent, Answered, or Abandoned.
`Shared::transition(CallEvent)` owns every production pending-call and loss-fence write.
Narrow wrappers dispatch events and return typed outcomes.

| Event | Transition or effect |
| --- | --- |
| Register | Refuse lost/full connections; install a bounded reply owner |
| Sent | Mark accepted outbound work without overwriting an earlier answer |
| CancelUnsent | Remove work that was never accepted |
| Finish | Claim an answer or retain an Abandoned tombstone after timeout |
| Deliver | Hand off the reply atomically or return its unclaimed route to the worker |
| Lose | Fence later registrations, fail waiters, and end routes |

Answers and abandoned tombstones both count toward MAX_IN_FLIGHT.
A late answer frees its tombstone and closes any unclaimed stream channel.
A full or disconnected reply waiter also returns channel ownership to the worker.
The worker sends those CLOSE frames directly, independent of outbound queue saturation.
The timeout/answer lock handoff claims an already delivered reply instead of leaking its route.
No timed-out request is replayed.

[`Stream::drop`](../crates/sheprd-core-client/src/stream.rs) owns ordinary attachment/subscription cleanup once.
A stream exists before successful reply metadata is parsed, so typed parse errors also release ownership.
Ordinary stream destruction retains the existing bounded best-effort queue policy.
That policy is distinct from direct worker cleanup for abandoned or unheard replies.

### Verification

- [Abandoned-call tests](../crates/sheprd-core-client/src/connection/tests/abandoned.rs) cover late answers and unclaimed routes.
- [Owner tests](../crates/sheprd-core-client/src/connection/tests/lifecycle.rs) cover bounds, loss fencing, and handoff races.
- [Isolated Core test](../crates/sheprd-core-client/src/connection/tests/live.rs) delays actual Core replies and observes CLOSE writes.
- Clean-member default and all-feature suites each passed 77 tests; opt-in tests remained ignored.
- The selected hc0.1.20 fixture passed and confirmed the terminal remained Live.
- All-target/all-feature Clippy, repository quick checks, and independent re-reviews passed.

The real-Core fixture does not inspect internal viewer accounting.
Other-platform execution remains a separate check.

## Click tunnel ownership — t9fk

[`listening_ports/tunnel.rs`](../crates/sheprd-gpui/src/listening_ports/tunnel.rs) stores Idle, Opening, Open, and Reopening per key.
`Tunnels::transition` owns every entry, state, and remembered-port write.
Begin starts one attempt; Finish and Fail accept only that exact Arc identity.
Close cancels activity while retaining port history; Forget also removes that history.
Prune drops dead children, and close_all/Drop dispatch Forget for each owned entry.
Reopening preserves an existing tunnel until its replacement succeeds.
Stale successes drop and reap their own child without consuming a replacement.
Open tunnels and pending attempts share the existing capacity bound.

### Verification

- [Attempt tests](../crates/sheprd-gpui/src/listening_ports/tests/tunnel_attempts.rs) cover all cancellation paths and both completion orders.
- Two stale-completion regressions failed before the correction.
- Clean listening-port checks passed 45 tests; scoped Clippy and quick checks passed.
- Independent review checked exact children, retained port preferences, bounds, and cleanup.

Real SSH and native browser interactions remain separate checks.
The host-menu duplicate-forward gap remains outside this conversion.

## Core connection start intent — fj07

[`core_terminals/hub.rs`](../crates/sheprd-gpui/src/core_terminals/hub.rs) stores Idle, Connecting, Ready, or Unavailable.
Connecting owns its receiver, start mode, and that attempt's explicit upgrade request.
Unavailable owns its typed error and retry timestamp.
`Hub::transition` owns state and exact-binding writes.
Connect starts or upgrades only the current attempt; Poll consumes its outcome.
Successful joins discard pending start permission.
Failed joins may consume their own explicit upgrade once.
Thread-creation failures enter the same retry delay as other failures.
BindExact suppresses startup and clears upgrades; ClearExact releases that view binding.
Known Core identity is checked again when the result arrives.

### Verification

- [Intent tests](../crates/sheprd-gpui/src/core_terminals/hub/tests/start_intent.rs) cover stale start, worker failure, dropped workers, retry expiry, and latest binding.
- Two original regressions failed before conversion.
- Clean default Core checks passed 189 tests; integration-feature checks passed 192.
- Clippy, quick checks, and independent lifecycle review passed.

Real installed-runtime startup, native GUI, and platform execution remain separate checks.
