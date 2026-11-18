# Clock conversions: the five prioritized items from `docs/clock-audit.md` §6

Worktree: `/Users/clint/code/spaceport/.worktrees/clock-conversions`
Base: `c2b368721` (merge of #109 on main)
Commits: small, one per site plus the notes, all on top of `c2b368721`; none
pushed and none staged.
Files changed outside `crates/sheprd-client`: `crates/sheprd-core-client/src/start.rs`
(the shared wait), `crates/sheprd-gpui/src/{menu/devices/setup.rs, processes.rs,
control/client/local/processes.rs, core_terminals/observation.rs, notice.rs,
state.rs, window/status_bar.rs}` and their tests. `crates/sheprd-acp` is untouched
(a peer owns it this wave).

Status at a glance:

- §6 item 1 (one wait helper): done. **Four of the audit's eleven copies** of the
  deadline loop are converted, plus a fifth from the audit's §3.1 (an SSH master's
  reap); the remaining seven are left alone for the reasons in §1.3. The helper
  keeps the deadline-kill-and-reap semantics and the caller's own bound (§1.1).
- §6 item 2 (`observation.rs`): done. The 100 ms spin is gone; a request wakes the
  reader, and the 2 s machine re-read is the wait's own bound (§2).
- §6 item 3 (`control/client/local/processes.rs`): done. The 100 ms-per-attempt nap
  is a blocking receive on the watch (§3).
- §6 item 4 (`forward.rs`): done, by a master-exit notification rather than a
  timer: the master's standard output is drained, so its close wakes the worker.
  A 30 s bound survives for an exit no close reports (§4).
- §6 item 5: `transport/windows.rs` no longer reads a clock for a pipe name
  (Windows CI leg only, by reading here) and the status bar reads one instant per
  frame instead of three (§5).
- Verified locally: `cargo test -p sheprd-client` (full), the focused
  `sheprd-gpui --bin spaceport` filters, clippy `-D warnings` on the touched
  targets, fmt, file-size, keep-sorted, and `just check-commits`. Deferred: the
  `cfg(windows)` and `cfg(unix)` platform legs, `--all-features` clippy (needs the
  Sparkle framework), the full 4100-test GPUI suite, and any native window (§6).

## 1. Item 1: one bounded wait for a child process

### 1.1 The helper and why it is shaped this way

`crates/sheprd-client/src/limits.rs` now ends with `pub mod wait`, re-exported as
`sheprd_client::wait` (`lib.rs`), so `sheprd-gpui` and `sheprd-core-client` — both
already depend on `sheprd-client` — reach it without a new crate or trait.

```rust
pub fn exit(child: &mut Child, deadline: Instant, cancelled: impl Fn() -> bool)
    -> io::Result<Exit>            // Exit::Exited(status) | Exit::Stopped(TimedOut | Cancelled)
```

- **The deadline kill and the reap stay inside the helper**, as the reviewer
  required: on a missed deadline or a stop it kills and reaps the exact child
  before it answers, so no caller can leave one running or unreaped.
- **The caller keeps the child, its pipes, and its own bound.** `&mut Child`, not
  an owned child handed to a waiter thread, because two converted callers need the
  child after the wait: `sheprd-core-client/src/start.rs` reads the start command's
  stderr tail on failure, and `ssh::run_with_stderr` keeps its `ChildGuard` (whose
  `Drop` kills and reaps on every early return, including the ones after the wait).
  The `Instant` deadline parameter, not a `Duration`, lets `ssh` reuse one deadline
  for the wait and the stderr drain it does afterwards.
- **A child's exit has no notification to wait on**, so the helper re-reads
  `try_wait` every `limits::POLL` (10 ms) rather than at each caller's old cadence
  (20, 25, 50 ms). That is the one behavioral difference: a wait that used to add
  up to 50 ms of latency adds at most 10, and a stop ends the wait at once instead
  of up to a period later. Nothing else changed: the check order (`try_wait`, then
  cancel, then deadline) matches `ssh`'s old loop, and the non-cancelling callers
  see the same order as before.

