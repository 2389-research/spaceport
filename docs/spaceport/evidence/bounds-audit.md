# Control-socket protocol bounds and queue discipline audit (wx8)

Commit path: `docs/spaceport/evidence/bounds-audit.md`.

## Scope and method

Audited the ten invariants named in `AGENTS.md` → *Protocol And Safety*, looking for
damage from the extraction waves (#133 settings, #143 browser crates, #140 render
split, #152 work-family removal) plus any bound the extraction added that is not actually
enforced.

This is a **regeneration**, not a new audit: the ten invariants and their reasoning are
the original's, re-verified against the tree the document now ships in. Every `file:line`
citation was re-read here; where a cited item moved, the path and lines were updated and
the bound re-confirmed in its new home. Changes that are more than re-pathing — a bound
that moved module, changed shape, or whose value changed — are called out in
**Revision history** and flagged in the row that owns them.

**Read base.** `origin/main` at `40e0358a7`, fetched in this run. Every citation below
was checked against that tree with `grep -n` / `sed -n`, so each one names a line that
holds the claimed item *now*. The original audit was written from a pre-rename
worktree, which is why its citations named `sheprd-*` paths that no longer resolve.

**What this run did and did not run.** Read the code and checked every citation;
`git diff --check` passes. No `cargo test`, `just` recipe, compile, or native window check
was run — this is a documentation pass. The logical claims are the original audit's,
re-read rather than re-executed.

**Three claims from the original pass, all now false:**

- The browser store **is** its own crate: `crates/spaceport-browser`, with the bounded
  store at `crates/spaceport-browser/src/store.rs`. It was extracted by `a2c534006`
  ("Give the browser's saved state a crate"), which landed after the checkout the
  original audit read. What stayed in `crates/spaceport-gpui/src/browser/` is the
  painting and view half (`view.rs`, `strip.rs`, `groups_view.rs`, …), which holds no
  store bound.
- The OSC 52 decoder **is** its own crate: `crates/spaceport-osc52`
  (`src/lib.rs` + `src/tests.rs`), extracted by `02c4bb094`. `crates/spaceport-gpui/src/osc52.rs`
  no longer exists.
- The control protocol version is **38**, not 37 (`protocol.rs:43`).

## Verdict table

| # | Invariant | Verdict | Evidence | Notes |
|---|---|---|---|---|
| 1 | Bounded request/response sizes and frame limits, enforced on both ends | **holds** | `crates/spaceport-control-protocol/src/protocol.rs:46` (`MAX_MESSAGE = 256 * 1024`), `:48` (`MAX_WAIT_SECONDS = 600`); server inbound `socket.rs:196-206`; server outbound `socket.rs:280-299`; client inbound `socket.rs:302-326` | Server and CLI share one `read_line` (`socket.rs:196-206`), which takes `MAX_MESSAGE + 1` and errors `MessageSize` past the bound. A reply that does not fit is replaced by an error, never truncated mid-line (`socket.rs:283-291`). Windows pipe hint sized to the same bound (`socket/pipe.rs:84`). Tests: `socket/tests/framing.rs:10-15,17-25,44-59`. |
| 2 | Strict decoding and atomic surface patch validation | **holds** | `protocol.rs:50-53` (`#[serde(tag = "method", deny_unknown_fields)]` on `Request`), `:81-83` (`LinkOp`), `:88-90` (`Call`), `:142-144` (`Caller`); `crates/spaceport-protocol/src/frame.rs:7-25`, `:38-74`, `:76-89`; `surface_scroll.rs:146+` | `apply_patch` = `validate_patch` then `commit_patch`; every reject path returns before any mutation. Patch identity checks boot id, projection revision, base revision and the `+1` surface revision, and refuses a patch over a popup (`frame.rs:39-46`). Row, hyperlink, pane-geometry and cursor bounds checked before commit. Scroll patches route through the same validate/commit pair. |
| 3 | Single in-flight API request; request correlation preserved | **holds** | `crates/spaceport-client/src/session.rs:453-462`; `session/barrier.rs:119-124`, `:89-91` | One lease: a command carrying `request` is re-queued and the batch breaks while `session.pending` or `session.barrier` is set, unless it is the one pipelined surface activation behind a synchronous focus request. Correlation is checked on the assembled response (`response.get("id") != Some(request_id)` → `Error::ResponseId`), and a response with no matching pending request is `Error::UnsolicitedResponse`. |
| 4 | Bounded command and event queues, FIFO, coalesced latest-state mailbox | **holds** | `crates/spaceport-client/src/limits.rs:7-8`; `crates/spaceport-client/src/event.rs:56-80`; `crates/spaceport-gpui/src/connection.rs:102,221-251,353-365`; `crates/spaceport-gpui/src/state.rs:681-725`; `crates/spaceport-control-protocol/src/socket.rs:35,37`; `crates/spaceport-gpui/src/control/server.rs:28` | Commands: `COMMAND_CAPACITY = 64` (`limits.rs:7`) into `bounded(capacity)` (`queue.rs:13`) from `connect.rs:180`, `try_send` → `Error::Full` with a `command_queue` backpressure log. Events: `bounded(8)`, `send_timeout(POLL)` never blocks unbounded and never drops silently. GPUI side is a single `Arc<Mutex<LiveState>>` (`connection.rs:102`) the reader mutates in place and the window takes with `try_lock` (`connection.rs:398-411`) — no unbounded UI event queue. Per-item caps stay in place: sound `spaceport_sound::MAX_PENDING` (`state.rs:685-687`), notifications `PENDING_LIMIT` (`state.rs:695-699`), clipboard `spaceport_osc52::MAX_PENDING` (`state.rs:712-714`), bells saturating add (`state.rs:720-725`). Control socket: 4 queued requests, 8 connections, 16 parked watches (`MAX_WATCHES`). |
| 5 | Cancellation never flushes queued work or joins a worker on the UI thread | **holds** | `crates/spaceport-client/src/handle.rs:152-156`; `session.rs:360`; `event.rs:62-65`; `crates/spaceport-control-protocol/src/socket.rs:127-131`; tests `crates/spaceport-client/src/tests/commands.rs:92-110,154-171`, `tests/partial_input.rs:479-492` | `disconnect()` only stores `stop` and wakes; the worker loop leaves without draining (`while !stop.load(...)`, `session.rs:360`, with per-command re-checks at `:412-424`), and a blocked `deliver` returns `Error::Cancelled` on the next poll. Control-socket worker threads are documented as never joined (`socket.rs:127-128`). Regression tests: cancellation behind a pending request sends nothing further and emits no events; a full event queue interrupts; a fenced FIFO head plus input is dropped without replay. |
| 6 | Partial frame prefixes survive read timeouts; inbound partial frame finished first | **holds** | `crates/spaceport-client/src/frame.rs:58-66`, `:131-159`, `:160-172`; `session.rs:432-440`; tests `tests/partial_input.rs:457-477`, `tests/framing.rs:3-21` | `FrameReader` keeps `bytes`/`target` across reads; `WouldBlock`/`TimedOut`/`Interrupted` return `Ok(None)` and keep them, and framing resumes at `target`. Length prefix bounded `0 < len <= MAX_GRAPHICS_FRAME_SIZE` (`crates/spaceport-protocol/src/codec.rs:9`). A second write is deferred while `reader.started.is_some()` unless the command is partial input that is permitted to interleave, and an in-flight API request keeps that barrier until the frame completes. `check_timeout` turns a stalled peer into a typed `Error::PartialFrameTimeout` (`frame.rs:121-129`). |
| 7 | Receiver clones compete, never broadcast; no accidental fan-out | **holds** | `crates/spaceport-client/src/handle.rs:25` (single `Receiver<ClientEvent>`), `crates/spaceport-gpui/src/connection.rs:353-365` (one reader loop consuming it) | Searched the whole workspace for `Receiver` construction and `.clone()` on receivers: `Client::events` is the only `Receiver<ClientEvent>` and is moved into the single bridge reader thread (`reader.read(&client.events)`). `crates/spaceport-control-protocol`'s `Server::requests` likewise has one consumer (`drain`/`next`). No fan-out was introduced. |
| 8 | Terminal content untrusted; only OSC 52 clipboard honored, bounded UTF-8 | **holds** | `crates/spaceport-osc52/src/lib.rs:14,16-19,27-33,35-40,69-78`; `crates/spaceport-gpui/src/state.rs:708-719`; `state.rs:742-771` | Decoder rejects any non-alphabet byte, data after padding, non-canonical trailing bits, invalid UTF-8, and a payload past `MAX_TEXT_BYTES` (2 MiB), and refuses an oversized payload before allocating. Only decoded text is queued, at most `MAX_PENDING` (8), and the queue drops its head on overflow. Every other exotic `ServerMessage` variant is explicitly ignored, including `GraphicsFile`, whose daemon-named path is never followed. |
| 9 | #152 work family gone: no old method ids; wire contract intact; `Method` has no serde order | **holds** (with a version note) | `crates/spaceport-control-protocol/src/protocol/work.rs:4-17`; `work/tests/excluded_fan_out.rs`; `work/tests.rs:26-43`; `work/tests/removed_checkpoint.rs`, `removed_teleport.rs`; `crates/spaceport-client/src/method.rs:15-29`; `protocol.rs:43` | `WorkOp` is exactly `Notes/Note/SetNote/MuteStatus/Mute` under `deny_unknown_fields`; negative tests assert `fan_out`, `script`, `restore`, `launch`, `set_note` without text are all rejected. No Rust source references the removed ids except those negative tests and the version doc comment (only marketing site text, removal-evidence docs, and the absence-check scripts elsewhere). `Method` derives no serde at all — the enum plus `as_str` is the only spelling path — so there is no bincode/serde ordering contract to break. **Version is 38, not 37**, and it moved for an unrelated reason: 37 removed chat conversion requests and replies, then 38 made `chat new` wait for the created tab and pane and reply `created`. The 38 doc block (`protocol.rs:13-42`) still matches the #152 removals recorded above. |
| 10 | New bounds from extraction are actually enforced | **holds, one gap** | `crates/spaceport-browser/src/store.rs:16-18,407-410`; `crates/spaceport-browser/src/store/tests.rs:161-176`; `crates/spaceport-browser/src/layouts.rs:21`; `crates/spaceport-state-file/src/lib.rs:21-36`; `crates/spaceport-settings/src/daemon.rs:12,54`; **`crates/spaceport-client/src/settings_files.rs:83-89`** | Browser store: `MAX_TABS = 256` enforced on both insertion paths, `MAX_FILE_BYTES = 1 MiB` on load, title truncated to `MAX_TITLE_CHARS = 200` — the bound test asserts the 257th open returns `None`. `spaceport_state_file::read` reads `limit + 1` and errors `Size` past it, and every app-owned JSON state file passes an explicit limit. **Gap:** the settings extension point `settings_files::read` is an unbounded `fs::read_to_string` — see Finding F2. |

## Findings

### F1 — Orphaned, uncompiled control-server test file (closed since the original audit)

The original audit found `crates/spaceport-gpui/src/control/server/browser/tests.rs`
present with no `mod browser;` declaring it, so the file was not part of the crate, and
noted it would not compile if wired. **That directory no longer exists**: it was removed
by `76d752e92` (`chore(gpui): Remove the orphaned browser control tests`), and
`control/server.rs:5-12` now declares exactly the families it has (`app`, `chat`,
`events`, `hosts`, `links`, `settings`, `tabs`, `work`). Nothing to do; recorded here so a
later reader does not re-open it.

### F2 — Settings files have no read bound where the daemon view-settings file does

`spaceport_client::settings_files::read`
(`crates/spaceport-client/src/settings_files.rs:83-89`) is `fs::read_to_string` with no
size check. It is the read path for the user's `spaceport/config.toml` and
`config.local.toml` (`crates/spaceport-settings/src/files.rs:35,43`), and `State::of`
(`files.rs:44`) then compares the two full texts.

Compare the sibling in the same settings wave: the daemon view-settings file is skipped
when `fs::metadata(path).len() > MAX_DAEMON_CONFIG_BYTES` (1 MiB)
(`crates/spaceport-settings/src/daemon.rs:12,54`), with tests that pad a file past the
bound (`crates/spaceport-settings/src/tests/sidebar_settings.rs:71-76`,
`tests/notification_settings.rs:285-291`). So the extraction established the pattern for
one settings file and did not apply it to the primary pair.

**Status: still open.** The read path moved from `crates/spaceport-gpui/src/config/` into
`crates/spaceport-settings/src/` in the extraction waves; the bound was not added on the
way, and `settings_files::read` is unchanged. The design question (Q1) remains
unanswered, so this is still a finding rather than a repair.

Both sides:

- *For a fix:* an unbounded allocation from a file whose size the user controls, read at
  every config load; every other app-owned file in the tree is bounded
  (`crates/spaceport-state-file/src/lib.rs:21-36` plus per-file limits), so this is the one
  exception.
- *Against a reflexive fix:* `prepare_files` is documented and used as a background
  config-worker path (`files.rs:14-15`), so it cannot stall the UI thread, and a hard
  refusal of an oversized config needs a product decision — refuse and run on defaults?
  refuse and show the migration/conflict dialog? truncate? The migration path is exactly
  where a wrong choice loses a user's settings (the code deliberately refuses to
  overwrite a personal `config.toml` until its exact contents are stored,
  `files.rs:36-41`).

