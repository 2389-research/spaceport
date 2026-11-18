# Image-paste drain: the four hung waits are a parked-refactor defect, not a main bug

Worktree: `/Users/clint/code/spaceport/.worktrees/image-paste-drain`
Base: `50b9063be` (merge of #93 on main). Working tree clean, no commits, **no repo files changed**.
Scope: section 5 of the remote-drop worker's report
(`…/02688418-57ec-407a-96ed-f1cb4ed2cfa6/docs/notes-flake-fix.md`), which parked the
clock-free refactor of `lifecycle_tests.rs` because four image/file-paste tests
seemed to hang under it.

Status at a glance:

- **Plain main is green.** 12/12 runs of the whole `endpoint::lifecycle_tests::`
  filter, 15/15 runs of the four named tests in one invocation, 40/40 runs of each
  named test alone, and 10/10 + 5/5 of the four-together / `endpoint::` under full
  CPU load. No flake, no hang, no retry.
- **The hang is a composition defect in the parked blocking wait**, and the
  mechanism is now measured rather than hypothesized: `wait_image_finished`'s
  condition is flipped by the client *socket worker* dropping its FIFO slot, which
  delivers **zero `ClientEvent`s**, so a wait that only wakes on
  `ConnectionBridge` inbox updates can never observe it (section 4).
- **Nothing to fix in main.** The refusal/held-behind path does complete; the 60 s
  `COMMAND_TIMEOUT` preparation expiry is the only slot end that reports to the
  inbox (section 5). Residual production risk is a hanging clipboard *reader*, not
  this path (section 7).
- Deliverable for the parked refactor: the two-phase composition design in
  section 6 (recommended: do **not** compose a wake source; give the helper a
  blocking slot-completion signal of its own).

## 1. What was run on plain main

Built once (`cargo test -p sheprd-gpui --bin spaceport --no-run`, 2 m 09 s) and
then driven through the produced test binary (`target/debug/deps/spaceport-a6eb47808ef02fba`)
so a repetition costs ~0.5 s instead of a cargo invocation. Every run under
`gtimeout 120` (or `300` for `endpoint::`), so a hang would have shown as a timeout
rather than stalling this investigation. None timed out.

The four named tests are in three different modules and no single substring covers
them, but this toolchain's libtest accepts several filters, so they really do run
together in one process and one thread pool:

```
$B a_remote_pane_that_turns_unready_never_gets_local_paths \
   unpastable_file_paths_report_like_a_drop_and_paste_nothing \
   text_paste_survives_a_settled_navigation \
   a_thumbnail_that_cannot_be_read_still_pastes_its_path
```

| Configuration | Repetitions | Result |
| --- | --- | --- |
| the four, one invocation | 15 | 15/15 green (0.02–0.04 s) |
| the four, 10 busy loops on 10 cores | 10 | 10/10 green (0.03–0.05 s) |
| each test alone (`--exact`) | 10 each = 40 | 40/40 green |
| `endpoint::lifecycle_tests::` (163 tests, parallel) | 12 | 12/12 green (0.38–0.49 s) |
| `endpoint::` (242 tests, parallel) | 5, under 10 busy loops | 5/5 green (0.68–0.85 s) |
| `endpoint::` (242 tests, parallel) | 1, after reverting instrumentation | green (0.63 s) |

`endpoint::` here selects the same 242 tests the parked refactor reported (238
green, 4 hung), so this is the same population that hung there. The hang does not
reproduce on main, and it is not a load-sensitivity of these tests: they pass 10/10
with every core saturated.

## 2. The two producers `wait_image_finished` waits for

`wait_image_finished` (`crates/sheprd-gpui/src/endpoint/lifecycle_tests.rs:406-414`)
spins `wait_until` (`:177-186`, 1 ms sleep, 3 s deadline) on

```rust
view.cancel_stale_image();
view.pending_images.is_empty()
```

`cancel_stale_image` (`crates/sheprd-gpui/src/window/images.rs:201-209`) cancels
every entry whose target is no longer current, then retains
`image.preparing || !image.cancellation.is_finished()` (`:208`). So an entry leaves
`pending_images` only when **both** of these have happened:

- **(a) the GPUI side finished**: `preparing = false`. Set in `finish_image_preparation`
  (`window/images.rs:483`) or in the task's early-exit arm (`:421-443`, the
  `image_target_current` / `prepared.is_err()` branches). Both run inside the
  `cx.spawn` task started at `window/images.rs:418`, i.e. on the GPUI foreground
  executor. `release_held_behind` (`window/pending_input.rs:478`) runs in the same
  `this.update`, so it is part of (a) too.
