# Clock audit: wall clocks, monotonic clocks, and sleeps

Inventories and classifies every wall-clock read and every `thread::sleep` in
production code, against the standing rule that production code reads a
monotonic clock (`Instant`, or `crate::motion::now()` for UI motion) and reads
the wall clock only where a duration, format, or persisted value honestly
requires it.

Classes: **DISPLAY/TIMESTAMP** (a person or a log reads it), **ANIMATION** (must
be `motion::now()`), **WORKER POLLING** (a spin that could be event-driven),
**DEADLINE/BOUND** (a bounded wait on an external process), **DOMAIN TIME**
(stored, compared, persisted, or shared across processes).

## Method

```sh
grep -rn "SystemTime::now\|Local::now\|Utc::now" crates/*/src --include=*.rs   # 29 production reads
grep -rn "Instant::now"                       crates/*/src --include=*.rs      # 327 reads, harnesses included
grep -rn "motion::now"                        crates/sheprd-gpui/src          # 51 motion reads
grep -rn "thread::sleep"                      crates/*/src --include=*.rs      # 34 production sleeps
```

Excluded by reading, not only by path: `*/tests/`, `*_tests/`, `tests.rs`, and
inline `#[cfg(test)]` blocks (`login_env.rs:486`, `script_tests/wsl.rs`), plus
the `integration-test` harnesses `smoke.rs` and `performance.rs`.

The brief's counts hold for its own list and understate the rest. All 8 named
GPUI sleeps are real (`clipboard.rs`, `clipboard/macos.rs`,
`clipboard/windows.rs`, `sound/playback.rs`, `menu/devices/setup.rs`,
`listening_ports/tunnel.rs`, `daemon.rs`, `core_terminals/observation.rs`), and
there are 34 in production in total: 11 in `sheprd-gpui` (3 more:
`core_terminals/hub.rs`, `control/client/events.rs`,
`control/client/local/processes.rs`), 16 in `sheprd-client`, 3 in
`sheprd-control-protocol`, 2 in `sheprd-acp`, and 1 each in
`sheprd-core-client` and `sheprd-diagnostics`. Wall-clock reads are 29, matching
"~30", and **every one of them is in the honest column** but one that is not a
time at all (section 2).

## 1. The two-clock rule this repo already follows

`Instant` is correct for every span that begins and ends in this process.
The wall clock is required in exactly three situations, and the code documents
all three:

1. **Across a process boundary** — a stamp written here and read by another
   program, or by a later run (`sheprd-acp/src/session.rs:538-548`).
2. **Across a suspend** — on macOS `Instant` does not advance while the machine
   sleeps, so "how much wall time passed" cannot come from it. Both
   `endpoint/wake.rs:80-96` (`woke`: `passed - awake >= SLEEP_GAP`) and
   `devices_overview/history.rs:57-70` pair an `Instant` with a `SystemTime` and
   trust the wall clock only for the gap that exceeds the monotonic one.
   `network_quality.rs:88-91` states it outright: the stamp "keeps counting
   while the machine sleeps, so the probes from before a night's sleep expire on
   waking".
3. **Formatting and persistence** — timestamps shown to a person, and values
   stored in files that outlive the process.

The complementary rule is the one in `AGENTS.md`: motion in the UI reads
`motion::now()` (`motion/slow.rs:3-6,40-42`), while an *expiry* stays on the
real clock — `window/flash.rs:82-84` says so and `window/toasts.rs:262` vs
`:173-188` shows the split in one file.

## 2. Wall-clock reads (29 sites)

