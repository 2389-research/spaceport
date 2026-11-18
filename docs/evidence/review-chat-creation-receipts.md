<!-- ABOUTME: Records acknowledged chat creation receipts and their verification. -->
<!-- ABOUTME: Separates runtime creation from ACP attachment and agent readiness. -->

# Chat creation receipts

Verified on 2026-11-17 for the chat creation report (`j2p2`).
Doctor Biz authorized implementation and validation in the aggregate worktree.
This work does not commit, publish, or update the issue tracker.

## Acceptance

`chat new` must return the created tab and pane without comparing chat lists.
The response must wait for runtime acknowledgement.
Core must also place the acknowledged terminal in the requested native tab.
Named windows and explicit panes must retain their exact targets.
Waiting, cancellation, and lost answers must never replay a creation request.

## Contract

Control protocol 38 replaces `Started` with `Created`.
The reply contains `agent`, `window_id`, a tagged `tab`, and `pane_id`.
Herdr tab IDs remain daemon strings.
Core tab IDs remain native integers, with a separate runtime terminal string.
The CLI prints `created: true` and these identities in JSON.
Creation does not promise ACP attachment or agent readiness.

Herdr reports after `tab.create` acknowledgement and launch input acceptance.
Core reports after its worker result reaches native tab or split placement.
Both paths use bounded one-slot receipts.
Core retains receipt ownership beside its existing bounded FIFO and tab reservation.
Dropping a receipt does not replay or flush the runtime request.
The existing exact Core identity fence remains in place.

`chat new` declares a 30-second wait in the shared request contract.
The socket admission limit and parked-request bound apply before creation starts.
The existing Herdr creation deadline remains 15 seconds.
Timeout, stale selection, and lost receipts expose uncertain outcomes.
The caller must inspect state before retrying.
Explicit missing panes and mismatched selected sessions refuse before effects.

## Regression evidence

The wait contract test failed with zero seconds before the change.
Log: `/tmp/spaceport-qnz9/cli-chat-wait-red.log`.
The real socket regression observed an immediate reply before daemon acknowledgement.
It failed with zero parked requests instead of one.
Log: `/tmp/spaceport-qnz9/cli-chat-creation-red-fixed-fixture.log`.
An earlier fixture called a private child method and failed compilation.
The corrected fixture uses the existing window chat polling method.

The real socket regression now returns the acknowledged `w1:t9` and `w1:p9`.
Focused chat controls passed five tests before the final target/error tests were added.
Log: `/tmp/spaceport-qnz9/cli-chat-green.log`.
Core spawn and receipt tests passed all twelve tests.
They cover actual private Core frames, native placement, refusal, and dropped receivers.
Log: `/tmp/spaceport-qnz9/cli-chat-core-green.log`.

The final protocol, control, and executable CLI batch passed.
Log: `/tmp/spaceport-qnz9/cli-chat-final-green.log`.
Protocol tests passed all 76 before the integrating owner's socket ownership regressions were added.
GPUI control tests passed all 131.
Executable CLI tests passed 43, with one existing producer fixture ignored.
The compiled CLI covers both receipt ID namespaces and checks that no request retries.
All-target, all-feature GPUI Clippy passed with the pinned Sparkle SDK.
The repository SDK script verified its archive, extracted bytes, license, and architectures.
Log: `/tmp/spaceport-qnz9/cli-chat-clippy-all.log`.
Independent review found a missing Core timeout classification after that pass.
The typed receipt regression failed with `Failed` instead of `Timeout`.
Log: `/tmp/spaceport-qnz9/cli-chat-core-errors-red.log`.
The mapper now classifies Core timeouts, lost answers, and invalid post-submission replies.
It preserves explicit refusals, worker launch errors, and unplaced terminal diagnostics.
Reviewed chat controls passed all nine tests.
All-target, all-feature Clippy passed again after the error mapping fix.
Log: `/tmp/spaceport-qnz9/cli-chat-reviewed-green.log`.
The integrating owner independently approved the final mapping and lifecycle path.
That review covered admission before effects, placement, FIFO ownership, and uncertain outcomes.
No material findings remain in the receipt change.
Quick file-size, sorted-region, formatting, and diff checks passed before that batch.

Full CI and native keyboard/performance checks remain deferred.
Scripted peers and headless windows do not prove native agent startup.
The existing macOS compact-unwind linker warning and `block` future-incompatibility notice remain.