- **(b) the client side finished**: `ClipboardImageCancellation::is_finished()`
  (`crates/sheprd-client/src/clipboard.rs:52-57`) — true only once the socket
  worker's `ImageSlot` is dropped (`clipboard.rs:184-189`).

Producer (b) lives in the `sheprd-client` worker thread and never touches the
`ConnectionBridge` inbox. The reader thread that fills that inbox
(`crates/sheprd-gpui/src/connection.rs:244-273`) only wakes on
`client.events.recv()`; the slot's end is not one of those events in any of the
paths these tests take (`crates/sheprd-client/src/session.rs:461-467` cancelled,
`:487` receiver disconnected, or a completed write — none of them `deliver(...)`s;
`:472-483` is the one case that does, and only after the 60 s `COMMAND_TIMEOUT`).

## 3. Measured: the completing transition carries no reader update

Two throwaway instrumentations (applied, measured, reverted — `git status` is clean):

1. `PendingImage` state at wait entry and the number of loop iterations;
2. a global counter of events applied by the `ConnectionBridge` reader, printed at
   wait entry and wait exit.

All four named tests, `--test-threads=1`, before any of it was reverted:

```
text_paste…             WAIT-ENTRY pending=[(false, true)]  events=14
                        WAIT-EXIT  iterations=1             events=14
text_paste…             WAIT-ENTRY pending=[(false, false)] events=14
                        WAIT-EXIT  iterations=1             events=14
temporary_drop…         WAIT-ENTRY pending=[(false, false)] events=17
                        WAIT-EXIT  iterations=1             events=17
```

`(preparing, finished)`. The condition flipped from non-empty to empty with the
reader's applied-event counter **unchanged** (14→14, 17→17): the wake that ends the
wait is the worker dropping the slot, and it produces no inbox update at all.

The same instrumentation over every `wait_image_finished` execution in the whole
lifecycle filter (`--test-threads=1`, 58 waits from 26 call sites — several tests
loop over cases) shows what the wait is really for:

| entry state at the start of the wait | waits |
| --- | --- |
| `pending_images` empty already (returns on the first check) | 18 |
| entries present, every one `preparing == false` and its slot already finished | 38 |
| entries present, a slot the worker had not finished yet (`iterations = 2`) | 2 |
| any entry with `preparing == true` | **0** |

56 of the 58 exited on their first check and the other two on their second; none
took more than ~2 ms. `pending_images` held two and four entries in two of the 40
non-empty waits, so the table is not an artefact of single-paste tests.

So after the `cx.run_until_parked()` that every call site performs first, producer
(a) has always already finished, and the helper is waiting purely on producer (b) —
a cross-thread signal from the client worker, with no inbox trace.

## 4. Why that hangs the parked refactor

The parked `wait_until` replaced the 1 ms sleep with "block until
`connection::applied` reports a new applied inbox update" (its hung stack was
`wait_image_finished → await_reader → connection::applied::wait → mpsc::Receiver::recv`,
report section 5). With producer (a) already done and producer (b) silent, the
daemon in these tests is quiet after the frame the test already consumed, so no
further wake ever arrives and the receiver blocks forever. The 3 s spin it replaced
did not need a wake: it re-evaluated the condition on its own timer, which is
exactly the clock the refactor removed. The finding is therefore *"the refactor must
not route this wait through the reader wake"*, **not** "a paste path never
completes".

Two further constraints come out of the same measurement and shape the design:

- **A blocking call in the test thread also stalls producer (a).** In tests, GPUI
  foreground *and* background tasks are polled synchronously on the test thread by
  `TestDispatcher::run_until_parked` (`gpui-pre-0.3.10/src/platform/test/dispatcher.rs:76-78`,
  `while self.tick(false) {}`) → `TestScheduler::tick`
  (`gpui-pre-scheduler-0.3.10/src/test_scheduler.rs:276-278`). So the preparation's
  `executor.spawn(... upload.complete ...)` step (`window/images.rs:452-461`) can
  only advance while the test thread is inside `run_until_parked`/`tick`. A blocking
  wait may only be entered once no entry is `preparing`, or the two deadlock each
  other. Today's call sites satisfy that (0 of 58 waits saw `preparing == true`), but by
  accident of ordering, not by construction.
- **The 60 s expiry is not a substitute.** `COMMAND_TIMEOUT`
  (`crates/sheprd-client/src/limits.rs:19`) turns an unpublished slot into a
  delivered `CommandRejected` (`session.rs:472-483`), so a reader-wake wait would
  eventually (after 60 s) surface a *stall* instead of hanging — but only for the
  never-published case, and 60 s is longer than the whole lifecycle filter.

