# tng8: retained-workflow acceptance run sheet

The executable plan for the 30 retained-workflow scenarios that no session has
ever run: **A03, A06–A09, A11–A19, A21–A28, A35–A39, A53–A55**. Every leg of
every one of them is `not_run` in
[`plan/acceptance.json`](../plan/acceptance.json). This document says, per
scenario, which command produces the evidence, which runtime it needs, what to
capture, what counts as a pass, and which leg(s) it records.

It is a plan, not a result. Nothing in it has been executed by this task, and
no acceptance status changes until a session records its own evidence files.

## 1. How to read a row

Each row names a **fixture** (a `just` recipe or an exact `cargo test` filter,
defined in §3), the **runtime** it drives, the **artifact** it must leave
behind, and the **pass rule**.

Runtime labels:

| Label | Meaning |
| --- | --- |
| `—` | no daemon and no agent: deterministic Rust tests, or CLI parsing |
| `h1` | a separately installed **Herdr 1** executable, selected explicitly (`HERDR_TEST_BINARY`) |
| `core` | a separately installed **herdr-core** (`hc`) executable, selected explicitly (`HERDR_CORE_TEST_BINARY`) |
| `h1+agent` | Herdr 1 plus a real host-side agent (`claude`, `codex`, …), for the rows the deterministic ACP peer cannot settle |

## 2. Legs: what #151 changed for the fixture

#151 (`feat/6k9q-legs`) made every scenario carry `legs.herdr1` and
`legs.core` independently, each `{status, evidence}`. `tools/spec.py` refuses a
scenario without both legs, refuses a leg with a status outside
`not_run|passed|failed|blocked`, and refuses `passed`/`failed` with an empty
`evidence` list. It does **not** inspect what an evidence file names, so the
attribution rule is a human one from the `leg_schema` text: *"Evidence naming
the other runtime stays on the leg that was recorded."*

Concretely, for this sheet:

1. **One leg is one file.** Each row writes `evidence/tng8/<ID>-herdr1.md`
   and/or `evidence/tng8/<ID>-core.md`. A single session that runs the same
   fixture under both runtimes still produces **two** records with two
   different logs. A Core log may never be cited on the `herdr1` leg.
2. **Same scenario, two harnesses.** The repository has no shared binary that
   runs both runtimes. `HERDR_TEST_BINARY` selects the Herdr 1 sandbox
   (`crates/test-support/sandbox.rs`); `HERDR_CORE_TEST_BINARY` selects the
   private Core sandbox (`crates/sheprd-core-client/src/tests/live.rs`). So
   "record twice" means: run the Herdr 1 fixture, then the Core fixture, then
   write one record per runtime — not one test run counted twice.
3. **Rows whose runtime work differs.** Some rows are runtime-agnostic code
   (input encoding, OSC 52 handling, control-target refusal). For those the
   fixture runs once with no daemon and the result is recorded on **both**
   legs only when the same code path provably serves both. Where it does not,
   the second leg stays `not_run` with the reason, never `passed`.
4. **Rows with no route on one runtime stay `not_run`/`blocked` there.**
   A15, A26 and A27 are Core-route gaps under D71 (`f48r`, `rnbn`, `dgw9`).
   A passing Herdr 1 leg never stands in for them.
5. **Fourteen legs already carry evidence but are `not_run`** (A09 herdr1,
   A13 core, A14 herdr1, A15 core, A16 herdr1, A23 herdr1, A24 herdr1,
   A25 herdr1, A26 herdr1, A27 core, A28 herdr1, A39 core, A53 herdr1,
   A54 herdr1). Those records are partial. This sheet's session finishes the
   scenario; it does not re-derive what is already recorded, but the new
   record must state which partial claims it retested.

## 3. Sessions

Six machine sessions, ordered so one sitting on one machine settles a whole
group, plus the `P1+agent` variant that the seven rows needing a real host-side
agent reuse. `H` is the only one that needs no desktop and no daemon.

| Session | Machine needed | Rows it settles (primary) |
| --- | --- | --- |
| `H` headless | any CI runner or laptop | A12 (part), A28 (part), A35 (part), A37, A38 (part) |
| `P1` private Herdr 1 | any host with a Herdr 1 binary on disk | A03, A06, A08, A23, A24, A35, and A14/A16/A19 with `P1+agent` |
| `P2` private Core | any host with a `hc`/herdr-core binary | A06, A07, A08, A09, A12, A13, A16, A18, A23, A25, A39, A53, A54 |
| `M` macOS desktop | unlocked macOS desktop | A11, A14, A21, A22, A28, A36, A55, and the A35 palette/shortcut cross-check |
| `W` Windows desktop | Windows runner (CI) or Windows desktop | A11, A21, A35, A36 |
| `L` Linux desktop | X11/Xvfb Linux host | A03, A09, A21, A22, A24, A53 |

### `H` — headless, no daemon