| Site | Feeds | Class | Verdict |
| --- | --- | --- | --- |
| `sheprd-acp/src/session.rs:543` | turn `started_ms`, read by the in-pane TUI, mirrored logs, and the GUI | DOMAIN TIME | Honest: written by one process, read by others; no monotonic clock crosses that boundary. |
| `sheprd-acp/src/herdr.rs:295` | `Sequence::next`, report numbers that must keep rising across restarts | DOMAIN TIME (identifier) | Honest: the clock is used only as a cross-restart ordering source, and `self.0 + 1` guards within a run. |
| `sheprd-client/src/transport/windows.rs:185` | `nanos` in a private pipe name | **Not a time** | **Convert, S**: `pid` plus the `NEXT` counter (`:184`) already make the name unique in the run. Drop the clock (or use a UUID) so a clock jump cannot influence a pipe name. |
| `sheprd-diagnostics/src/crash.rs:58` | local time in the crash report | DISPLAY/TIMESTAMP | Honest: a person reads it; `:139-142` explains why the *file name* is UTC so names still sort across DST. |
| `sheprd-diagnostics/src/crash/windows.rs:202,234` | minidump report and name | DISPLAY/TIMESTAMP | Honest, same reasoning. |
| `sheprd-diagnostics/src/lib.rs:480` | log line timestamp | DISPLAY/TIMESTAMP | Honest: a log must say when, in the same time base as the tools reading it. |
| `sheprd-gpui/src/sidebar/activity.rs:88` | `last_opened` per panel, compared with daemon item times and stored | DOMAIN TIME | Honest: compared against `DateTime<Utc>` from snapshots and persisted. |
| `sheprd-gpui/src/devices_overview.rs:140` (+ `:139`) | sleep gap for the activity history | DOMAIN TIME | Honest and exemplary: the paired `Instant` decides, the wall clock only fills a suspend (`history.rs:57-70`). |
| `sheprd-gpui/src/usage/section.rs:18`, `usage/render.rs:48`, `usage/panel.rs:78` | "resets in 2h", pace, reading age | DISPLAY/TIMESTAMP | Honest. Nit: three independent reads per frame; stamp once in the tick and pass it (S) for determinism in headless tests. |
| `sheprd-gpui/src/network_quality.rs:189` | probe arrival stamps and expiry | DOMAIN TIME | Honest, documented at `:88-91`; the paired `Instant` arrives in the same call. |
| `sheprd-gpui/src/endpoint/wake.rs:75` | sleep and clock-set detection | DOMAIN TIME | Honest: the reference implementation (`:80-96`). |
| `sheprd-gpui/src/endpoint/polling.rs:55` | recovery anchor, paired with `Instant::now()` at `:54` | DOMAIN TIME | Honest. |
| `sheprd-gpui/src/endpoint/polling.rs:105` | mute reconciliation clock | DOMAIN TIME | Honest: `muted_at` is persisted (`notification_mutes.rs:159`, `:557`). |
| `sheprd-gpui/src/device_tags.rs:278` | persisted tag timestamps | DOMAIN TIME | Honest. |
| `sheprd-gpui/src/menu/mute.rs:66`, `control/server/work/mute.rs:56` | `muted_at` for the stored mute set | DOMAIN TIME | Honest, as above. |
| `sheprd-gpui/src/overview/view.rs:48,49,698`, `overview.rs:119,141` | elapsed "4m", preset times, snooze wake times, snooze `until` | DISPLAY + DOMAIN TIME | Honest: `presets::presets(&Local::now())` builds local-clock times a person picks, and `Preset::until` is stored and compared. |
| `sheprd-gpui/src/control/server/hosts.rs:52` | `now` for the usage answer a CLI reads | DISPLAY/TIMESTAMP | Honest. |
| `sheprd-gpui/src/control/server/events.rs:50` | `instance()` = pid ^ start ms | DOMAIN (identifier) | Honest: distinguishes this run from an earlier one. `:74` (`at`) is the event timestamp for `events`/`wait` output. |
| `sheprd-gpui/src/palette/frecency.rs:110` | persisted visit times | DOMAIN TIME | Honest. |
| `sheprd-gpui/src/chat.rs:54` | elapsed of a turn the agent process started | DOMAIN TIME | Honest, but see residual risk 1. |

No wall-clock read drives an animation, and no span inside one process is
measured with one. **28 of 29 are honest**; one (`transport/windows.rs:185`)
would be better with no clock at all.

## 3. Sleeps (34 sites)

### 3.1 DEADLINE/BOUND — waiting on another process