## 5. Refuted: the refusal / held-behind path does complete

The report's best hypothesis was an await that never resolves on the refusal or
held-behind path. On main it does resolve, for every refusal shape these tests take:

- Refused before preparation (`!current`): the task marks `preparing = false`,
  cancels, calls `release_held_behind(Refused)` and returns without publishing
  (`window/images.rs:421-440`). Dropping `upload` closes the slot's sender, the
  worker sees `TryRecvError::Disconnected` (`session.rs:487`), `continue`s, and
  `ImageSlot::drop` sets `finished`.
- Refused after preparation (`Err`/`Files`): same `release_held_behind`, and the
  publish step maps `Err`/`Files`/`Empty` to `Ok(false)` without sending bytes
  (`window/images.rs:452-461`), so the slot again ends through the closed sender.
- Cancelled by `cancel_stale_image`: `cancel()` sets the flag and wakes the worker;
  the worker drops the slot at `session.rs:462-467` (no event) or, if a bounded
  write had begun, ends the connection (`:463-465`).

The `held` FIFO order the tests then assert (`server.receive()` after the wait) is
produced by the worker's own ordering, independent of `pending_images`. So there is
no production bug here and nothing to fix in main; the wait's exit is timing-only
in the sense that it needs *a* wake, not that it needs a *long* time.

## 6. Composition design for the parked refactor

Recommendation: **give `wait_image_finished` its own blocking completion signal and
keep it off the reader wake** (option R1). Composing a second wake source into
`connection::applied` (option R2) is rejected: `sheprd-client` owns the slot and has
no dependency on the app, so it would have to reach back into a test-only GPUI
module per reservation — inverting the crate dependency for a test convenience.

### 6.1 The signal (gated in `crates/sheprd-client/src/clipboard.rs`)

**Gate with `feature = "test-support"`, not `integration-test`.** These tests are
unit tests of the `spaceport` bin run with default features, so a
`#[cfg(any(test, feature = "integration-test"))]` inside `sheprd-client` would
never be active: `sheprd-client` is an ordinary dependency there, not under
`cfg(test)`, and the `integration-test` feature only gates the separate `live_gui`
and `performance_gui` targets (`crates/sheprd-gpui/Cargo.toml:89-116`, `:162-169`).
`sheprd-client`'s own `test-support` feature is already turned on by
`sheprd-gpui`'s dev-dependencies (`crates/sheprd-gpui/Cargo.toml:77`, confirmed with
`cargo tree -e features -i sheprd-client`: `sheprd-client feature "test-support"` →
`[dev-dependencies] sheprd-gpui`), which is the established route for a test-only
client API — see `login_env::probe_for_tests` (`crates/sheprd-client/src/login_env.rs:60`).

`std::sync` only, no new dependency, `is_finished()`/`finished` untouched:

```rust
#[cfg(any(test, feature = "test-support"))]
pub(crate) struct Finish {
    done: std::sync::Mutex<bool>,
    woke: std::sync::Condvar,
}
```
with `finish()` (set and `notify_all`) and `wait()` (lock, `while !done`, `Condvar::wait`).

- `ImageLease` gains `#[cfg(any(test, feature = "test-support"))] finish: Arc<Finish>`,
  created in `ClientHandle::reserve_clipboard`
  (`crates/sheprd-client/src/handle.rs:103-142`) beside the existing flags.
- `ImageSlot::drop` (`clipboard.rs:184-189`) calls `lease.finish()` next to the two
  atomic stores; `ImageLease::drop` should too, so a connection torn down after the
  slot is dropped cannot leave a waiter parked (the waiter holds the lease handle's
  `Arc`).
- `ClipboardImageCancellation` (`clipboard.rs:47-58`) gains the same gated
  `finish: Arc<Finish>` (its `cancellation_handle()` at `:61-67` clones it), plus

```rust
/// Blocks until the worker's slot is gone. Test-only: the app must never block
/// on this, it polls `is_finished` from the UI thread.
#[cfg(any(test, feature = "test-support"))]
pub fn wait_finished(&self) { self.finish.wait(); }
```

No timeout, by the option-3 decision: an uncompleting slot should hang CI's job, not
masquerade as a timed flake.

### 6.2 The helper (`crates/sheprd-gpui/src/endpoint/lifecycle_tests.rs`)

Two phases, so neither producer is waited on while it can only be advanced by the
thread doing the waiting:

```rust
fn wait_image_finished(view: &gpui::Entity<SheprdWindow>, cx: &mut gpui::VisualTestContext) {
    loop {
        // Phase (a): every GPUI-side preparation step, which only this thread can run.
        let (waiting, preparing) = view.update(cx, |view, _| {
            view.cancel_stale_image();
            (view.pending_upload_handles(), view.pending_images_are_preparing())
        });
        if waiting.is_empty() && !preparing { return; }
        if preparing { cx.run_until_parked(); continue; }
        // Phase (b): each remaining entry was handed to the socket worker, which
        // ends its slot without an inbox update; block on the slot itself.
        for handle in waiting { handle.wait_finished(); }
    }
}
```

with two accessors on the window/images module (no behaviour change, no fields made
public): `pending_images_are_preparing() -> bool` and
`pending_upload_handles() -> Vec<ClipboardImageCancellation>` for the entries that
are not yet finished — the cancellation handle is already `Clone`
and already held by `PendingImage` (`window/images.rs:48-57`), so none of the gated
client API leaks into production code paths.

Because 0 of 58 waits ever entered with `preparing == true`, phase (a) never runs
the loop twice: 56 of them return on the first check (18 with nothing pending, 38
whose slots had already ended) and the other two park once in `wait_finished()`
for the worker's drop — the two measured `iterations = 2` cells.

### 6.3 What else in the parked refactor is unaffected

Everything in the report's section 4 stands as-is: `project_until`
(`lifecycle_tests.rs:160-176`) and the other `poll_until` loops *are* fed by the
reader (their producer is `poll_endpoints` observing inbox updates), so the wake
source is right for them. Only `wait_image_finished` was routed through a producer
it cannot see. Also keep the mock peer's socket read/write timeouts — they bound a
failing run and are not what hung.

## 7. Residual risks / production questions

- The only way producer (a) can fail to finish is a clipboard read that never
  returns: `window/clipboard::read` is documented background-only and bounded, and
  these tests use GPUI's in-memory test clipboard, so it is untested here. A hung
  `wl-paste`/`xclip`/pasteboard read would leave the paste "preparing" in the app
  with no notice; that is a separate concern from this finding and was not observed.
- A slot that is never published is ended by the 60 s `COMMAND_TIMEOUT` and reported
  as `CommandRejected`; a *cancelled* slot ends silently by design (the app already
  showed its own reason). If a future change makes cancellation a user-visible
  outcome, that asymmetry becomes a real gap.
- The 26 `wait_image_finished` call sites (`image_paste.rs`, `image_paste_native.rs`,
  `file_paste.rs`, `held_paste.rs`, `temporary_drop.rs`) all currently precede the
  wait with `cx.run_until_parked()`. The loop in 6.2 does not depend on that, but the
  design note should say so: adding a call site without the pump is safe only because
  of phase (a) in the helper.

## 8. Commands run

```
df -h /System/Volumes/Data                                        before: 13 Gi free
cargo test -p sheprd-gpui --bin spaceport --no-run                 ok, 2m09s (fresh worktree)
$B endpoint::lifecycle_tests::                    ×12            12/12 ok, 163 passed
$B <four filters>                                 ×15            15/15 ok, 4 passed
$B --exact <one filter> (each of four)            ×10 each       40/40 ok
$B <four filters>  (10 busy loops on 10 cores)    ×10            10/10 ok
$B endpoint::      (10 busy loops on 10 cores)    ×5             5/5 ok, 242 passed
$B --nocapture --test-threads=1 <four filters>  (instrumented)   4 passed, WAIT-* trace
$B --nocapture --test-threads=1 endpoint::lifecycle_tests:: (instrumented)  163 passed, 58 WAITSTAT lines
cargo test -p sheprd-gpui --bin spaceport --no-run (reverted tree)  ok
$B <four filters> ×5; $B endpoint:: ×1            (reverted tree) 5/5 ok; 242 passed
git status --short                                clean (instrumentation reverted)
```

Instrumentation was applied to `window/images.rs` (a `pending_image_states()`
accessor), `endpoint/lifecycle_tests.rs` (`wait_image_finished` trace) and
`connection.rs` (a reader event counter) and then reverted with `git checkout`; the
final `git status --short` is empty. Disk: 13 Gi free before the first build, 1.5 GiB
of `target/debug/incremental` removed mid-session, 6.1 Gi free at the end.

## 9. Deferred

- Re-applying the parked refactor with section 6's helper, and re-running
  `endpoint::` 10× plus the loaded runs, is the next piece of work and belongs to
  whoever re-lands that branch (it was not in scope here, and the refactor is not on
  main).
- `just ci` (lint, i18n, spec, artwork, all-features), the full
  `-p sheprd-gpui --bin spaceport` suite (4107 tests) and the Windows/macOS legs were
  not run; only the filtered groups above.
- No native UI was exercised: none of this is a rendering change.
