# Splitting `crates/spaceport-gpui`

`crates/spaceport-gpui` is ~313k lines over 1911 files and links the app's heaviest
dependencies (mermaid-rs-renderer, ratex-*, image, nucleo-matcher) for
every change, whether or not it touches them. `syntect` left the app's own
dependencies with stage 3a below and stays in the binary through
`spaceport-chat-core`. This note records the
extractions so far, the pattern they proved, and the dependency analysis for the
next ones. Update it as each step lands.

## What moved

Eighteen crates have left the app. Four of them carry `gpui` — `spaceport-browser`,
`spaceport-widgets`, `spaceport-menu` and `spaceport-settings-window` — and the other
fourteen name no `gpui` type at all:

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
| `crates/spaceport-chat-core` | `crates/spaceport-gpui/src/chat/{env,list,existing,session,link,highlight,text}*`, and `chat/discovery.rs`'s `Route` and `route` (#234) | `pulldown-cmark`, `serde`, `serde_json`, `spaceport-chat-protocol`, `spaceport-client`, `spaceport-live`, `spaceport-release-notes`, `spaceport-remote`, `spaceport-settings`, `spaceport-web-url`, `syntect`, `thiserror`, `unicode-width`, `uuid` | 30: 12 production files (the `chat.rs` façade, `chat.rs`'s neighbours `widgets`, `visual`, `chats`, `text`, `labels`, `rich`, `discovery`, `start`, `prepared`, `selection`, and `i18n/errors/chat.rs`) and 18 test files |
| `crates/spaceport-widgets` | `crates/spaceport-gpui/src/{direction,toggles,accessibility}.rs` (#220), `fonts.rs` and `motion.rs`'s `pop_in` (#224), `browser::Thumb` (#230) | `gpui`, `spaceport-i18n`, `spaceport-motion`, `spaceport-settings`, `spaceport-sidebar` | 7 (through the app's own façades: `accessibility.rs`, `config.rs`, `direction.rs`, `fonts.rs`, `motion.rs`, `toggles.rs`, `i18n/current.rs`) |
| `crates/spaceport-settings-window` | `crates/spaceport-gpui/src/settings_window/ui.rs` and `ui/{number,popover,swatches}.rs` (#224), then `settings_window/{navigation,scrollbar,inputs}.rs` (#230) | `gpui`, `spaceport-i18n`, `spaceport-settings`, `spaceport-widgets` | 10 (through the façades under `src/settings_window/`) |
| `crates/spaceport-menu` | `crates/spaceport-gpui/src/menu/{page,workspace,appear,sessions,chrome}*`, `pane_menu.rs`, `tab_menu.rs`, `group_menu.rs` (#223, #225, #231), `menus/in_window/render.rs` (#231) | `gpui`, `serde_json`, `spaceport-client`, `spaceport-motion`, `spaceport-settings`, `spaceport-usage`, `spaceport-widgets`, `thiserror` | 18 (through the façades in `src/menu/*` and `src/{pane_menu,tab_menu}.rs`, plus `error.rs` and the i18n files) |
| `crates/spaceport-endpoint` | `crates/spaceport-gpui/src/endpoint/{history,wake,order,ids,follow,storage,catalog,fence,link,phase}*` and `origin.rs` (#226, #228, #233) | `spaceport-client`, `spaceport-live`, `spaceport-reorder`, `thiserror`, `tracing` | 13 (through the `crate::endpoint` façade, plus `sidebar/rail/render.rs` and `sidebar/render/agent_rows.rs` reading `HostView`) |
| `crates/spaceport-palette` | `crates/spaceport-gpui/src/palette.rs`'s ranking model, `palette/{search,agent_filter,error,frecency,projects,project_create}.rs`, `navigation.rs` (#229; it moved on to `spaceport-live` in #232) | `nucleo-matcher`, `serde`, `serde_json`, `spaceport-client`, `spaceport-commands`, `spaceport-live`, `thiserror`, `unicode-segmentation` | 7 (through the façades in `src/palette/*`) |
| `crates/spaceport-live` | `crates/spaceport-gpui/src/{state,notice,notifications}*`, `state/{activation,projection,agent_recency}.rs`, `navigation.rs` (#232) | `serde_json`, `spaceport-client`, `spaceport-osc52`, `spaceport-sound`, `tracing` | 4 (through the `crate::state`, `crate::notice` and `crate::navigation` façades) |

The browser's own record is
[`notes-browser-extraction.md`](notes-browser-extraction.md): what moved, what
stayed on the window, and the two seams the design had to change.

### `spaceport-ago`

`crates/spaceport-ago` is a new workspace member with **no dependencies at all**:

| Before | After |
| --- | --- |
| `crates/spaceport-gpui/src/ago.rs`, 32 lines | `crates/spaceport-ago/src/lib.rs`: `Elapsed` and `elapsed(seconds)` |
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
| `crates/spaceport-gpui/src/words.rs`, 35 lines | `crates/spaceport-words/src/lib.rs`: `previous_start(text, offset)` and `next_end(text, offset)` |
| `crates/spaceport-gpui/src/words/tests.rs`, 41 lines | `crates/spaceport-words/src/tests.rs` |
| `src/multiline_input.rs` (double-click selection) and `src/text_editing/keys.rs` (word movement) | both call `spaceport_words::` |

It carries no messages either: the fields that use it own their own text.

### `spaceport-lenient`

`crates/spaceport-lenient` depends only on `serde` and `toml`. Herdr writes
`config.toml` and this app does not own it, so every value the build cannot read
falls back to its own default: a property of a deserializer, not of the app.

| Before | After |
| --- | --- |
| `crates/spaceport-gpui/src/lenient.rs`, 107 lines | `crates/spaceport-lenient/src/lib.rs`, 51 lines: `value`, `or_default`, `or_true`, `entries` |
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

`crates/spaceport-chat-core` holds 1,662 production and 1,514 test lines after
stage 2, and `crates/spaceport-gpui/src/chat` is down to 190 files. The app's
chat tests lost 30 session tests and 8 link tests, the core crate gained all 38
and none was dropped: 357 chat-module tests before, 357 after.

Stage 3a took the two stage-3 modules that needed no split first. `highlight.rs`
was `gpui`-free and reached the app only for the settings types the move takes
below it; `text.rs` moved once it stopped writing words.

| Before | After |
| --- | --- |
| `crates/spaceport-gpui/src/chat/highlight.rs`, 188 lines: `Token`, `Contrast`, `Span`, `colour_file` | `crates/spaceport-chat-core/src/highlight.rs`, moved as it was apart from its two imports (`spaceport_settings::Theme` and `spaceport_settings::contrast::ink_on_chrome`) and its `pub(crate)` items becoming `pub` |
| `crates/spaceport-gpui/src/chat/text.rs`, 66 lines: `duration` and `turn` writing their own words through `crate::i18n::fl` | `crates/spaceport-chat-core/src/text.rs`, 58 lines: `Elapsed` (milliseconds, seconds, minutes with their seconds, hours with their minutes), `Measured` (how a turn ended, its `Elapsed`, its tokens), and `duration(ms)` and `turn(&TurnSummary)` to fill them |
| its 12 `chat-text-*` messages | stay in `chat/text.rs`, 69 lines: `words(Elapsed)`, and `duration(ms)` and `turn(&TurnSummary)` over the crate's values, as `src/ago.rs` writes `spaceport-ago`'s. Every call site keeps its signature |
| `chat/text/tests.rs`, 1 test comparing the English to the protocol's `display` | stays, driving the app's writer over the crate's measurement: it needs the shown locale |
| 2 new tests | `crates/spaceport-chat-core/src/text/tests.rs`: the unit boundaries of `duration`, and `turn` keeping what its summary says |

`chat.rs` keeps every path the app uses again, with `highlight` added to
`pub(crate) use spaceport_chat_core::{MAX_CHATS, env, existing, highlight, link,
list, session}` and `mod highlight` dropped, so `crate::chat::highlight::Span`
and `super::highlight::colour_file` resolve as before. No call site changed: the
6 chat files that read `highlight` and the 6 that call `crate::chat::text::{duration,
turn}` name the same items at the same paths.

`syntect` left `spaceport-gpui`'s own `[dependencies]` in the first of the two
commits: 54 direct dependencies before, 53 after, and `spaceport-chat-core` 6 to
8 (`spaceport-settings` and `syntect`). The grammars are still in the built
binary through `spaceport-chat-core`, so a code block is coloured exactly as
before; what changed is ownership, and that the parser can be tested without
building GPUI. The crate holds 1,910 production and 1,605 test lines after
this stage, `crates/spaceport-gpui/src/chat` is down to 189 files, and the core
crate's 60 test functions became 62: 2 added, none moved, none dropped. The
app's chat tests stay at 408 and its bin suite still runs 3,632.

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
| `chat` | 16,110 production / 14,070 test lines stay in the app (`src/chat` with `chat.rs`), beside `crates/spaceport-chat-core`'s 5,281 / 3,272 (which names no `gpui` at all) | 74 | 141 | 30 (through the `crate::chat` façade) | stages 1-3d landed: the crate holds the attach model, the syntax spans and measurements, the prepared entry model with its widgets, the rich block model and parse, and the selectable text. Stage 4's first slice landed in #234: `Route` and the local-daemon trust boundary now live in the crate over `spaceport-live`'s `LiveState`. What stays app-side is every painter (`view/` and `view.rs`, 10,005 lines with their tests), the `Visuals` workers, the `Prepared` shell that hands their wants over, the pane list with `Target`/`Source`/`Surface`, the start fence, the link classification, and the I/O and process paths — see [`notes-wx41.md`](notes-wx41.md) and [`notes-wx65.md`](notes-wx65.md) |
| `endpoint` | 2,069 production / 12,544 test lines stay in the app (`src/endpoint` with `endpoint.rs`), beside `crates/spaceport-endpoint`'s 1,659 / 1,163 and `crates/spaceport-live`'s 1,974 / 77 (neither names `gpui` at all) | 3, all `Context<SpaceportWindow>`; nothing builds an element | 15 | 75 (through the `crate::endpoint` façade) | all five stages landed — #226, #228, #232, #233, #235, one per stage in the table below: the two crates hold the connection policy, the saved-host catalog, `LiveState` with its activation fence and notices, the link machine behind `Host`, and the `HostView` a sidebar row is handed. What stays app-side is the socket half and the window's own state: `ConnectionBridge` in `connection.rs` with `connection/reader.rs` (766 lines), `Release`, `Redraw`, `polling.rs` (496), `reset_selected`, the fences' reads, `catalog.rs`, `follow.rs` and every view. The 10,710-line socket-driving lifecycle suite stays with them forever — it opens real sockets and processes, and the design's [honest-remainder table](notes-endpoint-extraction.md) counts it as permanent — see [`notes-endpoint-extraction.md`](notes-endpoint-extraction.md) |
| `sidebar` | 6,487 / 13,679 remain in the app, beside `crates/spaceport-sidebar`'s 6,610 / 1,611 (31 `gpui::` uses; the crate is a GPUI crate, like `spaceport-browser`) | 74 | 263 | stages 1–8a landed: the crate holds the row/agent models, geometry, layouts, tokens, row paint, the direction mirror, the interaction math (reorder, device drag, hover, order), the notes on checkouts (`worktree_notes`) and the tags on devices (`device_tags`, both #201); what stays app-side is window-owned glue (~680 lines of `impl SpaceportWindow` in `reorder.rs`, `device_drag.rs`, `hover.rs`, `order.rs`) that sends the crate's typed requests — the browser program's terminal shape, not a defect to fix.Two blockers are left before the render lane can follow: the menu page bodies (`menu` holds 10,865 production lines app-side, and its remaining pages wait on the P2 text-field crate — stage 1 landed in #252, `DialogInput` still app-side — and the P1 ruling, answered: the lookup moves into `spaceport-i18n`) and `keymap` — which stopped being one: all four stages landed (#239, #243, #249). `endpoint` stopped being one in #235: the row and rail renderers read the crate's `HostView` now |
| `text-field` | moved: `crates/spaceport-textfield` holds 3,727 lines (`SearchInput` with the shared caret, editing and key machinery) | — | 0 | — | stage 1 landed in #252: the crate holds `SearchInput` and the shared machinery; `DialogInput` and `MultilineInput` stay app-side behind the facade until stage 2 proves stage 1 out — see [`notes-wx82.md`](notes-wx82.md) |

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
  `text.rs`, `summary.rs`, `highlight.rs`.** `gpui`-free but read the app's
  words and neighbours: `crate::i18n::fl` (messages are the app's, per
  `docs/i18n.md`, so these have to return the values the app writes, as
  `spaceport-ago` does), `crate::release_notes`, `crate::browser::WebUrl`,
  `crate::terminal::file_target`, `crate::chat::visual` (whose `Visuals` holds
  `gpui::RenderImage`) and `crate::config::Theme`/`crate::contrast`. Each of
  those has to be below the crate first, and each was paid in turn: `text.rs`
  and `highlight.rs` (stage 3a), the rich block model and its parse with the two
  destinations injected as policies — `parse` and `Document` take the app's
  `destination` and `is_local_path` instead of the crate reading `WebUrl` or
  `file_target` (stage 3b), the release notes' line model with the prepared
  entry model and the widgets (stage 3c), and the selectable text (stage 3d).
  `prepared.rs` stays as the shell that pairs the crate's model with the
  window's `Visuals` and selectable `Document`; `selection.rs` and `rich.rs`
  stay as a facade and as the link policy at the paths every importer already
  uses. The `Visuals` sink is why a `visual` without `RenderImage` was never
  needed: the crate reports its wants and the app answers them.
- **Stage 4, everything that paints or holds an entity:** `view/` (the largest
  group), `widgets/state.rs`, `visual/`, `files.rs`, `pasted.rs`, `page.rs`,
  `harnesses.rs`, `start.rs`, `preview/`, `window.rs`, `chats.rs`, `reader.rs`,
  `discovery.rs`'s `LiveState` reads, `core_pane.rs`, `glide.rs`, `terminal.rs`.
  These need `SpaceportWindow`, `Entity`, `ListState` or `LiveState`. The
  live-state crate they waited on landed in #232, and the first slice of this
  stage with it: #234 moved `Route` and the local-daemon trust boundary into
  `spaceport_chat_core::discovery`, reading `spaceport_live::LiveState`. What is
  left is everything that holds an entity — `view/` alone is 10,005 lines with
  its tests — the pane list with `Target`/`Source`/`Surface` (which would drag
  `spaceport_backend::TerminalId` into the crate), and the start fence, which is
  the second and third spelling of `endpoint/fence.rs`'s value.

Stage 3's messages are what the attach error is displayed through, so its
`crate::i18n::fl` calls have to move the same way stage 2's arms did: the crate
returns the values, the app writes the words (`docs/i18n.md`). Stage 3a is the
pattern for the rest: `text.rs`'s 12 messages stayed in the app with the
typed `Elapsed` and `Measured` beside them, and no call site changed.

### `endpoint`, in five stages

The design is [`notes-endpoint-extraction.md`](notes-endpoint-extraction.md). All
five stages landed in one wave, one PR each, and each has its own note:
[`wx57`](notes-wx57.md), [`wx59`](notes-wx59.md), [`wx61`](notes-wx61.md),
[`wx64`](notes-wx64.md) and [`wx66`](notes-wx66.md).

| Stage | What crossed the line | Landed | Waited on |
| --- | --- | --- | --- |
| 1 | `History` and `Step`, `WakeClock` with `Trigger` and the macOS network monitor, `order::{rank, moved}`, `focus_requested`, the endpoint-ID policy, and the follow rule → `crates/spaceport-endpoint` | #226 | nothing: it is the dependency-free policy |
| 2 | `Catalog`, the saved-host storage step and the device list it describes → `crates/spaceport-endpoint` | #228 | stage 1 |
| 3 | `LiveState`, `ConnectionStatus`, the activation fence, the projection, `Notices` and the notification model → `crates/spaceport-live`, with one `Labels` struct for the seven `fl!` sites that would have crossed | #232 | stage 1. Chat stage 4 and the sidebar's `&LiveState` reads wanted it too |
| 4 | `Link`, `Failure`, `Transition`, `Phase`, the fence values and `live_of`'s rule, behind `spaceport_endpoint::Host`, which returns typed `Action`s the window performs on its socket | #233 | stage 3 |
| 5 | `HostView`: six prepared fields the four sidebar row and rail renderers take instead of `&crate::endpoint::Endpoint` | #235 | stage 4 |

Stage 3's one `Labels` struct is gone: `spaceport-live` words its own notices
through `spaceport-i18n` and takes only the two things it cannot word — a
client error's sentence, and what an agent notice is about — as `fn` items
([`wx84`](notes-wx84.md)). The same seam went from the Settings kit, the chat's
prepared text and the sidebar's Orca chip.

Three groups stay in the app, and the design's
[honest-remainder table](notes-endpoint-extraction.md) accounts for each. `ConnectionBridge` in
`connection.rs` with `connection/reader.rs` (766 lines together) stay until
`daemon`, `links`, `scrollback`,
`terminal::InputTarget` and `config::Theme` all have homes — four other
programs. `Release`, `Redraw`, `polling.rs` (496) and `reset_selected` stay
because they are the window's own state and its agreement about a socket. The
socket-driving lifecycle suite, 10,710 lines that open real sockets and
processes, stays forever. So the app still holds 2,069 production and 12,544
test lines of the endpoint tree, against the 1,659 production and 1,163 test
lines the two crates hold. Two follow-ups are open by decision, not by omission:
31 production sites still read `endpoint.live()` instead of `live_of(index)`, so
the window's stamped copy and a host's copy can disagree between a poll and a
reset — that is what `HostView` started to fix; and `Endpoint::generation` stays
app-side until the fences get a crate of their own.

### `menus`, in five stages

The design is [`notes-menus-extraction.md`](notes-menus-extraction.md). Each
landed stage has its own note: [`wx55`](notes-wx55.md) (stage 1),
[`wx56`](notes-wx56.md) (stage 2), [`wx60`](notes-wx60.md) (stage 3),
[`wx63`](notes-wx63.md) (stage 4) and [`wx67`](notes-wx67.md) (stage 5a).

| Stage | What crossed the line | Landed | Waited on |
| --- | --- | --- | --- |
| 1 | The page set (`Page`, `WorkspaceAction`), the workspace target with its request, and `pane_menu`/`tab_menu`/`group_menu`'s targets and `(Method, Value)` payloads → `crates/spaceport-menu` | #223 | nothing |
| 2 | `Appear` with the rows it keeps, the device picker's lines, the sessions popover's row values | #225 | stage 1 |
| 3 | The palette's ranking model → `crates/spaceport-palette`: `Filter`, `Action`, `Identity`, `Entry`, `Target`, `rank_rows`, `search`, `AgentFilter`, `frecency::Key`, the project values and all 21 error variants | #229 | stage 1, and `SharedString` becoming `String` in a row's label |
| 4 | The popup chrome and the in-window bar; `spaceport-menu` gains `gpui` and becomes a GPUI crate | #231 | stage 2, and `spaceport-widgets` (#220) for the logical sides and the AccessKit helpers |
| 5a | `menu/devices/setup.rs` and its error → `crates/spaceport-remote` | #236 | nothing: it is the one page body with no text field, endpoint state or keymap in it |
| 5b | The rest of the page bodies, cheapest first: the host menu's actions, the workspace popover, the tab and group page bodies, `pane_menu`, the Keybinds page, then Add Device, WSL and the device picker | open | P2, the text-field crate (`DialogInput`, every form and picker — `SearchInput` and the shared editing model landed in `crates/spaceport-textfield` with wx82) and the owner's answer to P1, where a crate's words live — answered 2026-11-25: the lookup moves into `spaceport-i18n` (wx83 landing it). `spaceport-keymap` is done (all four stages: #239, #243, #249) and the shortcut editor reads the crate directly; `spaceport-endpoint` landed as #236/#235-era stages |

The program was never going to move most of the module: the design counted
~4,300–4,900 lines of 20.6k as movable, and the three-quarters that stay are the
pickers and forms (endpoint state, text fields, workers), the dialogs and
shortcut editor, the paint of every page, and the application menu bar. Today
`spaceport-menu` holds 2,067 production and 966 test lines, `spaceport-palette`
860 and 379, and 10,865 production lines are still app-side in `src/menu*`,
`src/menus` and the two menu roots. So the sidebar's `menu` blocker is smaller by
the page set and the ranking, not by the pickers.

### `settings`, in seven stages

The design is [`notes-settings-extraction.md`](notes-settings-extraction.md), and
its stages have their own notes: [`wx48`](notes-wx48.md) (stage 1),
[`wx49`](notes-wx49.md) (P3), [`wx50`](notes-wx50.md) (the source snapshot),
[`wx51-rowkit`](notes-wx51-rowkit.md) (stage 2),
[`wx58-pagestate`](notes-wx58-pagestate.md) (stage 3) and
[`wx62`](notes-wx62.md) (stage 4).

| Stage | What crossed the line | Landed | Waited on |
| --- | --- | --- | --- |
| P3 | The widget layer — `direction`, `toggles`, `accessibility` — → `crates/spaceport-widgets`, with the app's façades left in place | #220 | nothing, and it had consumers far outside Settings |
| 1 | The draft and save-queue policy, the section and group list, and the settings catalog → `crates/spaceport-settings` | #219 | nothing, but it is the only place the save rules can be tested without GPUI |
| P4 | The source snapshot: one value the crate reads its config, theme and keys from, instead of 15 `SpaceportWindow` reads | #222 | nothing |
| 2 | The row kit — `ui.rs`, `ui/{number,popover,swatches}.rs`, the row bounds, `control_card` and `control_switch` — → `crates/spaceport-settings-window`, with words as data and a `native.rs` probe for its own tests | #224 | P3 |
| 3 | `PageState` and `PageText`: each page draws a prepared page instead of reading `self.config`, `self.shared` or `self.theme` | #227 | stage 2 |
| 4 | The section list and footer, `scrollbar`'s `scroll_area`, the number and dropdown previews → `crates/spaceport-settings-window`, driven by two closures the window answers | #230 | stages 2-3. `panel_resize` (P6) and `SearchInput` (P2) stayed app-side, so `tests/navigation_resize.rs` and the input tests stayed with them |
| 5 | The theme browser: `themes.rs` with its `appearance`, `grid` and `system` pages | open | P2 (it owns an `Entity<SearchInput>`) and P5 (the appearance service: `InitialAppearance`, `log_window`, `rebind_keys`) |
| 6 | The eleven page modules and `controls/*`, 3,692 production lines | open | P1, the owner's decision on where the shown language's words live. If the answer is no, this stage does not happen and the extraction stops after stage 5 |
| 7 | Record it: this row, `AGENTS.md`'s crate table, the stage tables above | part landed here | finishes when stage 6 does |

`spaceport-settings-window` holds 1,825 production and 1,356 test lines, and
`spaceport-widgets` 513 and 128. The app's `src/settings_window*` still holds
10,476 production and 8,571 test lines. Whoever takes P6 should move
`panel_resize` into `spaceport-widgets` and take `tests/navigation_resize.rs`
with it: that test drags the settings navigation edge and asserts `PanelWidth`'s
default and cap, so it is a `panel_resize` test wearing a settings name.

## What this does not do

No behaviour changes, no new dependencies, no file reshuffles beyond the module
that moved, and no change to another crate. `spaceport-gpui` still links
`mermaid-rs-renderer`, `ratex-*` and `image` until `chat` moves; that
is the commit that shrinks the build next. `syntect` is no longer one of its own
dependencies since stage 3a, though the binary still links it below
`spaceport-chat-core`.