**Not fixed.** Recording it as a finding: it is a real missing bound, but the limit and
the failure mode are design decisions for the owner, not an unambiguous mechanical
repair. If the owner wants it, the shape is a `MAX_SETTINGS_BYTES` constant beside
`MANAGED_HEADER`, `read` taking a limit like `spaceport_state_file::read`, and a
`State`/`Error` variant for an oversized file that the migration path can surface as a
conflict rather than silently defaulting.

### F3 — Version-test drift risk in removal assertions (informational)

Removal regression tests assert the protocol version with inline consts, e.g.
`const { assert!(crate::protocol::PROTOCOL_VERSION > 29) }`
(`protocol/app/tests/removed_project_open.rs:5`) and `>= 29`
(`removed_activity_panels.rs:5`). These are one-sided and cannot catch a version being
lowered past the removal they guard. Not a bounds violation and not introduced by the
audited waves; noted only because item 9 depends on the version being honest. No fix
proposed — changing those assertions is a broad, mechanical edit better done with the
next removal that bumps the version.

## Revision history

| Revision | Base tree | What it did |
|---|---|---|
| Original, merged as #183 | a pre-#169 worktree (`sheprd-*` crate names) | The first ten-invariant audit. Its evidence table named `sheprd-control-protocol`, `sheprd-client`, `sheprd-protocol`, `sheprd-gpui`, and two paths it believed did not exist as crates (`sheprd-browser`, `sheprd-osc52`). It also recorded that it had no shell tool, so no base tree, test, or commit was verified. |
| This regeneration | `origin/main` at `40e0358a7` | Prompted by a P2 code-review finding on #192: the audit shipped in a tree where none of its 25 `sheprd-*` paths resolve, while asserting "There is no `sheprd-browser` crate" and "There is no `sheprd-osc52` crate", both false after #169 renamed the workspace and the extraction waves gave both their own crates. Every row was re-read and re-cited; findings were re-checked; nothing new was audited. |

