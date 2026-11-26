# wx82 — text-field stage 1: `SearchInput` and the editing model

Crew task wx82, stage 1 of the text-field crate the Settings split named **P2**
(`docs/notes-settings-extraction.md:258`, `docs/gpui-split-plan.md:393`). The
single-line field a search box is built from, and the editing model the app's
own `DialogInput` and `MultilineInput` share with it, moved from
`crates/spaceport-gpui` to a new GPUI crate, `crates/spaceport-textfield`.
`DialogInput` and `MultilineInput` themselves stayed in the app.

This note continues from the checkpoint commit `d1111a1269` on
`feat/wx82-textfield-1`. §1 is what that commit had; §2 is what finished it;
§3–§7 record the four seams the survey left open and how each was answered.

## 1. What the checkpoint had

`d1111a1269` was written against `main` at `e871fb49fc`. It created the crate,
filled it, and left the app untouched, so for one commit the same code lived in
two places:

- `Cargo.toml`: the member between `spaceport-terminal` and `spaceport-truncate`
  and the workspace dependency, with `Cargo.lock` updated.
- `crates/spaceport-textfield/Cargo.toml` — `gpui`, `spaceport-commands`,
  `spaceport-settings`, `spaceport-widgets`, `spaceport-words`,
  `unicode-bidi`, `unicode-segmentation`; dev-deps `gpui/test-support` and
  `spaceport-widgets/test-support`; the `test-support` feature.
- `src/lib.rs`, `src/locale.rs` (`LocaleChanged`, seam 1), `src/actions.rs`
  (`Cut`/`Copy`/`Paste`/`SelectAll` and `edit_key`, seam 2),
  `src/search_input.rs` (678 lines, moved with its three seams answered),
  `src/caret.rs` + `caret/tests.rs`, `src/text_editing.rs` +
  `text_editing/{keys,key_matrix,tests}.rs` + `text_editing/tests/keys.rs`,
  `src/input.rs` + `input/weak.rs` + `input/weak/tests{,/lifecycle}.rs`.
- Tests moved: `caret/tests.rs`, `text_editing/tests.rs` and
  `text_editing/tests/keys.rs`, `search_input/tests.rs` with `composition`,
  `control_keys`, `key_table` (and both pinned tables), `word_keys`, and
  `rtl_caret` rewritten against `spaceport_widgets::direction::set` so it needs
  no app. `input/weak/tests/lifecycle.rs` kept the half over `SearchInput`; the
  app's half over `SpaceportWindow` was left behind in the app.

It compiled and passed standalone: `cargo test -p spaceport-textfield` ran 40
tests green. What it did not have was any of the app's side of the move.

## 2. What this commit finished

The app side, and the four decisions the survey left open.

- **The app depends on the crate.** `spaceport-textfield` in
  `crates/spaceport-gpui/Cargo.toml`, and again in `[dev-dependencies]` with
  `test-support` so the app's own tests can still read `key_matrix`, `framed()`,
  `placeholder()` and `label()` (`cargo` unions the two for a test build, so a
  production build does not get them).
- **The moved app files are gone**: `src/search_input.rs` and `src/search_input/`,
  `src/caret.rs` and `src/caret/`, `src/text_editing.rs` and `src/text_editing/`,
  `src/input/weak.rs` and `src/input/weak/` — 3,727 lines.
- **A façade at today's paths**, so the app's own fields changed only where the
  task said they would. `src/main.rs` keeps

  ```rust
  pub(crate) use spaceport_textfield::{caret, search_input, text_editing};
  ```

  and `src/input.rs` keeps `pub(crate) use spaceport_textfield::input::ViewInputHandler;`.
  `dialog_input.rs` and `multiline_input.rs` are unchanged apart from
  `edit_key`. Naming the crate's paths at every call site — 46 files,
  `spaceport_textfield::search_input::SearchInput` — is the mechanical pass that
  `wx81` did for `crate::keymap` (notes-wx81.md), and is left as its own stage
  so this one stays a move.
- **Cut/Copy/Paste/SelectAll are declared once, in the crate.** `actions.rs`
  re-exports them, so the app's `&dyn Action` comparisons, the Edit menu's
  labels and `spaceport_keymap::EDIT_KEYS` keep working unchanged.