| Site | Period | Verdict |
| --- | --- | --- |
| `sheprd-gpui/src/daemon.rs:324` | 50 ms | Convert **S**: the loop already has `exit_rx` (`:296`); `exit_rx.recv_timeout(50 ms)` replaces the sleep so a daemon that dies is noticed at once instead of up to 50 ms later. |
| `sheprd-gpui/src/menu/devices/setup.rs:257` | 50 ms | Convert **S** with the section 4 helper: a thread calls `child.wait()` and the parent `recv_timeout(timeout)`. |
| `sheprd-gpui/src/listening_ports/tunnel.rs:212` | 100 ms | Same shape (the `recv_timeout` at `:214` is already the good half). Convert **S**. |
| `sheprd-gpui/src/core_terminals/hub.rs:340` | 50 ms | Honest bound on a slow file system after `start` returned; no channel exists to wait on. Leave. |
| `sheprd-client/src/sessions.rs:119` | 20 ms | Convert **M** (needs the helper plus its own deadline). |
| `sheprd-client/src/ssh.rs:286` | 25 ms | Honest: the loop must also poll `cancelled()`; converting means a watchdog thread that kills on cancel. **M**. |
| `sheprd-core-client/src/start.rs:48` | 25 ms | Convert **M** (helper). |
| `sheprd-client/src/ssh/mux/shared.rs:469` | 50 ms | Honest: another process creates the control socket whenever it authenticates. |
| `sheprd-client/src/ssh/mux/shared.rs:137` | 50 ms | Honest: reap grace before `kill`. |
| `sheprd-acp/src/process.rs:98` | 50 ms | Honest by design: the `Child` sits in a `Mutex` that `stop()` must reach (`:88-101`), so blocking `wait()` would deadlock the stop path. Documented at `:94`. |
| `sheprd-acp/src/process.rs:134` | 50 ms | Honest: the grace period between `SIGTERM` and `SIGKILL` for the agent's process group, ended by a signal test rather than a notification (`:116-137`). |
| `sheprd-gpui/src/window/clipboard.rs:373` | 5 ms | Honest: reached only after stdout EOF, to observe the exit status under a deadline (`:309-315` explains the short read timeout that removes the earlier sleep-per-read). |
| `sheprd-gpui/src/window/clipboard/macos.rs:200` | 5 ms | Honest, same reasoning (`:145-152`). |
| `sheprd-gpui/src/window/clipboard/windows.rs:90` | 10 ms | Honest, same reasoning. |

### 3.2 WORKER POLLING — spins that could be event-driven

| Site | Period | Verdict |
| --- | --- | --- |
| `sheprd-gpui/src/core_terminals/observation.rs:633` | 100 ms | Convert **M**: the worker polls a mailbox (`:610`, `:619`) it is the only consumer of; a channel with `recv_timeout` serves both the 2 s machine refresh and remote doc requests. The machine re-read at `:625` also runs on a timer while nothing changes. |
| `sheprd-gpui/src/control/client/local/processes.rs:138` | 100 ms | Convert **S**: block on the watch instead of draining `try_update` in a loop. |
| `sheprd-client/src/forward.rs:304` | 250 ms, forever | Convert **M**: a 4 Hz spin for the life of the forward, only re-reading master state; notify on state change. |
| `sheprd-gpui/src/sound/playback.rs:65` | 25 ms | Honest-ish: rodio gives no completion event, and `sleep_until_end` cannot see `cancel`/`stop`. **M** to revisit. |
| `sheprd-client/src/login_env.rs:243` | 10 ms | Convert **M**: a read timeout on the probe pipe instead of non-blocking reads plus a nap. |
| `sheprd-client/src/forward.rs:263` | 50 ms | Honest: another process creates the socket. |
| `sheprd-gpui/src/control/client/events.rs:141` | 250 ms | Honest: a CLI waiting for an app that is not up, with nothing to subscribe to yet. |
| `sheprd-diagnostics/src/lib.rs:224` | 5 ms | Honest backoff, though `send_timeout` (crossbeam) would remove it. **S**. |
| `sheprd-client/src/ssh/mux/shared.rs:392` | computed | Honest and good: the janitor computes its own next expiry from `Instant` (`:381-389`) and sleeps exactly that. |
| `sheprd-client/src/upload.rs:368,390,420,463`, `script.rs:319` | 10 ms | Honest: pacing after `Interrupted`/`WouldBlock`/`TimedOut` on a socket nobody notifies. |
| `sheprd-client/src/ssh/failure.rs:217`, `private_endpoint/windows.rs:149,221` | 10–25 ms | Honest retry pacing. |
| `sheprd-control-protocol/src/socket.rs:140` | 100 ms | Honest: accept-failure backoff, so a broken listener cannot spin (`:134-141`). |
| `sheprd-control-protocol/src/socket/pipe.rs:164,205` | 1→50 ms | Honest: `PIPE_BUSY` on Windows has no notification; the backoff is exponential within the caller's deadline. |

## 4. The structural finding: one wait idiom, eleven times

The same loop — `try_wait()` (or an equivalent existence check) under a
`Instant` deadline, sleeping 5–50 ms — appears in `daemon.rs:303-326`,
`menu/devices/setup.rs:247-261`, `listening_ports/tunnel.rs:209-227`,
`window/clipboard.rs:325-375`, `window/clipboard/macos.rs:145-201`,
`window/clipboard/windows.rs:41-95`, `sheprd-client/src/sessions.rs:105-121`,
`ssh.rs:274-289`, `ssh/mux/shared.rs:457-470`, `core-client/src/start.rs:37-53`,
and `sheprd-acp/src/process.rs:93-101`. Each pays 5–50 ms of avoidable latency
per wait and repeats the same three deadline checks in its own words.

