# Intentional Herdr shutdown replacement

This plan covers the review finding “Stop restarting a Herdr server the user stopped” (`bs4s`, under `9s44`).
Doctor Biz approved focused replacements for PR125.
Keep this replacement separate from unrelated connection and rendering work.

The source map compares main `b5cf4bf2b669888addb9fee84fb4cddccd0de064` with PR125 `ac69cfa006f7fd04ecdb80c3dad64f952343570f`.
The replacement starts from PR155's reviewed dependency stack at `39c62c96ffb3404c28caf30d620ce09feacceaf8`.
The [original receipt](https://github.com/2389-research/spaceport/blob/ac69cfa006f7fd04ecdb80c3dad64f952343570f/docs/evidence/review-herdr-shutdown.md) supplies historical evidence only.

## Required behavior

| Target or event | Required result |
| --- | --- |
| Local or ordinary named Session | Observed shutdown holds automatic restart until explicit reconnect. |
| SSH or WSL | Observed shutdown holds automatic bridge retry until explicit reconnect. |
| Explicit Socket | Keep ordinary attach-only retry. Never start a daemon. |
| Development Session | Keep its existing startup restriction. |
| Ordinary transport loss | Keep retry and backoff. |
| Parked group | Never start local Herdr. Wait for a serving primary before creating remote bridges. |
| Manual recovery | Complete handshake and snapshot recovery. Clear stopped advice after successful connection. |
| Retarget, detach, cancellation | Keep attempt ownership, generation fencing, and cancellation behavior. |

Retaining the last diagnostic during an explicit dial is allowed.
SSH and WSL retain upstream start-on-attach behavior.
The observed-shutdown guard cannot make remote attachment atomic with an external concurrent stop.

## Implementation sequence

1. Extend the public client ordering test in `crates/sheprd-client/src/tests/commands.rs`.
   Require Shutdown Message before Disconnected, with the same reason.
   Inspect actual CI failure before changing the client.
2. Forward the existing Shutdown Message in `crates/sheprd-client/src/session.rs` before returning its existing shutdown error.
   Keep event FIFO, bounded delivery, cancellation, and disconnect handling unchanged.
3. Add wire-level endpoint policy tests under `crates/sheprd-gpui/src/endpoint/lifecycle_tests/`.
   Cover stopped targets, ordinary loss, socket retry, retained advice, and successful explicit recovery.
   Add pure readiness cases in `crates/sheprd-gpui/src/endpoint/tests/retry_readiness.rs`.
   Exercise undrained, dirty, and lock-contended inbox states without timing guesses.
4. Retain shutdown in `crates/sheprd-gpui/src/state.rs` through disconnect and include it in display equality.
   Require a whole-window update before disconnect in `crates/sheprd-gpui/src/state/tests/shutdown.rs`.
   Clear it on accepted Connected.
5. Add Stopped advice in `crates/sheprd-gpui/src/endpoint/link.rs` and its phase in `endpoint/phase.rs`.
   Preserve version precedence and existing missing-install retry policy.
6. Add nonblocking retry readiness in `crates/sheprd-gpui/src/connection.rs`.
   Require the reader's drained flag and a clean inbox acquired with `try_lock`.
7. Thread the original Start permission through `daemon.rs`, `connection.rs`, and their existing callers.
   Primary endpoints use WhenMissing; parked connections use Never.
   Guard new parked bridges in `group_terminals.rs` until the primary serves.
   Test startup ownership in an isolated subprocess under `group_terminals/tests/parked_start_ownership.rs`.
8. Add focused parked-startup and recovery-card tests.
   Reuse `window/reconnecting.rs` and the existing reconnect action for the stopped card.
   Add only the required en-US endpoint and window messages.
   Check the existing CLI status consumer and reconnect command.

Use selected hunks from PR125's `150eed475` checkpoint.
Keep the existing reader implementation, stream transport, executable lookup, and dependency pins.
Exclude typed-error sweeps, pipe fixtures, caches, clipboard, notice timing, keyboard, and profiling changes.

## Safe test coordination

Use private fixture sockets and exact owned children.
Changing a fixture's target identity must not execute that target's transport.
Use raw endpoint polling to inspect Local, SSH, or WSL policy.
Repoint to an owned Socket before assertions or window event-loop progress.
Never run window polling against those identities during an unfixed baseline.

Do not copy the preserved test that waits for worker shutdown while holding the inbox lock.
Main's reader-driven waiter can block behind that same lock.
Use deterministic readiness states alongside real socket ordering.
Keep failed prerequisites distinct from the intended regression assertions.

## Qualification

Every checkpoint needs static checks, review, and actual CI evidence.
Run product tests and builds on disposable CI; local product execution remains held.
Require default and all-feature client and app results on the final source.
Verify real isolated Herdr shutdown and exact fixture cleanup with an explicitly selected binary.
Qualify the stopped card and explicit recovery at normal and narrow widths.
Keep native screenshots, real SSH/WSL recovery, and unsupported platform resources explicitly pending until exercised.
Doctor Biz's Mac foreground and audible-speech holds remain in force.
No native or release completion follows from source review or headless metadata.