Deviation from the audit's sketch, and why: §4 of the audit sketches a waiter
thread that owns the child with the caller blocking on `recv_timeout`. The child
has to stay with the caller (see above), and a waiter thread that owns it would
also add a spawn-failure path that leaks the child. This version keeps the
reviewer's requirement — deadline, kill, and reap in one place — with fewer moving
parts and no ownership change at any call site. If a future caller needs to do
work *during* the wait, the helper can grow a thread behind that signature without
touching the call sites.

Tests (`crates/sheprd-client/src/limits/tests.rs`, three, no wall clock):
a child that exits reports its status; a deadline already passed reports
`Stopped(TimedOut)` **and the child is reaped** (`try_wait` answers with a status);
a stop reports `Stopped(Cancelled)` and the child is reaped. Both stopping tests
use a `cat` blocked on a stdin pipe the test holds open, so nothing can exit on its
own, and pass `Instant::now()` or `Instant::now() + BOUND` so no test waits on a
clock. `BOUND` (60 s) is a failure-only bound.

### 1.2 The copies converted

Four of them are the audit's own list; the `ssh/mux/shared.rs` reap is the extra
one, which the audit listed in §3.1 as an honest sleep rather than in §4's eleven.

| Site | Was | Bound kept |
| --- | --- | --- |
| `sheprd-client/src/sessions.rs` `delete_command` | `try_wait` + 20 ms nap + deadline kill/reap | caller's `timeout` (20 s stop, 15 s delete, 45 s remote) |
| `sheprd-client/src/ssh.rs` `run_with_stderr` | `try_wait` + 25 ms nap + cancel + deadline | caller's `timeout`; `cancelled` closure kept |
| `sheprd-client/src/ssh/mux/shared.rs` `Process::drop`'s reap thread | `try_wait` + `POLL` nap + `STOP_GRACE`, then kill+wait | `STOP_GRACE` |
| `sheprd-core-client/src/start.rs` `start` | `try_wait` + 25 ms nap + deadline kill/reap | caller's `timeout` |
| `sheprd-gpui/src/menu/devices/setup.rs` `run_bounded_untraced` | `try_wait` + 50 ms nap + deadline kill/reap | caller's `timeout` |

(That is five sites: the audit's `sessions.rs:119`, `ssh.rs:286`,
`ssh/mux/shared.rs:137`, `core-client/src/start.rs:48` and
`menu/devices/setup.rs:257`. `ssh.rs` was listed as "M — needs a watchdog that
kills on cancel"; the helper *is* that, because its cancellation kills and reaps
the child itself.)

### 1.3 The seven copies that are not copies, and why they were left alone

Each one has a second in-loop condition, so a wait helper could not carry them
honestly. None of them is a "same pattern" duplicate of the shared one, and each
sits in a file another worker or another concern owns this wave:

- `sheprd-gpui/src/daemon.rs` `connect` — the loop's condition is a **socket
  connect attempt** and it must never kill the daemon (it outlives the window). It
  already has a daemon-exit channel; its own conversion is the audit's §3.1 "two
  line `recv_timeout`", not this helper, and it is not in this brief's file set.
- `sheprd-gpui/src/listening_ports/tunnel.rs` `wait_ready` — already
  channel-driven (`recv_timeout`); the second condition is the child's exit through
  a closure, and it also cannot kill the tunnel child here.
- `sheprd-gpui/src/window/clipboard.rs`, `clipboard/macos.rs`,
  `clipboard/windows.rs` — the loop is a **socket read loop** that reaches the
  child's status only after EOF; the deadline is a read timeout, and the kill is
  unconditional on the way out. The audit classes all three as honest.
- `sheprd-client/src/ssh/mux/shared.rs:457-470` `start_bridge` — waits for **another
  process to create a socket file**; the socket itself gives no notification, so a
  50 ms re-read of `socket.exists()` stays (audit: honest).
