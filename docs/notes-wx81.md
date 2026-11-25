# wx81 — keymap stage 4: flatten the `crate::keymap` façade

Crew task wx81, the optional last stage of `docs/notes-keymap-extraction.md`.
Stage 4 is a rename pass and nothing else: every `crate::keymap::X` in
`crates/spaceport-gpui/src` now names the crate directly, and the re-export
block that stood in for it is gone. No behaviour changed, no line moved between
crates, and `crates/spaceport-keymap` was not touched.

Fetched `origin` first and branched `feat/wx81-keymap-4` from `origin/main` =
`7c5e31ddd72060d24c1f0de38a3410199db2d1e3` (PR #240, #246, #247 and #248
merged). Every count and command below ran against that tree.

## 1. What the façade was

`crates/spaceport-gpui/src/keymap.rs` was 20 lines: a doc comment, the app's two
remaining keymap tests, and this block.

```rust
pub use spaceport_keymap::daemon;
pub use spaceport_keymap::daemon::DaemonKeys;
#[cfg(any(test, feature = "integration-test"))]
pub use spaceport_keymap::parse_keystroke;
pub use spaceport_keymap::{Error, HOST_OS, Keymap, Reach, available_commands, secondary};
#[cfg(test)]
pub use spaceport_keymap::{identity, secondary_key, typed_matches};
pub use spaceport_settings::keys::{Binding, PaneKeys};
```

Before: 164 occurrences of `crate::keymap::` on 163 lines in 77 files, plus 15
`use crate::{…}` groups whose list named `keymap::X` rather than a `crate::keymap`
path. After: none, and 194 `spaceport_keymap::` occurrences in 87 files (most of
the growth is the longer `daemon::DaemonKeys` path).

## 2. The three name maps it needed

The rename was not uniform, because the façade re-exported three different
things under one path.

| Was | Now | Why |
| --- | --- | --- |
| `crate::keymap::DaemonKeys` (5 sites) | `spaceport_keymap::daemon::DaemonKeys` | It is a `daemon` module item; only the façade had lifted it to the crate root |
| `crate::keymap::Binding`, `crate::keymap::PaneKeys` (2 + 1 sites) | `spaceport_settings::keys::{Binding, PaneKeys}` | The façade re-exported the settings crate's own types; the keymap crate never owned them |
| every other name (`Keymap`, `Error`, `Reach`, `HOST_OS`, `available_commands`, `secondary`, `PaneKey`, `parse_keystroke`, `daemon`, `identity`, `secondary_key`, `typed_matches`) | `spaceport_keymap::X` | Already at the crate root |

The 15 grouped imports were split so each name keeps its own crate's path:
`use crate::{…, keymap::{DaemonKeys, Keymap}};` became `use crate::…;` plus
`use spaceport_keymap::daemon::DaemonKeys;` and `use spaceport_keymap::Keymap;`,
and the four lists that also carried `Binding`/`PaneKeys` gained a
`spaceport_settings::keys::…` line instead.

## 3. Where the `cfg` gates went

The façade's two gates were moved, not dropped, and the crate's own gate does
the work they did:

- `parse_keystroke` was gated `cfg(any(test, feature = "integration-test"))` on
  the façade. `spaceport_keymap` exports it unconditionally, so the 48 consumer
  sites (`menus/in_window/native.rs`, `smoke*`, the lifecycle tests) needed no
  gate at all — the wider gate is strictly less restrictive.
- `identity`, `secondary_key` and `typed_matches` are gated in the crate on
  `#[cfg(any(test, feature = "test-support"))]`, and `spaceport-gpui` already
  enables `test-support` from its `[dev-dependencies]`. Every one of their
  consumers is test code (`window/links/tests*`, `window/file_links/tests*`,
  `terminal/links/tests*`, `endpoint/lifecycle_tests/*`,
  `sidebar/layout_tests/device_multi_select.rs`, and `keymap/tests`), so no
  production file gained a gated import and the `default` and `all` CI passes
  see the same code.
