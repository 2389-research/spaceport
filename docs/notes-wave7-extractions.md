# Wave 7: three more modules out of `crates/sheprd-gpui`

`reorder`, `lenient` and `truncate` moved into their own workspace members,
following the pattern `crates/sheprd-ago` proved
(`docs/gpui-split-plan.md` § The pattern). No behaviour changes, no new
dependencies, no reshuffles beyond the three modules, and every commit builds on
its own.

## What moved

| Crate | Before | After | Dependencies |
| --- | --- | --- | --- |
| `sheprd-reorder` | `crates/sheprd-gpui/src/reorder.rs`, 170 lines, three inline tests | `src/lib.rs` 109 lines, `src/tests.rs` 59 lines | none |
| `sheprd-lenient` | `crates/sheprd-gpui/src/lenient.rs`, 107 lines, one inline test | `src/lib.rs` 51 lines, `src/tests.rs` 56 lines | `serde`, `toml` |
| `sheprd-truncate` | `crates/sheprd-gpui/src/truncate.rs`, 105 lines, plus `src/truncate/tests.rs`, 86 lines | `src/lib.rs` 105 lines, `src/tests.rs` 86 lines, moved unchanged | `unicode-segmentation`, `unicode-width`, `sheprd-client` |

`pub(crate)` became `pub` only for what the app calls. Each crate keeps the doc
comment that explains its job and has no `error.rs`, since none of the three has
an error path. None of the three names a message, so nothing moved in
`crates/sheprd-gpui/locales`.

## Call sites

| Module | Files calling it | Shape of the change |
| --- | --- | --- |
| `reorder` | 13, four of them through a `crate::reorder::` path and nine through `use crate::{… reorder::…}` | `crate::reorder::X` → `sheprd_reorder::X`, or the import moved to a `use sheprd_reorder::…` line |
| `lenient` | 4 (`herdr_settings`, `herdr_settings/palette`, `herdr_settings/remote`, `sound/config`), 44 paths | `crate::lenient::` → `sheprd_lenient::` in `#[serde(deserialize_with = "…")]` strings and two calls |
| `truncate` | 5 (`browser/view`, `config_diagnostic`, `sidebar/search`, `window/report_issue`, `window/tab_strip/agent_title`), 6 paths | `crate::truncate::` → `sheprd_truncate::` |

`sidebar::reorder` stays the sidebar's own module: it is the drag state machine
for workspace rows, and only its geometry helpers moved below.

`sheprd-truncate`'s `one_line` called `crate::notifications::safe_text`, which is
only a re-export of `sheprd_client::text::safe_text`; it now names that directly,
so the crate depends on `sheprd-client` and the app keeps one implementation. The
direction is UI → client, which `sheprd-gpui` already had.

`sheprd-reorder` keeps a fixed 150 ms slide duration for normal consumers.
GPUI's motion policy alone selects instant settling under `cfg(test)`.
This preserves headless layout tests while native test binaries retain real motion.
The temporary dependency feature also affected `CARGO_BIN_EXE_spaceport` through Cargo feature unification.
The integration removes that feature and covers midpoint progression from a normal dependency consumer.

## Gates

Every command ran in `.worktrees/wave7-extract`.

| Command | Result |
| --- | --- |
| `cargo test -p sheprd-reorder --offline` | passed: 3 tests |
| `cargo test -p sheprd-lenient --offline` | passed: 1 test |
| `cargo test -p sheprd-truncate --offline` | passed: 7 tests |
| `cargo check -p sheprd-gpui --tests --offline` | passed on the full three-extraction tree in 1 m 07 s, so every importer's use path and test module type-checks |
| `cargo fmt --all -- --check` | passed |
| `just check-file-size` | passed: all source files within 1000 lines |
| `just check-sorted` | passed: 10 script tests, all `keep-sorted` regions sorted |
| `just check-commits` | passed: `origin/main..HEAD` |
| `cargo metadata --offline` | refreshed `Cargo.lock` after each extraction; each commit carries its own lock entry |

## Deferred to CI

The focused `sheprd-gpui` test runs (sidebar row drag, device drag, workspace
drag phases, tab strip settle) and clippy were **not** run locally: both need the
`sheprd-gpui` codegen and link, and this volume is near-full.

- `cargo test -p sheprd-gpui --no-run` took free disk from 4.3 GiB to 2.5 GiB in
  30 seconds, under the 3 GiB floor this wave was told to stop at. The build was
  killed, `target/debug/incremental` and then `target/` were removed, and the
  supervisor confirmed the deferral.
- The type-level check above still covers the risk these edits carry: they only
  rename paths and move bodies. The moved test bodies are the ones that passed in
  the app before the move, unchanged apart from losing the `mod tests { … }`
  wrapper and its indentation.
- CI's `checks` job runs `cargo test` for the workspace with default and all
  features, and lints every target; that is where the focused runs and clippy
  land.

## Commits

| Commit | Contents |
| --- | --- |
| `refactor: Move drag reordering into the sheprd-reorder crate` | the crate, 13 call sites, workspace wiring, crate table, split note |
| `refactor: Move lenient settings decoding into the sheprd-lenient crate` | the crate, four settings files, wiring, note |
| `refactor: Move text truncation into the sheprd-truncate crate` | the crate, five files, wiring, note |

`docs/gpui-split-plan.md` records the three under "What moved" with the same
before/after evidence `ago` carries, and drops their rows from "Next
extractions". `words` and `osc52` still list there: they are done on a parallel
branch that has not merged, and this branch does not touch their rows.
