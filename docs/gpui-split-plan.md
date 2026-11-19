# Splitting `crates/sheprd-gpui`

`crates/sheprd-gpui` is ~313k lines over 1911 files and links the app's heaviest
dependencies (mermaid-rs-renderer, ratex-*, image, nucleo-matcher, syntect) for
every change, whether or not it touches them. This note records the
extractions so far, the pattern they proved, and the dependency analysis for the
next ones. Update it as each step lands.

## What moved

Five modules have left the app, each one a pure function of its inputs with no
`gpui` types:

| Crate | From | Dependencies | Files importing it |
| --- | --- | --- | --- |
| `crates/sheprd-ago` | `crates/sheprd-gpui/src/ago.rs` | none | 1 (`palette/render.rs`) |
| `crates/sheprd-lenient` | `crates/sheprd-gpui/src/lenient.rs` | `serde`, `toml` | 4 settings files |
| `crates/sheprd-osc52` | `crates/sheprd-gpui/src/osc52.rs` | none | 2 (`state.rs` and its tests) |
| `crates/sheprd-sound` | `crates/sheprd-gpui/src/sound/` | `rodio`, `serde`, `sheprd-lenient`, `sheprd-protocol`, `thiserror`, `toml`, `tracing` | 17 |
| `crates/sheprd-words` | `crates/sheprd-gpui/src/words.rs` | `unicode-segmentation` | 2 (`multiline_input.rs`, `text_editing/keys.rs`) |

### `sheprd-ago`

`crates/sheprd-ago` is a new workspace member with **no dependencies at all**:

| Before | After |
| --- | --- |
| `crates/sheprd-gpui/src/ago.rs`, 32 lines | `crates/sheprd-ago/src/lib.rs`: `Elapsed` and `elapsed(seconds)` |
| its unit table in `src/ago/tests.rs` | `crates/sheprd-ago/src/tests.rs` |
| its message table and `text()` | stay in `crates/sheprd-gpui/src/ago.rs`, 34 lines |

`ago` was picked over `sound`, `app_badge` and `about` because it is the only
one of the four with no `gpui` types and a single importer:

| Module | production lines | `gpui::` uses | files importing it |
| --- | --- | --- | --- |
| `ago` | 32 | 0 | 1 (`palette/render.rs`) |
| `sound` | 848 | 0 | 16 |
| `about` | 258 | 4 | 3 |
| `app_badge` | 302 | 25 | 3 |

### `sheprd-osc52`

`crates/sheprd-osc52` is the second, also with **no dependencies at all**. The
module file also held its tests; they moved out of the production file, and the
app keeps only the call site:

| Before | After |
| --- | --- |
| `crates/sheprd-gpui/src/osc52.rs`, 122 production lines and 3 inline tests | `crates/sheprd-osc52/src/lib.rs`: `MAX_TEXT_BYTES`, `MAX_PENDING`, `decode(data)` |
| its three inline tests (20 lines) | `crates/sheprd-osc52/src/tests.rs` |
| `src/state.rs` decoding a payload and bounding its mailbox | stays: `sheprd_osc52::decode`, `sheprd_osc52::MAX_PENDING` |
| `src/state/tests/daemon_messages.rs` reading the mailbox bound | stays: `sheprd_osc52::MAX_PENDING` |

It has no messages and no error type of its own: `decode` returns `None` for a
payload it will not write, and the app logs the drop.

### `sheprd-words`

`crates/sheprd-words` depends only on `unicode-segmentation`, and its tests were
already in `src/words/tests.rs`, so they moved as they were:

| Before | After |
| --- | --- |
| `crates/sheprd-gpui/src/words.rs`, 35 lines | `crates/sheprd-words/src/lib.rs`: `previous_start(text, offset)` and `next_end(text, offset)` |
| `crates/sheprd-gpui/src/words/tests.rs`, 41 lines | `crates/sheprd-words/src/tests.rs` |
| `src/multiline_input.rs` (double-click selection) and `src/text_editing/keys.rs` (word movement) | both call `sheprd_words::` |

It carries no messages either: the fields that use it own their own text.

### `sheprd-lenient`

`crates/sheprd-lenient` depends only on `serde` and `toml`. Herdr writes
`config.toml` and this app does not own it, so every value the build cannot read
falls back to its own default: a property of a deserializer, not of the app.

| Before | After |
| --- | --- |
| `crates/sheprd-gpui/src/lenient.rs`, 107 lines | `crates/sheprd-lenient/src/lib.rs`, 51 lines: `value`, `or_default`, `or_true`, `entries` |
| its inline test | `crates/sheprd-lenient/src/tests.rs`, 56 lines |
| 44 `crate::lenient::` paths in four settings files | the same paths naming `sheprd_lenient` |

It has no messages and no error type: a value it cannot read falls back, and
malformed TOML fails with `toml`'s own error in the caller.

### `sheprd-sound`

The first extraction that took a dependency out of the app rather than just
lines: `rodio` and the audio stack below it (`cpal`, CoreAudio, `symphonia`)
left `sheprd-gpui`'s build graph for good.

| Before | After |
| --- | --- |
| `crates/sheprd-gpui/src/sound.rs`, 455 lines | `crates/sheprd-sound/src/service.rs` (the worker) and `src/policy.rs` (`Policy`, `Live`, `MAX_PENDING`) |
| `src/sound/config.rs`, 196 lines | `src/config.rs`: `Settings::load(path)`, no path policy of its own |
| `src/sound/playback.rs`, 159 lines | `src/playback.rs`: `MAX_VOLUME`, `muted`, and rodio playback |
| `src/sound/error.rs`, 47 lines | `src/error.rs`: the crate's own `thiserror` `Error`, with `Io`, `Path` and `Toml` added for settings |
| 1,170 test lines | 29 crate tests alongside the code they drive; 3 tests that exercise the seam from the app side stay in `connection/tests/sounds.rs` and `state/tests/sounds.rs` |