What changed beyond path renames — call-outs, in row order:

- **Row 9, protocol version 37 → 38.** Not a bounds change. 37 removed the chat
  conversion requests and replies; 38 (`chat new` waits for the created tab and pane)
  landed later and is unrelated to the audited waves. The claim "no serde order contract
  on `Method`" is unchanged and still holds.
- **Row 10, browser store moved crates.** `crates/spaceport-gpui/src/browser/store.rs`
  became `crates/spaceport-browser/src/store.rs`; only the GPUI view/painting half stayed
  behind. The bound itself is unchanged (`MAX_TABS = 256`, `MAX_FILE_BYTES = 1 MiB`,
  `MAX_TITLE_CHARS = 200`), but the `MAX_TABS` check **consolidated**: the original
  described two independent insertion-path checks, and now `open` (`store.rs:381-400`)
  delegates to `restore_tab` (`store.rs:407-410`), which holds the single check. Both
  paths are still bounded, through one check rather than two.
- **Row 10, browser load now uses the shared reader.** The store and layouts files no
  longer read with a local `fs` call: they call
  `spaceport_state_file::read(path, MAX_FILE_BYTES)` (`store.rs:280`,
  `layouts.rs:104,128`), so the browser bound is now enforced by the same bounded reader
  as every other app-owned JSON state file. This strengthens the row rather than
  weakening it.