- `sheprd-acp/src/process.rs` — the `Child` sits in a `Mutex` that `stop()` must
  reach, so a blocking wait would deadlock the stop path (documented at `:94`).
  Owned by a peer this wave.
- Two more instances the audit did not list, read and left alone:
  `ssh/failure.rs::diagnose` (cancel flag plus a stderr receive that shares the same
  deadline; it deliberately does **not** kill, the caller does) and
  `login_env.rs`'s probe (a read-timeout probe, the audit's §3.2 `M`, a different
  fix). `upload.rs::finish`, `script.rs`'s runner, and `forward.rs`'s starting loop
  are read loops or multi-condition loops of the same kind.

## 2. Item 2: `core_terminals/observation.rs` — the 100 ms metadata spin

**Before.** The `herdr-core-machines` thread slept 100 ms between passes for the
life of the session. Each pass: if `Instant::now() >= next_machine_read`, call
`machine_list()` and set `next_machine_read = now + 2 s`; then, if a remote
document request had been left in its mailbox, `doc_get` that machine and post the
parsed document to its results mailbox. Errors surfaced as
`Error::Core(..)`/`parse_document(..)` results placed in the same mailboxes the
main worker drains (`machine_results`, `remote_results`) — unchanged.

**Now.** The thread blocks on `woken.recv_timeout(refresh)` where `refresh` is
`next_machine_read.saturating_duration_since(now)`:

- the 2 s machine re-read is the **wait's own bound** (the "timer" the audit asked
  for), so an idle session does no work between refreshes;
- a remote document request **wakes it at once** through the new
  `RemoteRequests { latest, wake }`, whose `replace()` stores the request and
  `try_send`s a wake (a full channel already holds one, so no queue grows);
- the observer's exit is the wake's disconnect: `watch` holds the sender and drops
  it when it returns, so the reader leaves immediately instead of up to 2 s later;
- `read_stop` is still checked at the top of each pass.

What it waited for is unchanged: the same three mailboxes, the same
`machine_list`/`doc_get` calls in the same order, the same epoch-fenced
publication on the main worker. Only the sleeping is gone.

**Test.** `crates/sheprd-gpui/src/core_terminals/observation/tests/machine_reader.rs`:
a replaced request wakes the reader, the taken request leaves no stale wake behind,
and a second request while a wake is pending is not queued twice. Both receives are
`recv_timeout(Duration::ZERO)` lookups of a wake already sent, never a wait.

## 3. Item 3: `control/client/local/processes.rs` — `spaceport processes`

**Before.** `sample` drained `Watch::try_update` and slept 100 ms per attempt while
waiting up to 7 s (`WAIT`) for the first listing whose CPU is measured; a failed
sample was returned as the answer.

**Now.** `Watch` gained `update(wait)`, a `recv_timeout` on the same channel
(`timeout` → `None`, `disconnected` → the same `Error::Stopped`), and `sample`
waits on it for `until.saturating_duration_since(now)`. The worker's answer is the
wake: a measured listing ends the wait the moment it arrives, a failed sample is
still returned at the deadline, and no clock is re-read on a timer. `try_update`
stays for the panel, which must never block.

**Test (existing, still passing).**
`control::client::local::processes::tests::sampling_waits_for_measured_cpu_or_returns_the_latest`
covers all four shapes with a fake watch and pre-sent updates, including
`Duration::ZERO` for the deadline and the empty case.

## 4. Item 4: `sheprd-client/src/forward.rs` — the 4 Hz forever loop

**Before.** After a forward reported its listener, the worker looped forever on
`master(slot)` (a `try_wait` under the slot mutex) and slept 250 ms between reads,
for the life of the forward. `stop()` set `stopped` and killed the child; the loop
noticed up to 250 ms later.

