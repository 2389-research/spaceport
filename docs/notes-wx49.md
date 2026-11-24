# wx49: the chrome's shared widget kit becomes `crates/spaceport-widgets`

Worktree: `pi-worktree-3a436e0a-c065-48ac-8a03-4023dc4e092c-s0-0`
Base: `8b9289f51` (origin/main, after #216, #217 and #218)
Branch: `feat/wx49-widgets`

Prerequisite **P3** of [`notes-settings-extraction.md`](notes-settings-extraction.md)
§2.2: `direction`, `toggles` and `accessibility`, 289 production lines the
Settings UI needs and five other surfaces already draw with. That note counted
only Settings as the consumer; §1 below is the survey that says otherwise. This
is a move and a facade, not a behaviour change.

## 1. The survey: who imports the three modules

Counted on `origin/main` with `git grep -l "crate::<module>::" crates/spaceport-gpui/src`.

| Module | Files | Call sites | Where |
| --- | --- | --- | --- |
| `direction` | 49 | 59 | 12 `settings_window*`, 15 sidebar (`sidebar/*` and `core_terminals/sidebar.rs`), 5 titlebar, 9 menus, popups, the palette and `dialog_input.rs`, 3 `window/*`, 2 `tab_layout*`, `panel_resize.rs`, `search_input.rs`, and `toggles.rs` itself |
| `toggles` | 8 | 8 | `chat/view/widgets/components.rs` (`switch`), `close_modal.rs`, `menu/devices/host_menu.rs` and `host_menu/tags.rs` (`checkbox`), `menu/devices/wsl.rs` (`radio`), `settings_window/{ui.rs, controls/forms.rs}`, `settings_window/ui/tests.rs` (`colors`) |
| `accessibility` | 9 | 14 | `chat/view/widgets/components.rs`, `menu/workspace/popover.rs`, `menus/in_window/render.rs` (4), `settings_window/{ui.rs, controls.rs, controls/forms.rs}` (5), `sidebar/accessibility.rs`, `titlebar/navigation.rs`, `window/config_diagnostic.rs` |

The design's "12 files" for `direction` was Settings alone. The real surface is
four times that, which is the argument for the crate rather than against it: the
trait's 34 `Logical` importers, the sidebar's 14 test and render files, and the
titlebar's five all move to `spaceport_widgets::direction::Logical` the day they
leave the app, without changing a call site.

## 2. What moved

| Before (`origin/main`) | Lines | After |
| --- | --- | --- |
| `crates/spaceport-gpui/src/direction.rs` | 147 | `crates/spaceport-widgets/src/direction.rs` (188: the direction store and its docs are new) |
| `crates/spaceport-gpui/src/direction/tests.rs` | 98 | `crates/spaceport-widgets/src/direction/tests.rs` (94) |
| `crates/spaceport-gpui/src/toggles.rs` | 97 | `crates/spaceport-widgets/src/toggles.rs` (97) |
| `crates/spaceport-gpui/src/toggles/tests.rs` | 34 | `crates/spaceport-widgets/src/toggles/tests.rs` (34) |
| `crates/spaceport-gpui/src/accessibility.rs` | 45 | `crates/spaceport-widgets/src/accessibility.rs` (45) |

The bodies are the same code: `toggles` and `accessibility` changed only from
`pub(crate)` to `pub` and from the app's façades (`crate::config::{Theme,
corners}`, `crate::contrast`) to the crate's own paths
(`spaceport_settings::{Theme, corners}`, `spaceport_settings::contrast`).

`crates/spaceport-widgets` depends on `gpui`, `spaceport-i18n` (the `Direction`
the types speak), `spaceport-settings` (the theme a mark is coloured by) and
`spaceport-sidebar` (`direction`'s rules, see §3.2). `spaceport-icons` is a dev
dependency for the icon-mirror test. There is no `error.rs`: nothing in the three
modules can fail, so a root `Error`/`Result` would be an enum with no producer —
`spaceport-motion`, `spaceport-reorder`, `spaceport-truncate`, `spaceport-words`,
`spaceport-lenient` and `spaceport-ago` are the crate's precedent.

`Cargo.toml` gains the member and the workspace dependency; `Cargo.lock` gains
the package entry and no version bumps.

## 3. The boundary decisions

### 3.1 The direction is a value the crate reads, published by `i18n`

`Logical`'s methods take no `cx` and 59 call sites read the direction while they
build an element, so the direction cannot be a parameter without touching every
one of them. It is now the crate's: `spaceport_widgets::direction::{set,
current}`, a process-wide `AtomicBool` that `i18n/current.rs::replace` writes as
it loads the catalog (`spaceport_widgets::direction::set(
catalog.locale().direction())`). `crate::i18n::direction()` and the app's own
`RTL` flag are gone; the flag has one owner and one writer, the same place that
switches the text.

The crate keeps the app's test isolation through a `test-support` feature: under
it (and under `cfg(test)`, for the crate's own tests) the flag is a
`thread_local` `Cell<bool>`, so a test that shows `en-XB` mirrors nothing but its
own layout, as `i18n`'s per-thread catalog already promised. `spaceport-gpui`
enables the feature in its dev-dependencies, the way it already does for
`spaceport-sidebar` and `spaceport-settings`. Without it, a process-wide flag
would leak a pseudo-locale between the 3,494 tests that run in parallel.

The `en-XB` → mirrored-layout link therefore lives app-side now:
`i18n/tests/locale.rs`'s pseudo-locale test asserts `crate::direction::current()`
follows `toggle_pseudo`, and the crate's own tests set a `Direction` directly.

### 3.2 The mirror rules stay in `spaceport-sidebar`

`direction`'s every method delegates to `spaceport_sidebar::direction`, keyed
there on a `bool` because a row's painter cannot read a locale. That does not
change, so `spaceport-widgets` depends on `spaceport-sidebar` for it. Moving the
rules up instead would put them below the row painters that use them and would
touch a crate extracted two PRs ago — a different refactor, and one that can
land later without changing this crate's call sites, since they name
`mirror::Logical` through one import.

### 3.3 The app keeps a façade at each old path

`crates/spaceport-gpui/src/{direction,toggles,accessibility}.rs` are now three
`pub(crate) use spaceport_widgets::<module>::*;` lines with the module doc
saying where the code lives. Every one of the 59 importing files, and the
`debug_selector` strings the Settings tests select by, stays as it was — the
risk `notes-settings-extraction.md` §5.1 ranks sixth. Each façade removes with
the feature that moves its callers.

## 4. What stays app-side

| Item | Lines | Why |
| --- | --- | --- |
| `direction.rs`, `toggles.rs`, `accessibility.rs` façades | 24 | the paths 59 files and their tests already name |
| `i18n/current.rs`'s publish call | 1 | the app owns the locale; the crate holds what it mirrors |
| `window/accessibility.rs`, `terminal/accessibility.rs`, `sidebar/accessibility.rs` | — | different modules that only share a name; untouched |

## 5. Tests

The six moved tests are the crate's: five `direction` tests (every physical side
under `Ltr` and `Rtl`, anchors, `toward_end`, and both directional tables checked
against the embedded icons) and `toggles`' contrast test over every builtin theme
at both contrast levels. `docs/i18n.md` names the new path in its layout list and
in "Right to left", and `AGENTS.md`'s crate table has the row.

| Check | Result |
| --- | --- |
| `cargo check -p spaceport-widgets --all-targets` | pass |
| `cargo clippy -p spaceport-widgets --all-targets -- -D warnings` | pass |
| `cargo test -p spaceport-widgets` | 6 passed, 0 failed |
| `cargo check -p spaceport-gpui --all-targets --features integration-test` | pass |
| `cargo clippy -p spaceport-gpui --all-targets --features integration-test -- -D warnings` | pass |
| `cargo test -p spaceport-gpui --bin spaceport` | 3,494 passed, 0 failed, 10 ignored |
| `cargo fmt --all --check` | clean |
| `just check-file-size`, `just check-sorted` | pass |

Run with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`, and
`SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk` for the `integration-test`
build. Left to CI: the Windows and Linux lint jobs (this change adds no `cfg`
gate, but the crate is new on both), and native screenshot QA — the moved bodies
are unchanged, so the Settings window and the menus should draw exactly as
before, which a reviewer can still confirm by eye.

## What this does not do

No `docs/gpui-split-plan.md` row: that table is edited by every extraction and
the entry belongs with the Settings design's stage 7, which also records the
remaining prerequisites. The app's `crate::config`/`crate::contrast` façades and
`spaceport-sidebar::direction` are untouched. No file reaches 1,000 lines
(`direction.rs` is the largest at 188).