- **`edit_key` moved with them**, into `spaceport-textfield::actions`, because
  the crate reads `spaceport_commands::CTRL_SHORTCUTS` for the same keystroke
  the fields edit with. Six app call sites name it: four in `multiline_input.rs`,
  `chat/view.rs`, `menu/chrome/keys.rs`, plus `actions/tests/edit_keys.rs`, which
  stays app-side because it also asserts the replayed keystroke against the
  app's `edit_modifiers`.
- **The locale notice is published by the app.** `i18n::install` and the
  `switch` that a setting reload or the QA pseudo-locale reaches now call
  `spaceport_textfield::locale::LocaleChanged::set(cx)` beside
  `current::replace`, so a field holding a locale-formatted placeholder words it
  again. `i18n/current.rs`'s test half got the same treatment on its own thread.
- **The app's placeholder test moved rather than died**:
  `search_input/tests/placeholder.rs` is now `i18n/tests/field_placeholder.rs`,
  the one test that has to stay where `fl!` and `install` are, and it passes
  against the published notice (it is the proof that seam 1 works end to end).
- **The weak-handler lifecycle test split.** The `SpaceportWindow` half is
  `src/input/tests/lifecycle.rs`, gated `#[cfg(all(test, any(target_os = "linux",
  windows)))]` exactly as the app's `mod weak` used to be — on macOS the dialog's
  handler is GPUI's strong `ElementInputHandler`, so there is nothing weak to
  prove, and the test never compiled there.
- **A crate row in `AGENTS.md`**'s table, and the two planning docs marked:
  `P2` is landed for `SearchInput` and the shared model, `DialogInput` is not.

## 3. Seam 1 — the locale observer (option A)

The crate owns `locale::LocaleChanged(u64)`, a GPUI `Global` whose count only
has to differ from the last one; `SearchInput::new` observes it where the app
used to observe `i18n::CurrentLocale`. Measured against the alternative: the
field words nothing itself (no `fl!`, no message, no sentence anywhere in the
crate), so this is the whole of stage 1's i18n cost.

**Where it is published, and why not in `current.rs`.** The survey proposed
setting it beside `spaceport_widgets::direction::set` inside
`i18n/current.rs`'s `replace`. `replace` has no `App`, and it cannot borrow one:
`i18n::show(locale)` is a `#[cfg(test)]` helper for tests of text and layout
outside any window (30+ call sites), which pass no app context at all. So
`replace` now answers `bool` (`false` when the locale's catalog could not load,
which is the case that must not re-word anything), and `i18n.rs`'s two-line
`publish(locale, cx)` sets the global only when the catalog actually switched.
The behaviour asked for is the same; the four lines sit one file up.

## 4. Seam 2 — the edit actions

Declared once in `spaceport-textfield::actions` and re-exported by
`src/actions.rs`, as above. `SearchInput` and `MultilineInput` register the
crate's types; the app's Edit menu binds `spaceport_keymap::EDIT_KEYS` to those
same types, so a menu click and its shortcut cannot diverge. `edit_key` sits
next to them in the crate, so the keystroke a menu item replays is the one the
fields' own tables answer.

## 5. Seam 3 — the default font