- **Row 10, settings files moved.** `crates/spaceport-gpui/src/config/daemon.rs` became
  `crates/spaceport-settings/src/daemon.rs` (`MAX_DAEMON_CONFIG_BYTES` still on the same
  line, 12) and `config/files.rs` became `crates/spaceport-settings/src/files.rs`. F2's
  unbounded read did not gain a bound in the move.
- **Row 10, state file moved.** The original's unprefixed `state_file.rs:17-31` is now
  `crates/spaceport-state-file/src/lib.rs:21-36` (`5b6731e6c` gave the writer its own
  crate); the bound (`take(limit + 1)`, then `Error::Size`) is identical.
- **Row 8, OSC 52 decoder moved crates.** `crates/spaceport-gpui/src/osc52.rs` became
  `crates/spaceport-osc52/src/lib.rs`. `MAX_TEXT_BYTES = 2 MiB` and `MAX_PENDING = 8` are
  unchanged; the decoder function moved from module scope to the crate root, so its
  `file:line` moved while its logic did not.
- **F1 closed** by `76d752e92`; see the finding.
- **Row 5/7 GPUI evidence moved.** The `Arc<Mutex<LiveState>>` field and the single
  reader thread are now at `connection.rs:102` and `:353-365`; the claim is unchanged.

## Residual risks