- The gates that stayed are the ones that live outside this façade: `actions.rs`
  keeps its own `#[cfg(any(target_os = "macos", test))] pub use
  spaceport_keymap::{SYSTEM_KEYS, free_system_keys};` and `#[cfg(test)] pub use
  spaceport_keymap::reserved_keystrokes;`. Those are `crate::actions` names with
  working consumers (`actions/tests.rs` calls `free_system_keys`), they were
  landed that way by stage 3, and stage 4 did not rename `crate::actions`.

## 4. Load-bearing re-exports left in place

None. The app has no `lib` target — `crates/spaceport-gpui/src/main.rs` declares
`mod keymap;` privately — so no external user could name `spaceport_gpui::keymap`
(`rg 'spaceport_gpui::keymap'` finds nothing in this repository), and the whole
block was internal. `crates/spaceport-gpui/src/keymap.rs` is now the module doc
plus `#[cfg(test)] mod tests;`, which keeps the two tests that are the app's
rather than the crate's: the reserved-key check in `keymap/tests/platform_defaults.rs`
and the pane serialization check in `keymap/tests/shift_enter.rs`. Their fixtures
in `keymap/tests.rs` now import the names they use (`daemon::DaemonKeys`,
`Keymap`, `PaneKey`, `identity`, `parse_keystroke`, `typed_matches`, and the
settings crate's `Binding` and `PaneKeys`) instead of globbing the façade.

## 5. Diff size

| | |
| --- | --- |
| files changed | 85 (77 by the first rename pass, 8 more by the grouped-import split and the two tests' imports) |
| insertions / deletions | 222 / 233 |
| production code changed | path text only, plus the `use` block of `keymap/tests.rs` |
| `crates/spaceport-keymap` | untouched |
| docs | this file; `docs/notes-keymap-extraction.md`'s landed markers are the supervisor's |

## 6. Validation run

| Command | Result |
| --- | --- |
| `rg -n 'crate::keymap' crates/spaceport-gpui/src` | empty (exit 1) |
| `cargo check --locked -p spaceport-gpui --all-targets` | passed |
| `python3 scripts/release/sparkle_framework.py acquire target/sparkle-sdk` | fetched 15.5 MB into `target/sparkle-sdk` |
| `SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk cargo clippy --locked -p spaceport-gpui --all-targets --all-features` | passed, no warnings from this crate |
| `cargo fmt --check` | passed |
| `cargo test --locked -p spaceport-keymap` | 54 passed, 0 failed (plus 0 doc-tests) |
| `cargo test --locked -p spaceport-gpui --bin spaceport keymap` | 9 passed, 0 failed (3350 filtered out) |
| `python3 scripts/check-sorted.py` | all keep-sorted regions sorted |
| `bash scripts/check-file-size.sh` | all source files within 1000 lines |
| `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check-i18n.py` | 2114 messages in 114 files valid and used |

Not run locally, and deferred to CI: `just test-gui` / `just test-sidebar` (need
an active desktop and a pinned Herdr), the `windows` and `checks` legs that prove
the macOS-only `SYSTEM_KEYS` gate, and `just ci`. Nothing in this diff changes
what runs at startup, so a native pass would re-prove the keymap's behaviour
rather than this commit's paths.

## 7. What is deliberately not done

- No reformatting beyond the renamed lines. `cargo fmt -p spaceport-gpui` was run
  once because splitting a grouped `use` leaves statement shells rustfmt
  collapses; the resulting diff is still path text plus the `use` block of
  `keymap/tests.rs`.
- `crate::actions`'s re-exports, `crate::config`'s `pub use spaceport_settings::…`
  block and the `spaceport_settings::keys` names the keymap façade passed through
  are all still where they were. Flattening `config.rs`'s alias list is the
  settings extraction's own closing task, not this one.
- Historical notes (`docs/notes-settings-extraction.md`, `docs/notes-wx77.md`,
  `docs/notes-keymap-extraction.md`) still mention `crate::keymap`; they record
  what the tree looked like when they were written.
