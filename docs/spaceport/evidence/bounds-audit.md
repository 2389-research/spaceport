# Control-socket protocol bounds and queue discipline audit (wx8)

Intended commit path: `docs/spaceport/evidence/bounds-audit.md`.

## Scope and method

Audited the ten invariants named in `AGENTS.md` → *Protocol And Safety*, looking for
damage from the recent extraction waves (#133 settings, #143 browser crates, #140 render
split, #152 work-family removal) plus any bound the extraction added that is not actually
enforced.

**Read base.** This worktree, `/Users/clint/code/worktrees/spaceport/pi-worktree-93a1dcb5-...-s0-0`
(content identical in tree shape to `/Users/clint/code/spaceport`).

**Tooling limitation, stated plainly.** This run had no shell tool: `git fetch origin
main`, `git log`, `cargo test`, and `git commit` were **not** run, so this report is
based on the worktree's checked-out content, not a verified `origin/main` tree, and I
made **no commits**. Every claim below is source evidence with `file:line`; no claim
rests on a test I ran.

**Task-brief assumptions that do not match the tree** (the brief, not the code, is wrong
on these):

- There is no `sheprd-browser` crate. The vendored browser code is
  `crates/sheprd-gpui/src/browser/` (`browser.rs` + `browser/` submodules); its bounded
  store is `browser/store.rs`. #143 did not create a browser crate.
- There is no `sheprd-osc52` crate. The OSC 52 decoder is `crates/sheprd-gpui/src/osc52.rs`.
- The control protocol version is **37**, not 38 (`protocol.rs:42`).

## Verdict table

| # | Invariant | Verdict | Evidence | Notes |
|---|---|---|---|---|
| 1 | Bounded request/response sizes and frame limits, enforced on both ends | **holds** | `sheprd-control-protocol/src/protocol.rs:45` (`MAX_MESSAGE = 256 * 1024`), `:47` (`MAX_WAIT_SECONDS = 600`); server inbound `socket.rs:197-206`; server outbound `socket.rs:276-292`; client inbound `socket.rs:302-311` | Server and CLI share one `read_line`, which takes `MAX_MESSAGE + 1` and errors `MessageSize` past the bound. A reply that does not fit is replaced by an error, never truncated mid-line (`socket.rs:280-292`). Windows pipe hint sized to the same bound (`socket/pipe.rs:84`). Tests: `socket/tests/framing.rs:11,18,45`. |
| 2 | Strict decoding and atomic surface patch validation | **holds** | `protocol.rs:48-52` (`#[serde(tag = "method", deny_unknown_fields)]`), `:62-63`, `:74-75`; `sheprd-protocol/src/frame.rs:7-25`, `:38-74`, `:76-89`; `surface_scroll.rs:146+` | `apply_patch` = `validate_patch` then `commit_patch`; every reject path returns before any mutation. Patch identity checks boot id, projection revision, base revision and the `+1` surface revision, and refuses a patch over a popup (`frame.rs:39-49`). Row, hyperlink, pane-geometry and cursor bounds checked before commit. Scroll patches route through the same validate/commit pair. |
| 3 | Single in-flight API request; request correlation preserved | **holds** | `sheprd-client/src/session.rs:451-459`; `session/barrier.rs:120-135`, `:49-53` | One lease: a command carrying `request` is re-queued and the batch breaks while `session.pending` or `session.barrier` is set, unless it is the one pipelined surface activation behind a synchronous focus request. Correlation is checked on the assembled response (`response["id"] != request_id` → `Error::ResponseId`), and a response with no matching pending request is `Error::UnsolicitedResponse`. |
| 4 | Bounded command and event queues, FIFO, coalesced latest-state mailbox | **holds** | `sheprd-client/src/limits.rs:6-7`; `sheprd-client/src/event.rs:48-73`; `sheprd-gpui/src/connection.rs:33,240-275,309-345`; `sheprd-gpui/src/state.rs:659-690`; `sheprd-control-protocol/src/socket.rs:35,37`; `sheprd-gpui/src/control/server.rs:23` | Commands: `bounded(64)` (`connect.rs:67`), `try_send` → `Error::Full` with a `command_queue` backpressure log. Events: `bounded(8)`, `send_timeout(POLL)` never blocks unbounded and never drops silently. GPUI side is a single `Arc<Mutex<LiveState>>` the reader mutates in place and the window takes with `try_lock` — no unbounded UI event queue. Per-item caps stay in place: sound `MAX_PENDING`, notifications `PENDING_LIMIT`, clipboard `osc52::MAX_PENDING`, bells saturating add. Control socket: 4 queued requests, 8 connections, 16 parked watches. |
| 5 | Cancellation never flushes queued work or joins a worker on the UI thread | **holds** | `sheprd-client/src/handle.rs:149-153`; `session.rs:326`; `event.rs:54-58`; `sheprd-control-protocol/src/socket.rs:127-130`; tests `sheprd-client/src/tests/commands.rs:146-166,90-97`, `tests/partial_input.rs:482-495` | `disconnect()` only stores `stop` and wakes; the worker loop leaves without draining, and a blocked `deliver` returns `Error::Cancelled` on the next poll. Control-socket worker threads are documented as never joined (`socket.rs:127-130`). Regression tests: cancellation behind a pending request sends nothing further and emits no events; a full event queue interrupts. |
| 6 | Partial frame prefixes survive read timeouts; inbound partial frame finished first | **holds** | `sheprd-client/src/frame.rs:140-160`, `:163-172`, `:158-190`; `session.rs:437-446`; tests `tests/partial_input.rs:457-476`, `tests/framing.rs:37` | `WouldBlock`/`TimedOut`/`Interrupted` return `Ok(None)` and keep `bytes`; framing resumes at `target`. Length prefix bounded `0 < len <= MAX_GRAPHICS_FRAME_SIZE` (`protocol/src/codec.rs:9`). A second write is deferred while `reader.started.is_some()` unless the command is partial input that is permitted to interleave, and an in-flight API request keeps that barrier until the frame completes. `partial_frame` timeout turns a stalled peer into a typed error (`frame.rs:121-129`). |
| 7 | Receiver clones compete, never broadcast; no accidental fan-out | **holds** | `sheprd-client/src/handle.rs:25` (single `Receiver<ClientEvent>`), `connection.rs:244` (one reader loop consuming it) | Searched the whole workspace for `Receiver` construction and `.clone()` on receivers: `Client::events` is the only `Receiver<ClientEvent>` and is moved into the single bridge reader thread. `sheprd-control-protocol`'s `Server::requests` likewise has one consumer (`drain`/`next`). No fan-out was introduced. |
| 8 | Terminal content untrusted; only OSC 52 clipboard honored, bounded UTF-8 | **holds** | `sheprd-gpui/src/osc52.rs:14,18,27-31,35-40,71-79`; `state.rs:684-695`; `state.rs:736-745` | Decoder rejects any non-alphabet byte, data after padding, non-canonical trailing bits, invalid UTF-8, and a payload past `MAX_TEXT_BYTES` (2 MiB), and refuses an oversized payload before allocating. Only decoded text is queued, at most 8 pending, and the queue drops its head on overflow. Every other exotic `ServerMessage` variant is explicitly ignored, including `GraphicsFile`, whose daemon-named path is never followed. |
| 9 | #152 work family gone: no old method ids; wire contract intact; `Method` has no serde order | **holds** (with a version note) | `sheprd-control-protocol/src/protocol/work.rs:5-18`; `work/tests/excluded_fan_out.rs`; `work/tests.rs:18-32`; `work/tests/removed_checkpoint.rs`, `removed_teleport.rs`; `sheprd-client/src/method.rs:16-37`; `protocol.rs:42` | `WorkOp` is exactly `Notes/Note/SetNote/MuteStatus/Mute` under `deny_unknown_fields`; negative tests assert `fan_out`, `script`, `restore`, `launch`, `set_note` without text are all rejected. No Rust source references the removed ids (only marketing site text, removal-evidence docs, and the absence-check scripts). `Method` derives no serde at all — the enum plus `as_str` is the only spelling path — so there is no bincode/serde ordering contract to break. **Version is 37, not 38** as the brief assumed; the 37 doc block (`protocol.rs:14-41`) matches the removals recorded above. |
| 10 | New bounds from extraction are actually enforced | **holds, one gap** | `sheprd-gpui/src/browser/store.rs:15-17,391-392`; `browser/store/tests.rs:159-176`; `browser/layouts.rs:11-12`; `state_file.rs:17-31`; `config/daemon.rs:12,52`; **`sheprd-client/src/settings_files.rs:83-88`** | Browser store: `MAX_TABS = 256` enforced on both insertion paths (`open` and `restore_tab`), `MAX_FILE_BYTES = 1 MiB` on load, title truncated to 200 chars — the bound test asserts the 257th open returns `None`. `state_file::read` reads `limit + 1` and errors `StateFileSize` past it, and every app-owned JSON state file passes an explicit limit. **Gap:** the settings extension point `settings_files::read` is an unbounded `fs::read_to_string` — see Finding F2. |

## Findings

### F1 — Orphaned, uncompiled control-server test file (dead code, no bound impact)

`crates/sheprd-gpui/src/control/server/browser/tests.rs` exists, but
`control/server.rs:5-12` declares no `mod browser;`, so the file is not part of the
crate. It would not even compile if wired: it uses `super::*` for a `pastable` helper
that exists nowhere in the crate (grep for `pastable` finds only this file) and declares
`mod scoped_notes;` for a file that does not exist in that directory (the directory holds
only `tests.rs`). Leftover from the browser control-family removal
(`docs/spaceport/evidence/t04-browser-control-family-removal.md`, protocol versions
3–14/33–35).

Both sides: it is unreachable, so it changes no behavior today; but it is a stale test
tree that a future reader can mistake for live coverage, and any attempt to re-enable it
fails to compile. **Not fixed** — it is dead-code cleanup, not a bounds violation, and
deleting files outside the audit's scope without the owner's word risks removing something
the owner is deliberately keeping as a reference. Recommendation: delete the directory in
its own `chore`/`refactor` commit.

### F2 — Settings files have no read bound where the daemon view-settings file does

`sheprd_client::settings_files::read` (`crates/sheprd-client/src/settings_files.rs:83-88`)
is `fs::read_to_string` with no size check. It is the read path for the user's
`spaceport/config.toml` and `config.local.toml`
(`crates/sheprd-gpui/src/config/files.rs:35,42`), and `State::of` then compares the two
full texts.

Compare the sibling in the same settings wave: the daemon view-settings file is skipped
when `fs::metadata(path).len() > MAX_DAEMON_CONFIG_BYTES` (1 MiB)
(`crates/sheprd-gpui/src/config/daemon.rs:12,52`), with tests that pad a file past the
bound (`config/tests/sidebar_settings.rs:71-76`,
`config/tests/notification_settings.rs:286-291`). So the extraction established the
pattern for one settings file and did not apply it to the primary pair.

Both sides:

- *For a fix:* an unbounded allocation from a file whose size the user controls, read at
  every config load; every other app-owned file in the tree is bounded
  (`state_file.rs:17-31` plus per-file limits), so this is the one exception.
- *Against a reflexive fix:* `prepare_files` is documented and used as a background
  config-worker path (`config/files.rs:11-13`), so it cannot stall the UI thread, and a
  hard refusal of an oversized config needs a product decision — refuse and run on
  defaults? refuse and show the migration/conflict dialog? truncate? The migration path
  is exactly where a wrong choice loses a user's settings (the code deliberately refuses
  to overwrite a personal `config.toml` until its exact contents are stored,
  `config/files.rs:36-41`).