Three couplings had to invert first, each one parameter or struct:

- `Policy::poll` took `&mut LiveState`, which is the app's. It now takes
  `sheprd_sound::Live`: the two cancellation tokens, whether the daemon is
  connected, the reload flag, the queue it drains, and the newest snapshot. The
  app builds it in one place, `LiveState::sound_live`.
- `Settings::load` resolved its own path through `config::view_settings_path`,
  which is the app's policy. It takes the file now; `Service::new(path, volume)`
  gets both from `app::shared_sound`, resolved once at construction.
- `Service` held `Arc<Mutex<Option<Arc<Settings>>>>` and the app called
  `ui_delay`, `path_for` and `allows` on it. Those became `pub(crate)`, and the
  app only calls the worker.

`crates/sheprd-truncate` depends on `unicode-segmentation`, `unicode-width` and
`sheprd-client`:

| Before | After |
| --- | --- |
| `crates/sheprd-gpui/src/truncate.rs`, 105 lines | `crates/sheprd-truncate/src/lib.rs`, 105 lines: `cut`, `cut_chars`, `cut_width`, `one_line`, `ellipsize`, `tab_title`, `ELLIPSIS` |
| `crates/sheprd-gpui/src/truncate/tests.rs`, 86 lines | `crates/sheprd-truncate/src/tests.rs`, moved unchanged |
| 6 `crate::truncate::` paths in five files | the same paths naming `sheprd_truncate`; `one_line` calls `sheprd_client::text::safe_text`, which it had reached through `crate::notifications` |

## The pattern

1. **Pick by footprint, not by size.** For each candidate module count what it
   imports (`crate::`, `gpui::`) and how many files import it back:
   `rg -l 'crate::<module>\b' crates/sheprd-gpui/src`. Fewest reverse importers
   first; a module that renders or holds a GPUI entity is not a candidate yet.
2. **Take a crate with its own tests.** New `crates/sheprd-<module>/Cargo.toml`
   copies `[package]`'s workspace fields from `crates/sheprd-i18n`, sets
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
   typed values and the app writes them. `sheprd-ago` names the unit and its
   count; `crates/sheprd-gpui/src/ago.rs` keeps `ago.ftl`'s IDs. A crate that
   names message IDs as data instead follows `sheprd-commands`/`sheprd-usage`.
6. **Wire and record.** Workspace `members`, `[workspace.dependencies]`,
   `sheprd-gpui`'s `[dependencies]`, the crate table in `AGENTS.md`, and the
   layout in `docs/i18n.md` when the module carries text.
7. **Verify before handoff:** `cargo check -p <crate> -p sheprd-gpui`,
   `cargo test -p <crate>`, `cargo fmt --all -- --check`, `just check-file-size`,
   `just check-sorted`, `just check-i18n`. `cargo metadata` refreshes
   `Cargo.lock`, which the commit includes.

## Next extractions

Cheap, dependency-free moves first; the blocked ones carry real work.

| Module | production / test lines | importers | blockers |
| --- | --- | --- | --- |
| `sound` | 848 / 1170 | 16 | the `rodio` win, but `Policy::poll(&mut LiveState)` (4 fields), `config::view_settings_path`, `config::SoundConfig`, `lenient`, and the root `Error::Sound` variant all have to move or invert first |

`reorder` was the next cheap one and did not land in this wave: both extractions
above compile `sheprd-gpui`, and the local disk floor (3 GiB free) stopped
further builds before a third one. It needs no design work when disk allows it —
`reorder.rs` is std-only and its inline tests move to `src/tests.rs`.

`chat`, `browser` and `sidebar` are the app's three large features and each is a
program of its own; none can move before the state and text they read have
homes. Counted today, production lines first:

| Feature | production / test lines | `gpui::`/entity uses | i18n messages | files importing it | what blocks it |
| --- | --- | --- | --- | --- | --- |
| `browser` | 4,748 / 3,064 | 55 | 10 | 53 | reads `endpoint` and `group_terminals` and paints the tab strip from them; wants those value types below it first |
| `chat` | 20,440 / 16,011 | 128 | 174 | 12 | GPUI views all the way down; also imports `browser`, `terminal_painter`, `notifications`, `controls`, `actions`, `constants`, `motion`, `contrast`, `bundled` |
| `sidebar` | 11,195 / 14,506 | 162 | 75 | 263 | the most-imported module in the crate; reads `config`, `menu`, `keymap`, `state`, `endpoint`, `icons`, `worktree_notes`, `device_tags` |

Order that falls out of the counts: `browser` before `sidebar` (53 against 263
importers), and `chat` only after `browser` and the shared paint/settings types
(`terminal_painter`, `config`, `notifications`) are below it. `sound` proved the
pattern a feature needs: a value type for what the crate reads, and a parameter
for what the app owns.

## What this does not do

No behaviour changes, no new dependencies, no file reshuffles beyond the module
that moved, and no change to another crate. `sheprd-gpui` still links
`mermaid-rs-renderer`, `ratex-*`, `image` and `syntect` until `chat` moves; that
is the commit that shrinks the build next.