Unchanged: `new(cx)` still starts from `spaceport_settings::Config::default().ui`
and `Theme::default()`, which is what `crate::config::Config::default().ui` was
(the app's `config` re-exports `spaceport_settings::Config`), so no field draws
differently. All 28 construction sites set an appearance on the next line
anyway; giving `FontConfig` a `Default` below in `spaceport-settings` is a
follow-up, not a move.

## 6. Seam 4 — the default placeholder, and the debug selector

**The placeholder default is now empty.** `"Search themes..."` was a leftover
from the theme picker. Audited all 28 production `SearchInput::new` calls:
every one either sets a placeholder (`set_placeholder`,
`set_placeholder_with`) or has one set for it (`sidebar/search.rs`, whose
heading supplies placeholder and label), except two, and neither ever wanted
that text:

| Field | Where | Now |
| --- | --- | --- |
| `NumberEdit` | `settings_window/inputs.rs:86` | no placeholder: the number itself fills the field, and clearing it used to show "Search themes..." |
| `HexEdit` | `settings_window/sidebar.rs:270` | no placeholder: a host with no colour starts empty, and used to show the same text |

No site needed the old default restored, so none sets one.

**The debug selector is the caller's.** `set_debug_selector` names the element a
test locates, and the default is now `"search-input"` instead of the theme
picker's `"theme-search"`. Of the three tests that located a field by the old
name, two name the default (`log_window/tests.rs`) and one takes the palette's
own element name, `"palette-search"`, set in `palette.rs` — the palette shares a
window with the sidebar's search field, which is exactly why the name belongs to
the caller. `debug_bounds` is last-painted-wins, so two fields under one name
were already order-dependent.

## 7. What stayed app-side, and what was kept

- `DialogInput` (`dialog_input.rs`), its draft bounds and key applier, and
  `render_dialog_input`; `MultilineInput`'s wrap, rows, `goal_x` and completion;
  `SpaceportWindow`'s own `EntityInputHandler`, `held_option_keys`,
  `discard_platform_composition` and `TerminalInputHandler`.
- The production-unreachable `masked` path in the field, with its three tests
  (`key_table_masked.txt`, the kill-buffer rule, the offset helper). It moved
  rather than being deleted: it is the field's behaviour under
  `set_masked`, and deleting it is a product decision this move did not make.
- Both pinned key tables moved with the field, and `key_matrix`'s
  regeneration note now names `cargo test -p spaceport-textfield key_table` as
  well as the app's.

## 8. Validation

Run in this worktree (`CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`,
`-p`-scoped only):

| Command | Result |
| --- | --- |
| `cargo check -p spaceport-textfield -p spaceport-gpui --all-targets` | pass, no warnings |
| `cargo test -p spaceport-textfield` | 40 passed, 0 failed |
| `cargo test -p spaceport-gpui --bin spaceport` | 3,309 passed, 0 failed, 10 ignored (the whole app suite) |
| `cargo test -p spaceport-gpui --bin spaceport key_table` | 7 passed (the dialog's and the multiline's pinned tables, reaching the crate's driver) |
| … `palette_query` | 1 passed (the palette's own element name) |
| … `sidebar` / `settings_window` / `log_window` / `palette` | 315 / 220 / 15 / 94 passed |
| … `field_placeholder` | 2 passed (a field words its placeholder again on a locale switch) |
| `SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk cargo clippy -p spaceport-textfield -p spaceport-gpui --all-targets --all-features` | pass, no warnings (SDK acquired with `scripts/release/sparkle_framework.py`) |
| `cargo fmt --all --check` | clean |
| `just check-file-size` | all files under 1,000 lines (largest new: `search_input.rs`, 678) |
| `just check-sorted` | clean |

The `text_editing` and `caret` slices the task named now run in the crate
(`cargo test -p spaceport-textfield`), so the app's filter matches only its own
remaining users of the model; `--bin spaceport search_input` matches the one
app test named for the field, the palette's.

Not run here, and why:

- **The Linux/Windows half.** `src/input/tests/lifecycle.rs` is
  `cfg(any(target_os = "linux", windows))`, as its `mod weak` was, so this macOS
  run compiles neither it nor the crate's `input/weak` tests. The `windows` CI
  job is the only place either is proved.
- **Native visual QA.** No window was driven by hand; the field's paint,
  `w_full()`/`line_height()` box, the framed and frameless styling, selection,
  caret and IME box all moved byte for byte, and the app's headless tests for the
  palette, sidebar, log window and settings search boxes pass, but a real window
  was not looked at. `just test-gui` (or a window on the desktop) is the check a
  reviewer should do for the search fields in the palette, Settings, the log
  window and the sidebar.

## The Linux window-lifecycle lesson (post-merge addendum)

The lifecycle tests surfaced a second platform divergence after the macro bug:
**macOS releases a window's root view when the last strong reference drops;
Linux keeps it alive until the window closes.** A test asserting "the handler
must not own the view" therefore measures the window's policy on Linux unless
the window is closed first — and handler reads need the live window, so the
honest order is: exercise the handler, read it while the view lives, drop the
view, close the window, park, then assert the weak handle is dead.

Main's original test dodged this by building its view outside any window
(a throwaway fixture window plus `cx.new`), which made the release assertion
trivially true. The stronger in-window shape this PR ships is the one worth
having — it just needed the close-before-assert order.

The twin's post-release handler reads (selection, ranges, all `None` after the
view dies) were removed: on Linux a `VisualTestContext` cannot drive handler
reads on a closed window, and while the window is open the handler correctly
answers through the live view, so those asserts contradicted themselves on one
platform or the other. The release guarantee is carried by the surviving
`weak.upgrade().is_none()` assert, which is the one both platforms can honor.