```sh
# H1  whole workspace, both feature sets: the CI gate most headless rows rest on
just test

# H2  Core credit/flow control, frame bounds, queue limits
just test-protocol

# H3  terminal input encoding: keys, kitty flags, mouse modes, bracketed paste
cargo test --locked -p sheprd-terminal

# H4  workspace document CAS/rebase/refusal, no live Core
cargo test --locked -p sheprd-gpui --bin spaceport \
  core_terminals::workspace_document::edit::tests -- --nocapture

# H5  control target matching and refusal
cargo test --locked -p sheprd-gpui --bin spaceport control::targets::tests

# H6  attach-only and stopped-Herdr lifecycle policy, scripted peers
cargo test --locked -p sheprd-gpui --bin spaceport endpoint::lifecycle_tests

# H7  local daemon launch policy: attach-only and cancelled never launch
cargo test --locked -p sheprd-gpui --bin spaceport daemon::tests

# H8  the app's CLI surface without a window, including removed commands
cargo test --locked -p sheprd-gpui --test cli
cargo test --locked -p sheprd-gpui --test removed_updater
just test-build
```

### `P1` — private Herdr 1

`HERDR_TEST_BINARY` must be an absolute path to an executable Herdr 1 binary;
the sandbox refuses anything else, clears the environment and isolates
HOME/XDG/config/socket.

```sh
# P1-a  live client fixtures: named session, stable endpoint, completion status,
#       plugin action projection
just test-live /abs/path/to/herdr

# P1-b  live client fixtures, one at a time, when a failure needs a single row
HERDR_TEST_BINARY=/abs/path/to/herdr cargo test --locked -p sheprd-client \
  --test live named_session_create_and_delete_cli_contract -- --ignored --nocapture
HERDR_TEST_BINARY=/abs/path/to/herdr cargo test --locked -p sheprd-client \
  --test live daemon_completion_status_live -- --ignored --nocapture

# P1-c  in-UI conversation start and terminal survival after an owned stop
HERDR_TEST_BINARY=/abs/path/to/herdr cargo test --locked -p sheprd-gpui \
  --features integration-test --test sparkle_lifecycle \
  failed_fixture_cleanup_stops_shell_without_pid_signals -- --ignored --nocapture
```

### `P2` — private Core