One helper — `wait::exit(child, deadline, cancelled)`, a waiter thread plus
`recv_timeout` — covers `menu/devices/setup.rs`, `sheprd-client/src/sessions.rs`
and `core-client/src/start.rs` outright, gives `daemon.rs` a two-line
`recv_timeout`, and leaves the sites with a second in-loop condition
(`tunnel.rs`, the clipboards after EOF, `acp/process.rs`'s mutexed child,
`ssh/mux/shared.rs`'s socket check) with one place for their deadline
arithmetic. `sheprd-client/src/limits.rs` is the natural home. This is the only
finding here that touches many files, and it is a pure simplification.

## 5. Animation clock compliance

Every animation module reads `motion::now()`: `sidebar/render.rs:127`,
`sidebar/rail.rs:106`, `sidebar/slide.rs:36,85,97`, `sidebar/device_drag.rs:197`,
`window/tab_drag.rs:334`, `menu/appear.rs:154,190`, `panel_resize/grip.rs:194`,
`window/toasts/motion.rs`, `window/zen.rs:146`, `window/tab_strip*.rs`,
`browser/group_motion.rs`, `chat/glide.rs`. Two near-misses are **not**
anomalies:

- `sidebar/hover.rs:88,101` with `window.rs:543` — the hover dwell. This is an
  input settling threshold, not motion, so the real clock is right; note only
  that `motion::now()` can step *backwards* when QA > Slow Animations is turned
  on, which would restart a dwell held in it.
- `notice.rs:60,85,130` and `window/toasts.rs:173-188` — expiry, which
  `flash.rs:82-84` establishes stays on the real clock while the fade uses
  `motion::now()`. Small improvement, **S**: `text()`/`is_some()` read the clock
  internally, so a caller cannot assert on a notice without waiting; pass `now`
  in as `text_at` already allows.

## 6. Prioritized conversions

1. **One wait helper** replacing the eleven copies of the deadline loop
   (`sheprd-client/src/limits.rs`, `M`). Removes 5–50 ms of latency per wait and
   puts the deadline arithmetic in one place, with no behavior change.
2. **`core_terminals/observation.rs:610-633`** to a channel with
   `recv_timeout`, and the 2 s machine re-read to a timer (`M`). Deletes a
   100 ms spin that runs for the life of the session.
3. **`control/client/local/processes.rs:117-139`** to a blocking receive on the
   watch (`S`). `spaceport processes` naps 100 ms per attempt while it waits up
   to 7 s (`:19`) for the first CPU sample.
4. **`sheprd-client/src/forward.rs:304`** to a notified master-state change
   (`M`). Ends a 4 Hz forever-loop per forward.
5. **Clock reads that buy nothing**: `transport/windows.rs:185` (drop the clock
   from the pipe name) and `notice.rs` (take `now` from the tick). Both `S`.
   Honorable mentions: `sound/playback.rs:65`, `daemon.rs:324`,
   `login_env.rs:243`.

## 7. Residual risks

1. **Turn elapsed is wall-clock arithmetic across a clock change.**
   `chat/view.rs:145` and `acp/src/tui/render.rs:182` compute
   `now_ms() - turn.started_ms`. A clock change mid-turn makes the shown elapsed
   jump; `saturating_sub` keeps it non-negative but not monotonic. Fixing it
   means carrying a local `Instant` for the live display alongside the shared
   stamp — worth doing only if the jump is ever observed.
2. **Every new monotonic stamp that may outlive a suspend needs the paired wall
   clock.** The two honest reference implementations are `endpoint/wake.rs` and
   `devices_overview/history.rs`; a future span measured with `Instant` alone
   will silently lose the night's sleep.
3. **Sleeps are latency, not correctness.** No site here runs on the UI thread
   (each is documented as blocking or reached from a worker/background
   executor: `window/clipboard.rs:20`, `menu/devices/setup.rs:176-178`,
   `listening_ports.rs:165-171`, and `connection.rs:186-187` for
   `daemon::connect`), so the conversions above are quality-of-implementation
   work, not bug fixes.
4. **Not verified on Windows or macOS hardware.** The platform-specific sites
   (`clipboard/{macos,windows}.rs`, `transport/windows.rs`, `socket/pipe.rs`)
   were read, not run; their sleeps and clock reads are gated on `cfg` branches
   this audit did not execute.