# Splitting `crates/spaceport-gpui`

`crates/spaceport-gpui` is ~313k lines over 1911 files and links the app's heaviest
dependencies (mermaid-rs-renderer, ratex-*, image, nucleo-matcher, syntect) for
every change, whether or not it touches them. This note records the
extractions so far, the pattern they proved, and the dependency analysis for the
next ones. Update it as each step lands.

## What moved

Twelve modules have left the app. Eleven are pure functions of their inputs
with no `gpui` types; `spaceport-browser` is the first that has to carry them:

| Crate | From | Dependencies | Files importing it |
| --- | --- | --- | --- |
| `crates/spaceport-ago` | `crates/sheprd-gpui/src/ago.rs` | none | 1 (`palette/render.rs`) |
| `crates/spaceport-lenient` | `crates/sheprd-gpui/src/lenient.rs` | `serde`, `toml` | 4 settings files |
| `crates/spaceport-osc52` | `crates/sheprd-gpui/src/osc52.rs` | none | 2 (`state.rs` and its tests) |
| `crates/spaceport-settings` | `crates/sheprd-gpui/src/config*`, `src/contrast.rs` | `spaceport-client`, `spaceport-commands`, `spaceport-i18n`, `spaceport-usage`, `spaceport-lenient`, `serde`, `toml`, `thiserror`, … | 307 (through the `crate::config` façade in `src/config.rs`) |
| `crates/spaceport-sound` | `crates/sheprd-gpui/src/sound/` | `rodio`, `serde`, `spaceport-lenient`, `spaceport-protocol`, `thiserror`, `toml`, `tracing` | 17 |
| `crates/spaceport-words` | `crates/sheprd-gpui/src/words.rs` | `unicode-segmentation` | 2 (`multiline_input.rs`, `text_editing/keys.rs`) |
| `crates/spaceport-panes` | `crates/sheprd-gpui/src/core_terminals/panes*` | `serde`, `spaceport-backend`, `thiserror` | 16 (through `spaceport_panes::`) |
| `crates/spaceport-state-file` | `crates/sheprd-gpui/src/state_file.rs` | `serde`, `serde_json`, `tempfile`, `thiserror`, `tracing` | 10 |
| `crates/spaceport-motion` | `crates/sheprd-gpui/src/motion/slow*`, `motion.rs`'s clock | none | 49 (through the `crate::motion` façade) |
| `crates/spaceport-browser` | `crates/sheprd-gpui/src/browser*` | `gpui`, `serde`, `serde_json`, `spaceport-client`, `spaceport-motion`, `spaceport-panes`, `spaceport-reorder`, `spaceport-state-file`, `spaceport-truncate`, `thiserror`, `tracing`, `url`, `uuid` | 51 (through the `crate::browser` façade) |
| `crates/spaceport-chat-core` | `crates/spaceport-gpui/src/chat/{env,list,existing,session,link}*` | `serde_json`, `spaceport-chat-protocol`, `spaceport-client`, `spaceport-remote`, `thiserror`, `uuid` | 13 (`chat.rs`'s façade, `chat/chats.rs`, `chat/discovery.rs`, `chat/start.rs`, `i18n/errors/chat.rs` and 8 tests) |

The browser's own record is
[`notes-browser-extraction.md`](notes-browser-extraction.md): what moved, what
stayed on the window, and the two seams the design had to change.

### `spaceport-ago`

`crates/spaceport-ago` is a new workspace member with **no dependencies at all**:

| Before | After |
| --- | --- |
| `crates/sheprd-gpui/src/ago.rs`, 32 lines | `crates/spaceport-ago/src/lib.rs`: `Elapsed` and `elapsed(seconds)` |
| its unit table in `src/ago/tests.rs` | `crates/spaceport-ago/src/tests.rs` |
| its message table and `text()` | stay in `crates/spaceport-gpui/src/ago.rs`, 34 lines |

`ago` was picked over `sound`, `app_badge` and `about` because it is the only
one of the four with no `gpui` types and a single importer:

| Module | production lines | `gpui::` uses | files importing it |
| --- | --- | --- | --- |
| `ago` | 32 | 0 | 1 (`palette/render.rs`) |
| `sound` | 848 | 0 | 16 |
| `about` | 258 | 4 | 3 |
| `app_badge` | 302 | 25 | 3 |

### `spaceport-osc52`

`crates/spaceport-osc52` is the second, also with **no dependencies at all**. The
module file also held its tests; they moved out of the production file, and the
app keeps only the call site:

| Before | After |
| --- | --- |
| `crates/sheprd-gpui/src/osc52.rs`, 122 production lines and 3 inline tests | `crates/spaceport-osc52/src/lib.rs`: `MAX_TEXT_BYTES`, `MAX_PENDING`, `decode(data)` |
| its three inline tests (20 lines) | `crates/spaceport-osc52/src/tests.rs` |
| `src/state.rs` decoding a payload and bounding its mailbox | stays: `spaceport_osc52::decode`, `spaceport_osc52::MAX_PENDING` |
| `src/state/tests/daemon_messages.rs` reading the mailbox bound | stays: `spaceport_osc52::MAX_PENDING` |

It has no messages and no error type of its own: `decode` returns `None` for a
payload it will not write, and the app logs the drop.

### `spaceport-words`

`crates/spaceport-words` depends only on `unicode-segmentation`, and its tests were
already in `src/words/tests.rs`, so they moved as they were:

| Before | After |
| --- | --- |
| `crates/sheprd-gpui/src/words.rs`, 35 lines | `crates/spaceport-words/src/lib.rs`: `previous_start(text, offset)` and `next_end(text, offset)` |
| `crates/sheprd-gpui/src/words/tests.rs`, 41 lines | `crates/spaceport-words/src/tests.rs` |
| `src/multiline_input.rs` (double-click selection) and `src/text_editing/keys.rs` (word movement) | both call `spaceport_words::` |

It carries no messages either: the fields that use it own their own text.

### `spaceport-lenient`

`crates/spaceport-lenient` depends only on `serde` and `toml`. Herdr writes
`config.toml` and this app does not own it, so every value the build cannot read
falls back to its own default: a property of a deserializer, not of the app.

| Before | After |
| --- | --- |
| `crates/sheprd-gpui/src/lenient.rs`, 107 lines | `crates/spaceport-lenient/src/lib.rs`, 51 lines: `value`, `or_default`, `or_true`, `entries` |
| its inline test | `crates/spaceport-lenient/src/tests.rs`, 56 lines |
| 44 `crate::lenient::` paths in four settings files | the same paths naming `spaceport_lenient` |

It has no messages and no error type: a value it cannot read falls back, and
malformed TOML fails with `toml`'s own error in the caller.

### `spaceport-sound`

The first extraction that moved a dependency out of the app crate's own
direct dependencies: `rodio` (with `cpal`, CoreAudio, `symphonia` below it)
is no longer in `spaceport-gpui`'s `[dependencies]`, `spaceport-sound` can be
tested without building GPUI, and the audio code has its own crate. The
stack is still in the app's *transitive* graph — `spaceport-gpui` depends on
`spaceport-sound`, which depends on `rodio` — so a built app binary links it
as before; the win is ownership and test isolation, not link size. A
boundary that actually excludes those dependencies would need the sound
feature to be optional, which is a product decision, not a refactor.

| Before | After |
| --- | --- |
| `crates/sheprd-gpui/src/sound.rs`, 455 lines | `crates/spaceport-sound/src/service.rs` (the worker) and `src/policy.rs` (`Policy`, `Live`, `MAX_PENDING`) |
| `src/sound/config.rs`, 196 lines | `src/config.rs`: `Settings::load(path)`, no path policy of its own |
| `src/sound/playback.rs`, 159 lines | `src/playback.rs`: `MAX_VOLUME`, `muted`, and rodio playback |
| `src/sound/error.rs`, 47 lines | `src/error.rs`: the crate's own `thiserror` `Error`, with `Io`, `Path` and `Toml` added for settings |
| 1,170 test lines | 29 crate tests alongside the code they drive; 3 tests that exercise the seam from the app side stay in `connection/tests/sounds.rs` and `state/tests/sounds.rs` |

Three couplings had to invert first, each one parameter or struct:

- `Policy::poll` took `&mut LiveState`, which is the app's. It now takes
  `spaceport_sound::Live`: the two cancellation tokens, whether the daemon is
  connected, the reload flag, the queue it drains, and the newest snapshot. The
  app builds it in one place, `LiveState::sound_live`.
- `Settings::load` resolved its own path through `config::view_settings_path`,
  which is the app's policy. It takes the file now; `Service::new(path, volume)`
  gets both from `app::shared_sound`, resolved once at construction.
- `Service` held `Arc<Mutex<Option<Arc<Settings>>>>` and the app called
  `ui_delay`, `path_for` and `allows` on it. Those became `pub(crate)`, and the
  app only calls the worker.

`crates/spaceport-truncate` depends on `unicode-segmentation`, `unicode-width` and
`spaceport-client`:

| Before | After |
| --- | --- |
| `crates/sheprd-gpui/src/truncate.rs`, 105 lines | `crates/spaceport-truncate/src/lib.rs`, 105 lines: `cut`, `cut_chars`, `cut_width`, `one_line`, `ellipsize`, `tab_title`, `ELLIPSIS` |
| `crates/sheprd-gpui/src/truncate/tests.rs`, 86 lines | `crates/spaceport-truncate/src/tests.rs`, moved unchanged |
| 6 `crate::truncate::` paths in five files | the same paths naming `spaceport_truncate`; `one_line` calls `spaceport_client::text::safe_text`, which it had reached through `crate::notifications` |

### `spaceport-chat-core`

The first step of the `chat` extraction, and a model-only one. Of the 203 files
in `crates/spaceport-gpui/src/chat`, 111 import `gpui`; of the rest, only three
modules were both `gpui`-free and independent of the app (no `crate::i18n::fl`,
no window state, no `crate::chat::prepared`):

| Before | After |
| --- | --- |
| `crates/spaceport-gpui/src/chat/env.rs`, 106 lines: `MAX_ENV`, `MAX_ENV_BYTES`, `check_env`, `env_text`, `parse_env`, its `Error` | `crates/spaceport-chat-core/src/env.rs`; `PROGRAM_ENV` moved here from `chat/start.rs`, which re-exports it, because the reserved name and the table that may not replace it are one fact |
| `chat/tests/env.rs`, 2 tests | `src/env/tests.rs`, moved unchanged apart from its import paths |
| `crates/spaceport-gpui/src/chat/list.rs`, 54 lines: `Splice`, `splices` | `src/list.rs` |
| `chat/tests/list.rs`, 4 tests | `src/list/tests.rs` |
| `crates/spaceport-gpui/src/chat/existing.rs` and `existing/error.rs`, 505 lines: `Existing`, `Binding`, `Scope`, `candidates`, the probe worker | `src/existing.rs` and `src/existing/error.rs`; the one app call it made, `super::discovery::endpoint`, became `crate::endpoint`, and `discovery.rs` re-exports that name so its own callers keep their paths |
| `chat/existing/tests.rs` and `existing/tests/scripts.rs`, 16 tests (4 of them `cfg(target_os = "linux")`) | `src/existing/tests.rs` and `src/existing/tests/scripts.rs`, moved unchanged; the crate takes `spaceport-remote`'s `test-support` feature for `Host::env` as the app does |
| `chat.rs`'s `const MAX_CHATS` and `discovery.rs`'s `endpoint` | `spaceport_chat_core::{MAX_CHATS, endpoint}`; `chat.rs` re-exports both, so no file outside the moved modules changed |

`crates/spaceport-gpui/src/chat.rs` keeps every path the app uses: it replaces
`mod env`, `mod existing` and `mod list` with `pub(crate) use
spaceport_chat_core::{env, existing, list}`, so `crate::chat::env::MAX_ENV`,
`crate::chat::existing::Binding` and `crate::chat::list::splices` resolve as
before. The only other edits in the app are where a definition moved and its
old path is now a re-export (`chat/discovery.rs` for `endpoint`,
`chat/start.rs` for `PROGRAM_ENV`) and the two dropped `mod` lines in
`chat/tests.rs`. No call site changed, which is the `#143` discipline this
crate is meant to prove, and the app's chat error, session and link were left
for stage 2.

Stage 2 added the attach model: `chat/session.rs`, `chat/link.rs` and their
tests, after splitting the one type that held them back.

| Before | After |
| --- | --- |
| the app's `chat::Error`, 18 variants: 15 attach-side (`Apply`, `Busy`, `Bye`, `Closed`, `Connect`, `Handshake`, `NoEndpointDir`, `NotAttached`, `Overflow`, `Read`, `ReadOnly`, `Remote`, `Spawn`, `Stream`, `Version`) beside `Environment`, `Pasted` and `Start` | `src/error.rs` holds those 15 with `permanent()` and `desynced()`, next to the `Environment` and `Existing` it already had; the app's `chat::Error` keeps its three wrappers and gains one transparent `Attach(#[from] spaceport_chat_core::Error)` |
| `i18n/errors/chat.rs`, 18 arms matching the app's chat error, plus `env::Error`'s 2 | 4 arms on the app's error, each delegating, and `impl Localize for spaceport_chat_core::Error` with 17 arms — the same messages, at the same `.ftl` ids; the crate's `Existing` variant, which no attach path raises, keeps its own words |
| `chat/session.rs`, 384 lines: `Session`, `Refusal`, the pending-command state machine | `src/session.rs` |
| `chat/link.rs`, 475 lines: `Link`, `Dial`, `Polled`, `WriteOutcome`, the reader and writer threads | `src/link.rs` |
| `chat/tests/session.rs`, `session_stream.rs`, `delivery.rs`, 30 tests | `src/session/tests.rs` (fixtures and the session tests) with `src/session/tests/stream.rs` and `src/session/tests/delivery.rs`, moved as they were |
| `chat/link/tests.rs`, 8 tests | `src/link/tests.rs`, moved as it was |
| `chat.rs`'s `MAX_PENDING`, `MAX_EVENTS` | `spaceport_chat_core::{MAX_PENDING, MAX_EVENTS}`: both bound the attach side, so they moved with it |

`crates/spaceport-gpui/src/chat.rs` again keeps every path the app uses, with
`pub(crate) use spaceport_chat_core::{MAX_CHATS, env, existing, link, list,
session}` and `pub(crate) use spaceport_chat_core::session::Refusal`, so
`crate::chat::link::Link`, `crate::chat::session::Session` and
`crate::chat::Refusal` resolve as before. `chat/tests/link.rs` stays in the
app: it drives a `Link` through the shared private-endpoint fixture (`Program`,
`Peer`) that the window's own chat tests use, and it names
`spaceport_chat_core::MAX_EVENTS` for the bound it checks. `MAX_FAILURES` and
every retry decision stay in `chat/chats.rs`.

The error split is why this stage is two commits. Splitting the error alone
touched 19 files and moved none of them: 4 sites in `chat/chats.rs`, 6 in
`chat/tests/chats/phase.rs`, one each in
`chat/tests/{uploads,source,overlay,feature}.rs`, the 4 test connectors that
now map `Link::start`'s error into the app's, and the two i18n files — 18 arms
on the app's error became 4 that delegate, and the core error's 17 arms carry
the messages. The second commit then moved `session.rs` and `link.rs` with no
behaviour change, which is what makes the diff reviewable.

`crates/spaceport-chat-core` now holds 1,662 production and 1,514 test lines;
`crates/spaceport-gpui/src/chat` is down to 190 files. The app's chat tests
lost 30 session tests and 8 link tests, the core crate gained all 38 and none
was dropped: 357 chat-module tests before, 357 after.

## The pattern

1. **Pick by footprint, not by size.** For each candidate module count what it
   imports (`crate::`, `gpui::`) and how many files import it back:
   `rg -l 'crate::<module>\b' crates/spaceport-gpui/src`. Fewest reverse importers
   first; a module that renders or holds a GPUI entity is not a candidate yet.
2. **Take a crate with its own tests.** New `crates/spaceport-<module>/Cargo.toml`
   copies `[package]`'s workspace fields from `crates/spaceport-i18n`, sets
   `[lints] workspace = true`, and lists only the dependencies the module uses,
   inherited from `[workspace.dependencies]`. A crate with no error paths has no
   `error.rs`; one with them gets its own `thiserror` `Error`/`Result` and one
   `#[error(transparent)]` variant in the app's root `error.rs`.
3. **Re-export a narrow root API.** `src/lib.rs` keeps the doc comment that
   explains the module's job; private items stay private. `pub(crate)` becomes
   `pub` only for what the app calls.
4. **Move the tests, not copies.** `src/<module>/tests/` becomes
   `src/tests.rs` + `src/tests/`, starting with `use super::*;`; test bodies stay
   out of production files. A test that needs the app (the shown locale, a
   window) stays in the app next to the code that writes the words.
5. **Messages are the app's** (`docs/i18n.md`): a crate below the app returns
   typed values and the app writes them. `spaceport-ago` names the unit and its
   count; `crates/spaceport-gpui/src/ago.rs` keeps `ago.ftl`'s IDs. A crate that
   names message IDs as data instead follows `spaceport-commands`/`spaceport-usage`.
6. **Wire and record.** Workspace `members`, `[workspace.dependencies]`,
   `spaceport-gpui`'s `[dependencies]`, the crate table in `AGENTS.md`, and the
   layout in `docs/i18n.md` when the module carries text.
7. **Verify before handoff:** `cargo check -p <crate> -p spaceport-gpui`,
   `cargo test -p <crate>`, `cargo fmt --all -- --check`, `just check-file-size`,
   `just check-sorted`, `just check-i18n`. `cargo metadata` refreshes
   `Cargo.lock`, which the commit includes.

## Next extractions

Cheap, dependency-free moves first; the blocked ones carry real work.

The next extractions are the three large features; see the feature table below.
The settings crate landed with the `crate::config` façade still in place in
`src/config.rs`, so importers keep compiling unchanged; flattening those paths
to `spaceport_settings::` is a separate, batched readability commit, not a
migration step.

`chat`, `browser` and `sidebar` are the app's three large features and each is a
program of its own; none can move before the state and text they read have
homes. Counted today, production lines first:

| Feature | production / test lines | `gpui::`/entity uses | i18n messages | files importing it | what blocks it |
| --- | --- | --- | --- | --- | --- |
| `browser` | moved: `crates/spaceport-browser` holds 3,398 production and 1,904 test lines | — | 10 | 51 | done; 1,395 production lines stay in the app as the window's own (wrappers, element building, `group_shown`, the close driver) — see [`notes-browser-extraction.md`](notes-browser-extraction.md) |
| `chat` | 20,706 / 16,719, of which `crates/spaceport-chat-core` holds 1,662 production and 1,514 test lines | 128 | 174 | 12 | GPUI views all the way down; stages 1-2 landed the core (`spaceport-chat-core`: the model and the attach side), stages 3-4 below; needs `browser` below it for the URL and page parts, then the proposed live-state crate described in the settings design |
| `sidebar` | 11,195 / 14,506 | 162 | 75 | 263 | the most-imported module in the crate; settings now below it, still needs `menu`, `keymap`, `endpoint`, `icons`, `worktree_notes`, `device_tags` |

Order that falls out of the counts: `browser` before `sidebar` (53 against 263
importers), and `chat` only after `browser` and the shared paint/settings types
(`terminal_painter`, `config`, `notifications`) are below it. `sound` proved the
pattern a feature needs: a value type for what the crate reads, and a parameter
for what the app owns.

### `chat`, in four stages

Counted on the commit that landed stage 1: of the 203 files under
`crates/spaceport-gpui/src/chat`, 111 import `gpui`, 92 do not, and of those 92
only three modules were free of the app itself (stage 1, above). The other 89
are `gpui`-free but reach the app, and split into the stages below.

- **Stage 2, the rest of the attach model: `session.rs` and `link.rs`.** Done,
  in two commits: the error split, then the move. What the split cost is
  counted under `spaceport-chat-core` above; the move took 859 production and
  921 test lines out of the app without changing a line of behaviour. One test
  file stayed: `chat/tests/link.rs` drives the moved `Link` through the app's
  shared private-endpoint fixture.
- **Stage 3, parsing and measurement: `prepared.rs`, `rich/`, `selection/`,
  `text.rs`, `summary.rs`, `highlight.rs`.** `gpui`-free today but read the
  app's words and neighbours: `crate::i18n::fl` (messages are the app's, per
  `docs/i18n.md`, so these have to return the values the app writes, as
  `spaceport-ago` does), `crate::release_notes`, `crate::browser::WebUrl`,
  `crate::terminal::file_target`, `crate::chat::visual` (whose `Visuals` holds
  `gpui::RenderImage`) and `crate::config::Theme`/`crate::contrast`. Each of
  those has to be below the crate first.
- **Stage 4, everything that paints or holds an entity:** `view/` (the largest
  group), `widgets/state.rs`, `visual/`, `files.rs`, `pasted.rs`, `page.rs`,
  `harnesses.rs`, `start.rs`, `preview/`, `window.rs`, `chats.rs`, `reader.rs`,
  `discovery.rs`'s `LiveState` reads, `core_pane.rs`, `glide.rs`, `terminal.rs`.
  These need `SpaceportWindow`, `Entity`, `ListState` or `LiveState` and stay
  until the live-state crate described in the settings design exists.

Stage 3's messages are what the attach error is displayed through, so its
`crate::i18n::fl` calls have to move the same way stage 2's arms did: the crate
returns the values, the app writes the words (`docs/i18n.md`).

## What this does not do

No behaviour changes, no new dependencies, no file reshuffles beyond the module
that moved, and no change to another crate. `spaceport-gpui` still links
`mermaid-rs-renderer`, `ratex-*`, `image` and `syntect` until `chat` moves; that
is the commit that shrinks the build next.