**Not fixed.** Recording it as a finding: it is a real missing bound, but the limit and
the failure mode are design decisions for the owner, not an unambiguous mechanical
repair. If the owner wants it, the shape is a `MAX_SETTINGS_BYTES` constant beside
`MANAGED_HEADER`, `read` taking a limit like `state_file::read`, and a `State`/`Error`
variant for an oversized file that the migration path can surface as a conflict rather
than silently defaulting.

### F3 — Version-test drift risk in removal assertions (informational)

Removal regression tests assert the protocol version with inline consts, e.g.
`const { assert!(crate::protocol::PROTOCOL_VERSION > 29) }`
(`protocol/app/tests/removed_project_open.rs:5`) and `>= 29`
(`removed_activity_panels.rs:5`). These are one-sided and cannot catch a version being
lowered past the removal they guard. Not a bounds violation and not introduced by the
audited waves; noted only because item 9 depends on the version being honest. No fix
proposed — changing those assertions is a broad, mechanical edit better done with the
next removal that bumps the version.

## Residual risks

- **Base tree unverified.** No shell tool in this run, so `origin/main` was not fetched;
  the audit reflects this worktree's checkout. If the four extraction waves landed on
  `origin/main` after this worktree was cut, the relevant files should be re-checked
  (they are listed below).
- **No checks run.** No `cargo test`, `just` recipe, `git diff --check`, or compile was
  performed in this run. The two test files that would prove items 5 and 6
  (`sheprd-client/src/tests/commands.rs`, `src/tests/partial_input.rs`) were read, not
  executed. `cargo test -p sheprd-client -p sheprd-control-protocol -p sheprd-protocol`
  plus `cargo test -p sheprd-gpui --lib control::` is the minimal set to run before this
  evidence is relied on.
- **Windows-only paths reviewed by reading only.** `socket/pipe.rs` bounds and the
  named-pipe hint were read, not exercised; the `windows` CI job is the proof there.
- Out of scope but seen: `crates/sheprd-gpui/src/sessions.rs:308` joins device-probe
  threads. That is the device scan, not the connection worker, and not a cancellation
  path for the socket worker; not audited further.

## Files that would need changes if the owner acts on the findings

- F1: delete `crates/sheprd-gpui/src/control/server/browser/` (whole directory).
- F2: `crates/sheprd-client/src/settings_files.rs` (bound + error), its `tests.rs`,
  `crates/sheprd-gpui/src/config/files.rs` (surface the refusal),
  `crates/sheprd-gpui/src/error.rs` (or reuse `StateFileSize`).