- **No checks run.** No `cargo test`, `just` recipe, compile, or native window check was
  performed in this run; `git diff --check` and per-citation `grep`/`sed` were. The two
  test files that would prove items 5 and 6 (`crates/spaceport-client/src/tests/commands.rs`,
  `tests/partial_input.rs`) were read, not executed.
  `cargo test -p spaceport-client -p spaceport-control-protocol -p spaceport-protocol`
  plus `cargo test -p spaceport-gpui --lib control::` is the minimal set to run before
  this evidence is relied on.
- **Windows-only paths reviewed by reading only.** `socket/pipe.rs` bounds and the
  named-pipe hint were read, not exercised; the `windows` CI job is the proof there.
- **Citations are a snapshot.** They were correct at `40e0358a7`. Anything that moves
  these files again invalidates the line numbers; the invariant text is the durable part.
- Out of scope but seen: `crates/spaceport-gpui/src/sessions.rs:311` joins device-probe
  threads. That is the device scan, not the connection worker, and not a cancellation
  path for the socket worker; not audited further.

## Files that would need changes if the owner acts on the findings

- F2: `crates/spaceport-client/src/settings_files.rs` (bound + error), its `tests.rs`,
  `crates/spaceport-settings/src/files.rs` (surface the refusal),
  `crates/spaceport-settings/src/error.rs` (or reuse `spaceport_state_file::Error::Size`).