**Now.** The master's standard output is `Stdio::piped()` instead of
`Stdio::null()`: closing it is the one exit notification a child process gives this
one. A `herdr-forward-exit` thread drains it (so a chatty master can never block on
a full pipe) and sends a wake on EOF; the worker blocks on
`woken.recv_timeout(EXIT_RECHECK)` and never re-reads the master on a timer. Both
ends of the old loop are covered by that one signal: a master that exits closes the
pipe, and `stop()` kills the master, which closes it too. Stderr stays `null` (it
can carry banners and secrets, and nothing reads it).

Why not a blocking `Child::wait`: the owner must be able to `kill` the master from
the UI thread at any moment, and `std::process::Child` cannot be awaited and killed
from two threads (the audit calls the same shape in `acp/process.rs` honest by
design). The pipe close gives the notification without giving up the kill handle.

`EXIT_RECHECK` (30 s) is a **failure-only bound**, not the wake: it covers an exit
no pipe close reports, which needs a descendant of the master to have inherited and
held the pipe. In that pathological case the old code reported the end within
250 ms and this one within 30 s; every ordinary exit is reported at once, which is
the trade the conversion makes.

**Tests (existing, still passing, `forward::tests`):** `a_forward_is_alive_only_while_its_master_runs`
(SIGKILLs the master and still expects `Ended(ForwardExit)`),
`a_master_that_exits_ends_the_forward_without_a_retry`,
`a_granted_port_is_listening_and_stop_kills_the_master`,
`a_stopped_or_never_started_forward_is_not_alive`,
`a_master_that_never_connects_times_out_and_is_killed`, and the two refusal tests —
11 of the 15 `forward::` tests drive a real child process.

## 5. Item 5: the two clock reads that bought nothing

- **`sheprd-client/src/transport/windows.rs` `private_pipe`**: the name was
  `\\.\pipe\herdr-client-<pid>-<counter>-<subsecond nanos>`. The wall-clock field
  bought nothing — pid plus the module's `NEXT` counter is already unique for this
  run, and the pipe is created with `FILE_FLAG_FIRST_PIPE_INSTANCE`, so a leftover
  from an earlier run fails the call instead of being shared. The clock read is
  gone, together with its `SystemTime`/`UNIX_EPOCH` imports. **Read only here**:
  no Windows target is installed in this worktree (`rustup target list` shows only
  `aarch64-apple-darwin`), so compilation and the pipe tests are deferred to the
  Windows CI leg.
- **`sheprd-gpui/src/notice.rs` and the status bar**: `render_status_bar` read the
  clock three times per frame (two `is_some`/`text` calls plus the one inside
  `LiveState::status_text`) while deciding whether a notice still shows. It now
  takes **one instant when the frame is drawn** and passes it down:
  `Notices::text_at`/`is_some_at(now)` and
  `LiveState::status_text(local_error, now)`. A notice's deadline stays on the real
  clock, exactly as `window/flash.rs:82-84` requires for expiry; only the duplicate
  reads inside one frame are gone. `Notices::is_some` now has no caller in the app
  and is gated to `test`/`integration-test`, where the smoke harness still uses it;
  `is_none` stays for `window/pending_input.rs`, which is not a render path.

The three `status_text` call sites that were asserts in tests now pass
`Instant::now()` explicitly (the same read the code did internally), and none of
them waits.

## 6. Commands run, and results

```
df -h /System/Volumes/Data                             11 Gi free at start; 5.0 Gi free at end
cargo test -p sheprd-client --locked                   ok — 386 passed, 2 ignored (explicit-resource), 1 doctest
cargo build -p sheprd-client / -p sheprd-core-client   ok
cargo clippy --locked -p sheprd-client -p sheprd-core-client \
    --all-targets --all-features -- -D warnings        ok
cargo test -p sheprd-gpui --bin spaceport --locked --no-run
                                                       ok, 1m 07s
cargo test -p sheprd-gpui --bin spaceport --locked -- \
    core_terminals:: processes:: notice status_bar      ok — 203 passed, 8 ignored
cargo test -p sheprd-gpui --bin spaceport --locked -- \
    machine_reader sampling_waits                      ok — 2 passed (the new and the changed test)
cargo clippy --locked -p sheprd-gpui --bin spaceport --tests -- -D warnings
                                                       ok (default features)
cargo fmt --all -- --check                             clean
bash scripts/check-file-size.sh                        all files within 1000 lines
python3 scripts/check-sorted.py                        all keep-sorted regions sorted
python3 scripts/release/check-commit-messages.py range origin/main..HEAD
                                                       ok, every commit subject
git diff --check                                       clean
```

