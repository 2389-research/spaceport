# Splitting `crates/sheprd-gpui`

`crates/sheprd-gpui` is ~310k lines over 1894 files and links the app's heaviest
dependencies (rodio, mermaid-rs-renderer, ratex-*, image, nucleo-matcher,
syntect) for every change, whether or not it touches them. This note records the
first extraction, the pattern it proved, and the dependency analysis for the
next ones. Update it as each step lands.

## What moved

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

`sound` also has no GPUI use, and it is the extraction that would actually drop a
dependency (`rodio`) from `sheprd-gpui`. It is not a thin slice: `Policy::poll`
takes `&mut LiveState`, and `Settings` loads through `crate::config` and
`crate::lenient`. See the next-extraction table.

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
| `words` | 35 / 41 | 2 | none: `unicode-segmentation` only |
| `osc52` | 122 / 0 | 2 | none: own base64 decoder, no dependencies |
| `reorder` | 170 / 0 | 4 | inline tests must move to `tests.rs`; std only |
| `lenient` | 107 / 0 | 4 | `serde`/`toml` only; four settings files import it |
| `truncate` | 105 / 86 | 5 | `one_line` calls `notifications::safe_text`, which is `sheprd_client::text::safe_text`, so the crate would depend on `sheprd-client` |
| `sound` | 848 / 1170 | 16 | the `rodio` win, but `Policy::poll(&mut LiveState)` (4 fields), `config::view_settings_path`, `config::SoundConfig`, `lenient`, and the root `Error::Sound` variant all have to move or invert first |

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
(`terminal_painter`, `config`, `notifications`) are below it. Moving `sound`
first removes `rodio` and proves the port pattern those features need.

## What this does not do

No behaviour changes, no new dependencies, no file reshuffles beyond the module
that moved, and no change to another crate. `sheprd-gpui` still links `rodio`,
`mermaid-rs-renderer`, `ratex-*`, `image` and `syntect` until `sound` and `chat`
move; those are the commits that shrink the build.