`HERDR_CORE_TEST_BINARY` must name an absolute herdr-core binary. The Core
sandbox uses a private home, a private `HC_SOCKET`, and stops only its own
core (`just test-core` ends that core's terminals and reaps its `serve` child).

```sh
# P2-a  the core-client live suite (lifecycle, observation, observer, resize,
#       termination, machines, remote terminal, remote document, file read)
just test-core /abs/path/to/herdr-core

# P2-b  host-adapter tests: needs the built adapter beside them
just build-release
HERDR_CORE_TEST_BINARY=/abs/path/to/herdr-core \
SHEPRD_ACP_TEST_BINARY="$PWD/target/release/sheprd-acp" \
  cargo test --locked -p sheprd-core-client --features adapter-live-test \
  --lib acp -- --ignored --nocapture

# P2-c  workspace document CRUD, rebase and membership against a live Core
HERDR_CORE_TEST_BINARY=/abs/path/to/herdr-core cargo test --locked \
  -p sheprd-gpui --bin spaceport core_terminals::workspace_document::edit::tests::live \
  -- --ignored --nocapture

# P2-d  workspace UI membership and conflict fencing against a live Core
HERDR_CORE_TEST_BINARY=/abs/path/to/herdr-core cargo test --locked \
  -p sheprd-gpui --bin spaceport core_terminals::workspace_ui::tests::live \
  -- --ignored --nocapture

# P2-e  document refetch after a notice and after hub/machine reconnect
HERDR_CORE_TEST_BINARY=/abs/path/to/herdr-core cargo test --locked \
  -p sheprd-gpui --bin spaceport core_terminals::observation::tests::live \
  -- --ignored --nocapture
```

### `M` — macOS desktop (one unlocked desktop, ordered)

Run top to bottom: the cheap no-daemon fixtures first, so a locked or busy
desktop fails before the expensive ones. This session is also the macOS row of
the native-acceptance checklist (`6q5m`); its log is the record `6q5m` needs.

```sh
# M1  rendering/controls, no daemon
just test-sidebar                 # native_sidebar
cargo test --locked -p sheprd-gpui --features integration-test --test live_gui \
  native_local_controls -- --ignored --exact --nocapture --test-threads=1

# M2  AppKit routing, works even while the screen is locked
just test-sidebar-routing

# M3  notifications, no daemon
cargo test --locked -p sheprd-gpui --features integration-test --test live_gui \
  native_notifications -- --ignored --exact --nocapture --test-threads=1

# M4  conversation-start presentation, no daemon
cargo test --locked -p sheprd-gpui --features integration-test --test live_gui \
  native_chat_start_errors -- --ignored --exact --nocapture --test-threads=1

# M5  failure paths exit without aborting
cargo test --locked -p sheprd-gpui --features integration-test --test live_gui \
  native_sidebar_probe_failure_exits_without_aborting -- --ignored --exact --nocapture --test-threads=1
cargo test --locked -p sheprd-gpui --features integration-test --test live_gui \
  native_sidebar_preferences_failure_exits_without_aborting -- --ignored --exact --nocapture --test-threads=1

# M6  a real window against a private Herdr 1, then quit/restart survival
just test-gui /abs/path/to/herdr
HERDR_TEST_BINARY=/abs/path/to/herdr cargo test --locked -p sheprd-gpui \
  --features integration-test --test sparkle_lifecycle \
  native_quit_restart_preserves_daemon_and_terminal -- --ignored --nocapture

# M7  release-mode paint budget
just test-perf 30

# M8  manual: one isolated profile for the rows no fixture covers
iso=$(mktemp -d /tmp/tng8-m.XXXXXX)
HOME="$iso/home" XDG_CONFIG_HOME="$iso/config" XDG_STATE_HOME="$iso/state" \
  mkdir -p "$iso/home" "$iso/config" "$iso/state"
HOME="$iso/home" XDG_CONFIG_HOME="$iso/config" XDG_STATE_HOME="$iso/state" \
  PATH="/abs/dir/of/herdr:$PATH" just run-debug
```

### `W` — Windows desktop

The repository has no Windows machine in this clone; the session runs on the
`windows-native` CI job (`windows-2025`, and `windows-11-arm` without
`native_gui_live`). Locally the same commands work, with the two environment
steps the job performs.

```sh
# W0  fixture acquisition (the CI job's own commands)
HERDR_TEST_BINARY="$(python scripts/install-herdr-windows.py "$TMP/herdr")"
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/set-display-resolution.ps1

# W1  every native GUI fixture, a real daemon, serial
cargo test --locked -p sheprd-gpui --features integration-test --test live_gui \
  -- --ignored --nocapture --test-threads=1 --skip native_gui_live   # ARM64 only

# W2  UI Automation tree and Ctrl-F6 regions
cargo test --locked -p sheprd-gpui --features integration-test \
  --test accessibility_windows \
  ui_automation_names_the_chrome_and_ctrl_f6_reaches_it -- --ignored --exact --nocapture

# W3  release-mode budget (behind the WINDOWS_ENABLED repository variable)
HERDR_PERF_P95_MS=100 cargo test --locked --release -p sheprd-gpui \
  --features integration-test --test performance_gui \
  dense_terminal_sidebar_performance -- --ignored --nocapture --exact
```

### `L` — Linux desktop

Two of the three Linux rows are already automated end to end by CI on a private
Xvfb display with a pinned, digest-checked Herdr; run them the same way
locally. The third (application menus, Wayland input shutdown) needs a real
Wayland/X11 desktop.

```sh
# L0  dependencies
bash scripts/install-linux-deps.sh

# L1  view-only restore across restarts (needs Xvfb + capture tools)
export SPACEPORT_SAVED_STATE_EVIDENCE="$PWD/docs/spaceport/evidence/tng8/_raw/saved-state"
export HERDR_TEST_TMPDIR=/tmp
export HERDR_TEST_BINARY=/abs/path/to/herdr-linux-x86_64     # pinned v0.9.5
cargo test --locked -p sheprd-gpui --all-features --test live_gui \
  saved_state::recovery::native_saved_state_recovery -- --ignored --exact --nocapture --test-threads=1

# L2  intentional Herdr shutdown, stopped card, explicit start action
export SPACEPORT_SHUTDOWN_EVIDENCE="$PWD/docs/spaceport/evidence/tng8/_raw/shutdown"
cargo test --locked -p sheprd-gpui --all-features --test live_gui \
  shutdown::native_local_shutdown_and_reconnect -- --ignored --exact --nocapture --test-threads=1

# L3  GUI-only menus and Wayland input shutdown (real desktop only)
cargo test --locked -p sheprd-gpui --features integration-test --test live_gui \
  native_application_menus -- --ignored --exact --nocapture --test-threads=1
WAYLAND_DISPLAY=wayland-0 cargo test --locked -p sheprd-gpui --features integration-test \
  --test live_gui native_input_shutdown_terminal -- --ignored --exact --nocapture
```

### `P1+agent` — a private daemon and a real agent

Only the rows whose pass rule says "the same agent/conversation" need an agent
the repository does not ship. The deterministic peer
(`crates/sheprd-core-client/tests/fixtures/acp_peer.py`) settles the protocol
rows; it does **not** settle "a real agent resumed its conversation".

```sh
# A1  a private Herdr 1 with one built-in agent installed on PATH
iso=$(mktemp -d /tmp/tng8-a.XXXXXX)
HOME="$iso/home" XDG_CONFIG_HOME="$iso/config" XDG_STATE_HOME="$iso/state" \
  mkdir -p "$iso/home" "$iso/config" "$iso/state"
HOME="$iso/home" XDG_CONFIG_HOME="$iso/config" XDG_STATE_HOME="$iso/state" \
  PATH="/abs/dir/of/herdr:$PATH" just run-debug        # then: spaceport chat new claude
```

## 4. Rows

Coordinates in the fixture column are `session·command`; a second coordinate is
a supporting run. `legs` says which records the row must produce.

### 4.1 Headless and private-runtime rows

| ID | Title | Proves | Fixture | Runtime | Artifact | Pass rule | Legs |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A03 | Installed runtime startup | Connect first; start an installed local runtime only when necessary; an explicit socket never starts one; no download/install/update hook runs | `H7`, `H6`, `P1·P1-a`, `L·L2`, plus `M·M6`; Core side `P2·P2-a` plus `cargo test --locked -p sheprd-core-client --lib tests::start` | `—`, `h1`, `core` | `evidence/tng8/A03-herdr1.md` citing the `H7`/`L2` logs and `started`-absent assertion; `evidence/tng8/A03-core.md` citing the private-core start/lifecycle log | `explicit_and_remote_targets_never_start_local_daemon`, `attach_only_and_cancelled_do_not_launch` and `explicit_socket_shutdown_keeps_attach_only_retry` pass; the shutdown fixture's fake `herdr` on `PATH` leaves no `started` file and the window holds "Herdr stopped" for the full retry ceiling before the start button reconnects | herdr1, core |
| A06 | Workspace document CRUD | Create/rename/archive/delete a presentation workspace through the namespaced Core document; no process kill, file deletion or other-namespace write | `P2·P2-c` (`create_rename_archive_delete_only_the_grouping_document`), supporting `H4`; herdr1 side `P1·P1-a` + `M·M6` native workspace controls | `core`, `h1` | `evidence/tng8/A06-core.md` + log; `evidence/tng8/A06-herdr1.md` + screenshot of the Herdr 1 workspace row after archive | Core: the live test passes and the fixture terminal's PID is unchanged after archive and delete. Herdr 1: the native action changes only presentation grouping — daemon keeps the pane process (`spaceport processes` before/after, same PID) | herdr1, core |
| A07 | Concurrent workspace edits | Two clients editing from one version; a stale write conflicts and a safe rebase preserves the other writer | `P2·P2-c` (`disjoint_conflict_rebases_and_same_target_conflict_refuses`), `P2·P2-d` (`ui_membership_rebases_disjoint_client_edit_without_touching_terminal`), supporting `H4` | `core` | `evidence/tng8/A07-core.md`, logs from both P2 runs | Both live tests pass: the disjoint edit rebases to a new version with the other writer intact, and the same-group edit is refused with a typed conflict rather than last-writer-wins | core |
| A08 | Partial launch and membership | A launch that succeeds while the document write conflicts reports the partial outcome and exposes the ungrouped terminal without rolling the process back | `P2·P2-d` (`ui_refuses_same_group_conflict_and_fences_an_uncertain_version`), `P2·P2-c` (`membership_edits_only_the_private_core_document`), plus one manual step in `M·M8` | `core`, `h1+agent` | `evidence/tng8/A08-core.md`; `evidence/tng8/A08-herdr1.md` with the window screenshot and `spaceport window list --json` | Core: the conflict is reported while the fixture terminal stays alive and ungrouped. Herdr 1: the same conflict path in a real window shows the terminal outside the group and the process still running | herdr1, core |
| A09 | View-only restore | Quit and reopen with surviving, missing and renamed references: fresh state wins, survivors reattach, missing agents never restart | `L·L1` (`native_saved_state_recovery`), supporting `P2·P2-e` | `h1` (Linux), `core` (document refetch) | `evidence/tng8/A09-herdr1.md` citing `phase-{a,b,c}-window.json`, `phase-{a,b,c}-exit.json`, the five `*.sha256`, `repair-{640,900}.png`, `rejected-tabs.bin`, `backup.json`, `herdr-events.txt`, `cleanup.txt`, `complete.txt`; `evidence/tng8/A09-core.md` | The fixture passes and `complete.txt` exists; no phase launches an absent agent (its receipts show only the surviving terminals), and the rejected saved state is preserved rather than rewritten | herdr1, core |
| A11 | Keyboard and mouse | Input bytes follow the terminal's actual modes with no lost or doubled submission | `H3` (`keys`, `kitty`, `mouse`, `paste`), then manual IME/composition step in `M·M8` | `—` | `evidence/tng8/A11-herdr1.md` and `A11-core.md` with the `-p sheprd-terminal` log plus an IME transcript (what was typed, what the fixture program printed) | Every encoding test passes; the manual step shows one composed character, one bracketed paste and one mouse report reaching the program exactly once per event | herdr1, core |
| A12 | Attach flow control | Flooding one terminal and stalling one view keeps queues bounded, credits correct, and other streams and control responsive | `H2` (`stream_bytes_that_follow_the_answer_are_kept_and_credited`, `confirmed_observer_keeps_resize_before_later_data_and_credits_only_bytes`, `PastCredit` refusal), `P2·P2-a` (`real_core_late_streams_close_without_ending_the_terminal`) | `core` | `evidence/tng8/A12-core.md` and the `just test-protocol` log | Bounds and credit arithmetic hold in both suites; the live Core case closes the late stream without ending the terminal; no test reports an unbounded queue | core (the herdr1 leg stays `not_run`: Herdr 1 has no credit flow control, and no Herdr 1 flood fixture exists) |
| A13 | Observer isolation | Observers never type or seize size, and a confirmed RESIZE precedes later paint and data | `P2·P2-a` (`observer_input_and_resize_leave_the_program_for_its_active_view`, `a_private_core_orders_observer_resize_before_replacement_paint`), supporting `H2` | `core` | `evidence/tng8/A13-core.md` + logs (the leg already carries `t05-core-observer.md` and `t05-core-lifecycle.md`; the new file states which of their claims it retested) | Both live tests pass: observer keystrokes do not reach the program, observer-driven resize does not change its size, and the resize confirmation ordering assertion holds | core |
| A14 | Local native chat | Attaching to an existing runtime-owned local adapter persists the same agent and conversation across view switches and a quit/reopen, with no desktop-owned agent process and no automatic resume | `P2·P2-b` (`a_core_owned_adapter_survives_all_clients_detaching`), `M·M4`, `M·M6`, then `P1+agent·A1` | `core`, `h1+agent` | `evidence/tng8/A14-core.md` (private-Core adapter PID before/after detach); `evidence/tng8/A14-herdr1.md` with the chat transcript digest and `spaceport chat status --json` before and after quit | Core: the adapter PID is identical before and after all clients detach and the transcript continues. Herdr 1: the same conversation survives a view switch and a quit/reopen, the adapter is a terminal under the daemon (not a child of Spaceport), and `spaceport chat status` reports it | herdr1, core |
| A15 | Remote native chat gate | A verified supported remote native-chat route exists, or the requirement stays blocked rather than satisfied by terminal fallback | No fixture exists: the blocker is a Herdr-owned authenticated duplex route. Re-audit `t06-core-duplex-route-audit.md` against the installed Core method table and record the answer | `core` | `evidence/tng8/A15-core.md` — a method-table audit, not a run | Pass only if an authenticated non-PTY duplex route is found **and** an end-to-end fixture passes. Otherwise the record says `blocked` with the audit and names `f48r` | core (blocked) |
| A16 | Concurrent permission response | Two views answering one pending request resolve to one source-defined outcome; expired answers are rejected; descriptive tool text cannot grant consent | `P2·P2-b` (`a_core_owned_adapter_shares_one_permission_result`), supporting `H1`; herdr1 half already recorded in `t06-adapter-detach-process.md` and must be retested in `P1+agent·A1` | `core`, `h1+agent` | `evidence/tng8/A16-core.md` + log; `evidence/tng8/A16-herdr1.md` with both views' answers and the resulting transcript | Core: the second answer is rejected with a typed rejection and the transcript records exactly one decision. Herdr 1: the same, plus a tool-output block that looks like an approval produces no approval | herdr1, core |
| A17 | Prompt uncertainty | A disconnect after submission but before acknowledgement never replays the prompt, and the acknowledgement/observed/unknown states stay truthful | `H1`: `cargo test --locked -p sheprd-gpui --bin spaceport chat::tests::delivery` (`a_failed_write_is_unknown_and_is_never_replayed`) and `chat::tests::session`; then the live disconnect step in `P1+agent·A1` | `—`, `h1+agent` | `evidence/tng8/A17-herdr1.md` with the scripted-peer log and the live transcript showing one submission | No automatic replay appears on reconnect; the CLI and UI both show the prompt as submitted-but-unconfirmed, then resolved or unknown — never as "the agent ran it" | herdr1, core |
| A18 | Unknown versus idle | Programs that never report, report done but stay alive, and become unreachable stay distinct as unknown / reported / process / reachability | `P2·P2-a` (`reported_status_and_unreported_process_are_distinct`), supporting `P1·P1-a` (`daemon_completion_status_live`) | `core`, `h1` | `evidence/tng8/A18-core.md`, `evidence/tng8/A18-herdr1.md` with `spaceport status --json` for each of the three programs | Each of the three programs maps to a different reported state; no unreported program renders as idle and no reported-done-but-alive program renders as finished | herdr1, core |
| A19 | Timeline scope | The retained timeline is preserved with honest search coverage and no new history store or global index | `H1` (`cargo test -p sheprd-gpui --bin spaceport chat::tests::timeline`), plus `P1+agent·A1` to walk the timeline against a real conversation | `—`, `h1+agent` | `evidence/tng8/A19-herdr1.md` with the timeline search transcript and a source check that no index/history store exists | Timeline navigation and search behave as the retained implementation; search reports only what the loaded transcript contains, and no file/terminal index or history database is created | herdr1, core |
| A21 | Desktop notifications | Notifications, badges and sounds follow retained preferences per workspace and agent, do not duplicate, and stop when the app exits | `M·M3` (`native_notifications`), `W·W1` (same fixture on Windows), `L·L3` if a Linux desktop is available; delivery check in `M·M8` | `—` | `evidence/tng8/A21-herdr1.md` with the fixture log plus screenshots of one banner and one badge and the `NOTIFICATIONS native PASS:` marker per platform | Fixture passes per platform; the manual pass shows one notification per event, honouring a per-workspace mute, and none after the app exits | herdr1, core |
| A22 | Last-window lifecycle | Closing the last window quits Spaceport while the runtime and its terminals keep running | `L·L2` (`native_local_shutdown_and_reconnect` quit step), `M·M6` (`native_quit_restart_preserves_daemon_and_terminal`) | `h1` | `evidence/tng8/A22-herdr1.md` with both logs and the terminal heartbeat/PID recorded before and after quit | The GUI exits with a success status, the daemon PID and the terminal PID are unchanged, and a freshly attached client still reaches that terminal after the app is gone | herdr1, core |
| A23 | Explicit termination | With duplicate labels and several views, terminating one qualified fixture terminal ends only that program | `P2·P2-a` (`killing_one_address_keeps_same_labeled_terminal_and_other_views`), `P1·P1-a`; the herdr1 leg already cites `t05-explicit-termination.md` but needs a native action | `core`, `h1` | `evidence/tng8/A23-core.md`; `evidence/tng8/A23-herdr1.md` with `spaceport processes --json` for both panes before and after | Core: the selected address reports finished while the same-labelled sibling keeps its PID and its observer stream. Herdr 1: the native terminate action ends exactly the chosen pane and the sibling's PID is unchanged | herdr1, core |
| A24 | Offline behavior | After a disconnect no mutation is queued, nothing stale is presented as live, and no persistent transcript exists | `H6` (`explicit_socket_shutdown_keeps_attach_only_retry` and the outage/status assertions), `L·L2` stopped-card window, `P2·P2-a` (`a_terminal_survives_detach_and_another_client_reattaches`) | `—`, `h1`, `core` | `evidence/tng8/A24-herdr1.md` with the stopped-card screenshot/JSON and a saved-file listing; `evidence/tng8/A24-core.md` | The window shows the outage reason and empty content rather than stale data, a mutation attempted while stopped is refused rather than queued (CLI exit code recorded in the same log), the restarted client shows fresh authoritative state, and no transcript file exists under the state directory | herdr1, core |
| A25 | Attachments | Bytes reach the intended machine and "transferred" stays distinct from "submitted as a path" | `P2·P2-a` (`a_real_core_stages_distinct_binary_files_for_one_terminal`), supporting `cargo test -p sheprd-core-client put`; drag/drop and clipboard-image steps in `M·M8` | `core`, `h1` | `evidence/tng8/A25-core.md` with the two staged paths and the recovered bytes; `evidence/tng8/A25-herdr1.md` with the target path and the terminal transcript | The staged bytes round-trip unchanged to the target machine and nothing is typed into the terminal by the transfer. The drag/drop step shows the file arriving on the intended host, with the path only appearing in the terminal if the user submits it | herdr1, core |
| A26 | General binary transfer gate | Retained upload/download, overwrite, cancellation, symlink and temp-file behaviour is demonstrated through a verified route, or the gap stays open | `P2·P2-a` (`a_real_core_reads_only_text_from_the_renamed_machine_id`), plus the case table in `t07-file-route.md`; Herdr 1 transfer path in `M·M8` | `core`, `h1` | `evidence/tng8/A26-core.md` — the case table with each case marked reachable/unreachable and the reason; `evidence/tng8/A26-herdr1.md` | Herdr 1 passes its own upload/download/overwrite/cancel cases through the retained route. Core records `blocked` naming `rnbn`/G-FILE for every case with no route; no text-only API is claimed as full transfer | herdr1, core (blocked) |
| A27 | Loopback service forwarding gate | A service bound only to a fixture remote loopback is reachable in the system browser without public rebinding | `M·M8`: `spaceport ports forward PORT` against a fixture SSH host, then `spaceport ports open PORT`; Core side is an audit only | `h1`, `core` | `evidence/tng8/A27-herdr1.md` with the forward log, `lsof -nP -iTCP:PORT` showing a loopback-only bind, and a browser screenshot; `evidence/tng8/A27-core.md` — audit | Herdr 1: the port is reachable at `127.0.0.1:PORT` only, no public listener appears, and the forward is an existing Herdr 1/SSH route. Core: `blocked` naming `dgw9`/G-PORT | herdr1, core (blocked) |
| A28 | External-only browsing | URLs, services and files open in the system handler while embedded browsers, annotations and webviews are absent | `H8` (`browser_commands_are_rejected_before_socket_lookup`, `code_open_is_rejected_before_socket_lookup`, `code_edit_is_rejected_before_socket_lookup`, `removed_design_feedback_is_rejected_before_socket_lookup`), `just audit-deps`, then a native open in `M·M8` | `—` | `evidence/tng8/A28-herdr1.md` with the CLI log, the system-browser screenshot, and the dependency/source audit | Every removed command exits before socket lookup; a URL opens the system browser (not an in-app view); the dependency graph and source tree contain no embedded browser engine, as `t04-native-browser-engine-removal.md` records | herdr1, core |
| A35 | Unified command semantics | GUI, palette, shortcut and CLI share one catalog and dispatch contract, so effects, confirmations and errors agree | `H1` (`cargo test --locked -p sheprd-gpui --bin spaceport cli::tests` and `-p sheprd-commands`), `P1·P1-a` (live CLI), then the palette/shortcut cross-check in `M·M8` and `W·W1` | `—`, `h1` | `evidence/tng8/A35-herdr1.md` with the catalog test log and a table of four invocation paths for the same command with their identical results | The same command reaches the same typed request from all four paths, and a refusal (bad target, ambiguous name) produces the same exit code and message from CLI and dialog | herdr1, core |
| A36 | Ambiguous and stale target | Automated mutations never guess or redirect; a stale or ambiguous target is refused | `H5` (`control_target_refuses_unrelated_daemon`), then a live step in `M·M8` and `W·W1` (rename a host between two windows and act) | `—`, `h1` | `evidence/tng8/A36-herdr1.md` with the test log and the refusal output for a renamed/duplicate-label target | The unrelated-daemon request is refused; a dispatch whose target was renamed or reordered between windows is refused rather than redirected; no request falls back to keyboard focus or a display label | herdr1, core |
| A37 | Timeout is not failure | A lost reply to a launch or other non-idempotent mutation shows an unknown outcome and reconciles before any new action | `H1`: `cargo test --locked -p sheprd-gpui --bin spaceport chat::tests::delivery` (unknown-and-never-replayed cases), `chat::tests::session`, and `core_terminals::workspace_document::edit::tests` (fenced `WriteUnknown` cases); then `P2·P2-d` | `—`, `core` | `evidence/tng8/A37-core.md` with the scripted-peer log and the fenced-version assertion | An unanswered request is reported as outcome-unknown in both CLI and UI; the client reconciles before the next mutation; no duplicate launch happens and no local request ID is presented as backend idempotency | herdr1, core |
| A38 | Content is not authority | Terminal text, tool output, labels, filenames and links grant no authority, and a fake approval request executes nothing | `H1`: `cargo test --locked -p sheprd-osc52` (`rejects_non_utf8_and_oversized_payloads`, `rejects_malformed_and_noncanonical_payloads`), `cargo test --locked -p sheprd-gpui --bin spaceport control::` and `window::clipboard`, then a live malicious-content step in `M·M8` | `—` | `evidence/tng8/A38-herdr1.md` with the test log and the pasted-content transcript showing nothing executed | Only bounded UTF-8 OSC 52 text reaches the pasteboard; no escape, path, URL or approval-looking tool text triggers execution, a credential fetch, an approval, unexpected file/network access, or a command-catalog change | herdr1, core |
| A39 | Bounded inspection | A large session's default inspection stays paged and bounded, with no full transcript fetch and no model invocation | `H1` (`cargo test -p sheprd-gpui --bin spaceport processes::`, `listening_ports::`, `port_forward::tests`), `P2·P2-a` for the Core catalog/status path | `—`, `core` | `evidence/tng8/A39-core.md` with the bounded-output assertions and a `spaceport processes --json` capture on a large fixture session | Output is paged/truncated with its coverage stated, the request never fetches a full transcript, and no model/embedding/summariser call is made on the default path | herdr1, core |
| A53 | Event epoch and document refresh | A runtime restart and a document change while disconnected both replace stale projections; old sequence numbers do not suppress new events | `P2·P2-a` (`remote_document_reloads_by_core_id_after_hub_and_machine_reconnect`), `P2·P2-e` (`selected_remote_document_refetches_after_notice_and_reconnect`, `worker_refetches_local_document_after_notice`), `L·L1` for the saved-state half | `core`, `h1` | `evidence/tng8/A53-core.md` with the three logs; `evidence/tng8/A53-herdr1.md` citing the saved-state phase receipts | The restarted hub reports `missed`, the document is re-read through the unchanged machine ID to the newer version, and the post-restart event is delivered rather than suppressed by an older sequence | herdr1, core |
| A54 | No automatic conversion/resume | Showing or hiding native chat, or restoring an absent adapter, kills, restarts, resumes or deploys nothing; a missing capability stays explicit | `P2·P2-b` (`a_core_owned_adapter_survives_all_clients_detaching`), `M·M4`, plus the removal audit recorded in `t04-teleport-engine-removal.md` and `t06-chat-lifecycle-policy-removal.md` | `core`, `h1+agent` | `evidence/tng8/A54-core.md` and `evidence/tng8/A54-herdr1.md` with the adapter PID set before and after show/hide and the missing-adapter message | No show/hide or restore changes the adapter or agent PID set, no conversion/recovery path is reachable in the binary, and an absent adapter produces an explicit message instead of a silent restart | herdr1, core |
| A55 | Supported versus unsupported controls | Prompt, input and permission controls appear only where the session advertises them, with no duplicated provider configuration or runtime CLI | `H1` (`cargo test -p sheprd-gpui --bin spaceport chat::tests::` capability cases), then `P1+agent·A1` with two agents advertising different capabilities and one native pass in `M·M8` | `—`, `h1+agent` | `evidence/tng8/A55-herdr1.md` with the two sessions' capability output and screenshots of the differing control sets | Each session shows exactly the controls its advertisement supports; the unsupported one shows no prompt/permission affordance; no provider/model configuration panel or cloned runtime CLI is reachable | herdr1, core |

### 4.2 Desktop-only rows folded into the native sessions

These are the rows that only an unlocked desktop can settle, including the
`6q5m` (macOS), `h8g5` (Windows) and `r844` (Linux) checklists. They are in the
table above; this list says which session must own the record.

| Session | Rows it must record | Also satisfies |
| --- | --- | --- |
| `M` macOS | A11, A14, A21, A22, A28, A35, A36, A55 | `6q5m` native macOS acceptance row, and the Dock/About check carried by `r69x` |
| `W` Windows | A11, A21, A35, A36 | `h8g5` native Windows acceptance row |
| `L` Linux | A03, A09, A21, A22, A24, A53 | `r844` native Linux acceptance row |
| `a5ks` qualification | A36, A37, A38, A39 above, plus A34/A40/A41/A46/A52 | A36–A39 rows here are the ones `a5ks` needed |

## 5. Where artifacts go

```
docs/spaceport/evidence/tng8/
  A<ID>-herdr1.md          # one record per leg; this is what acceptance.json cites
  A<ID>-core.md
  logs/                    # raw command output, one file per run
  json/                    # spaceport CLI --json captures
  images/                  # screenshots, one per named state
  _raw/                    # unedited fixture dumps (saved-state, shutdown), copied verbatim
```

`acceptance.json` then records, for example:

```json
{
  "id": "A03",
  "legs": {
    "herdr1": { "status": "passed", "evidence": ["evidence/tng8/A03-herdr1.md"] },
    "core":   { "status": "passed", "evidence": ["evidence/tng8/A03-core.md"] }
  }
}
```

Each record must state: the exact commit, the exact runtime binary and its
digest or version, the command that ran, the machine and OS, which partial
claims it retested, and — explicitly — what it does not prove. That last
sentence is what keeps a leg honest, as in
[`t05-explicit-termination.md`](t05-explicit-termination.md).

## 6. What is genuinely blocked

| Row(s) | Blocked by | What would unblock it |
| --- | --- | --- |
| A15 core | No authenticated non-PTY duplex route in herdr-core (`f48r`, G-CHAT) | A Herdr-owned route plus an end-to-end fixture; until then the Core leg records `blocked` |
| A26 core | G-FILE: no binary transfer route; `file.read` is UTF-8-only and `file.write`/`file.edit` follow symlinks with no overwrite precondition (`rnbn`) | A Core transfer contract with overwrite/cancel semantics |
| A27 core | G-PORT: no Core forwarding method (`dgw9`) | A Core forwarding contract with owner and lifetime |
| A21, A35, A36 on `W` | No Windows machine in this clone; `windows-native` runs only for PRs on `2389-research/spaceport`, and requires the `harperreed` actor for `push`/`dispatch` | One PR from the repository (not a fork) runs `windows-native`; `W3` additionally needs `WINDOWS_ENABLED=true` |
| A11, A35, A36 on `L`, A21 on `L` | `native_application_menus` and the Wayland input-shutdown fixtures need a real Wayland/X11 desktop; no CI job runs them | A borrowed Linux desktop; `L1`/`L2` already run on CI |
| All `M` rows | An unlocked macOS desktop; past attempts recorded `CGSSessionScreenIsLocked=Yes` (`t11-native-locked-desktop-attempt.md`) | One sitting at an unlocked desktop — `M2` still works while locked, so routing never blocks the session |
| A14, A16, A17, A19, A54, A55 herdr1 halves | No automated fixture starts a real host-side agent; only the deterministic `acp_peer.py` exists | `P1+agent·A1`: a private Herdr 1 with one built-in agent on `PATH` |

## 7. What one operator can execute today

Two sittings, no extra hardware beyond the binaries the repository already
documents:

1. **Anywhere (headless).** `H1`–`H8`, `P2·P2-a`–`P2·P2-e`, `P1·P1-a`–`P1·P1-c`.
   That is the full evidence for A37, A38, A39, most of A03/A06/A07/A08/A12/A13/
   A16/A18/A23/A53, and the supporting half of the rest. It needs a Herdr 1
   binary and a herdr-core binary on disk and nothing else.
2. **One unlocked macOS desktop.** `M1`–`M7` in that order, then the `M8`
   manual steps. That closes A11, A21, A22, A28 and the palette/shortcut and
   refusal halves of A35/A36/A55, and produces the `6q5m` record.

One Linux desktop with Xvfb adds A09 and the A03/A24 stopped-card evidence
(`L1`, `L2`) — and CI already runs those two commands on the `ubuntu-24.04`
all-features leg of every code-changing PR, so their logs are obtainable
without owning the machine. Everything else needs either a
Windows runner or an owner decision on the three Core contracts.