Not run locally, deliberately: `cargo test -p sheprd-gpui` in full (4100+ tests),
`just ci`, `--all-features` gpui clippy/build (the `sparkle-updater` build script
needs `SPARKLE_FRAMEWORK_PATH`; `--all-features` fails locally with "Sparkle.framework
was not found"), `just lint-windows`/`lint-linux` (targets not installed), and any
native window — none of this is a rendering change.

## 7. Residual risks and open questions

1. **`cfg(windows)` is unproven here.** `transport/windows.rs` was changed by
   reading; only the Windows CI legs can compile it. Nothing else in this batch has
   a platform branch, so `cfg(unix)` code is proven by the local runs and
   `cfg(windows)` is proven by the Windows jobs.
2. **`forward.rs`'s inherited-pipe edge.** If a descendant of the master inherits
   fd 1 and outlives it, the exit notification never arrives and the `Ended` report
   waits for the 30 s bound instead of the old 250 ms. `ssh -N` with
   `ControlPersist=no` does not fork locally, and an exec `ProxyCommand` has its
   stdio rewired by ssh itself, so this needs a config that leaves a child holding
   the master's stdout. `PortForward::alive()` is unaffected: it still asks the
   child directly.
3. **The shared wait polls every 10 ms.** It cannot be event-driven (a child's exit
   has no notification in `std`), and the tighter cadence costs 2.5x the wakeups of
   the old 25 ms loops for the seconds a wait lasts. If that ever shows up in a
   profile, `POLL` is the one knob.
4. **`arg`/master-state reads under a mutex.** Unchanged: `master(slot)` and the
   forward loop still share the slot mutex with `stop()`/`alive()`, exactly as
   before.
5. **Not converted, on purpose** (see §1.3): `daemon.rs`, `tunnel.rs`, the three
   clipboard readers, `start_bridge`'s socket wait, `acp/process.rs` (peer-owned),
   and the audit's honorable mentions (`sound/playback.rs`, `login_env.rs`).

## 8. Deferred

- Windows CI legs for `transport/windows.rs` and the `cfg(windows)` branch of
  `notices`/`PortForward` (the latter is a `match *self` on an empty enum
  elsewhere, untouched).
- `just ci` and the full GPUI suite, as above.
- Nothing in this batch needs native QA: no rendering, layout, or glyph changes;
  the status-bar change is which instant a comparison uses, not what is drawn.

## CI failure found after the notes: the forward wake's reader exited early

`forward::tests::a_forward_is_alive_only_while_its_master_runs` failed on
`ubuntu-24.04-arm` ("no event from the forward"). The exit-reader thread's loop
(`while matches!(read, Ok(n) if n > 0)`) treated any `Err` as EOF, including
`Interrupted`. On Linux a signal can interrupt the blocking pipe read; the
reader then sent the single wake while the master still ran, the listening
loop's first `recv_timeout` consumed it, and the real exit had no reader left:
the worker sat out the 30 s re-check bound while the test's 10 s `WAIT`
expired. macOS restarts interrupted pipe reads, so the macOS CI legs passed.

The reader now retries `Interrupted` and breaks only on true EOF or a real
read error. The 30 s `EXIT_RECHECK` bound stays as the design's guard for an
exit no pipe close reports (an inherited write end). 12 consecutive local runs
of the failing test pass.
