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
