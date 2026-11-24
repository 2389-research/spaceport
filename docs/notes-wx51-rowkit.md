# wx51 — settings stage 2: the row kit gets its GPUI crate

Branch `feat/wx51-settings-rowkit`, cut from `origin/main` (`c6eec28dd`, with
#219's drafts/catalog, #220's widget crate and #222's source snapshot landed).
This is stage 2 of `notes-settings-extraction.md` §3: the Settings window's row
kit is now `crates/spaceport-settings-window`, and the pages keep drawing
through the app's own `settings_window::ui` paths.

## What the stage had to settle first

Stage 2 reads as a pure move of six files. Against the code it was not one:

| App-only read | Sites | In |
| --- | --- | --- |
| `fl!` (`settings-source-herdr`, `-herdr-ui`, `settings-unit-percent/pixels/seconds`, `settings-number-range`, `settings-use-herdr`, `settings-swatch-none`) | 6 call sites, 8 ids | `ui.rs`, `ui/number.rs`, `ui/popover.rs`, `ui/swatches.rs` |
| `Entity<SearchInput>` | 2 | `ui/number.rs` (`NumberEdit`), `ui/swatches.rs` (`hex_input`) |
| `font_for` + `fonts::StyledFont` | 1 | `ui/swatches.rs`'s mono hex field |
| `crate::settings_window::controls::native_probe` | 1 | `ui/number.rs`, `integration-test` only |

`fl!` is P1 (§7's decision, not landed) and `SearchInput` is P2 (not landed), so
neither could be carried across, and the design's own §2.1 says so — its stage 2
precondition named only P3. The owner chose words-as-data (the pattern stage 1
set with `catalog::search` and `chat_core`'s `Labels`), which is what this
branch does.

## What moved, and what it became

| Was | Is | Production lines |
| --- | --- | --- |
| `settings_window/ui.rs` (`Source`, `tag`, `section`, `Row`, `row`, `data_row`, `toggle`, `button`, `segmented`, `segmented_elements`, `ROW_HEIGHT`) | `spaceport-settings-window/src/ui.rs` | 383 → 426 |
| `settings_window/ui/number.rs` | the same crate's `ui/number.rs` | 303 → 288 |
| `settings_window/ui/popover.rs` | `ui/popover.rs` | 215 → 213 |
| `settings_window/ui/swatches.rs` | `ui/swatches.rs` | 192 → 195 |
| `search.rs`'s `RowBounds`, `track_bounds`, `SCROLL_MARGIN` | `src/bounds.rs` | 29 → 26 |
| `controls/forms.rs`'s `control_card`, `control_switch` | `src/forms.rs`, as functions over `&Theme` | 66 → 66 |
| — | `src/native.rs` (the probe the kit's tests click by) | 40 |
| — | `src/lib.rs` | 20 |

The files that moved were 1,188 production lines in the app (`ui.rs` 383,
`number.rs` 303, `popover.rs` 215, `swatches.rs` 192, `forms.rs` 66, and the
29 `search.rs` gave up); the crate is 1,274, the difference being the seams
below. The app keeps a 133-line `settings_window/ui.rs` facade, a 25-line
`controls/forms.rs`, and nothing else of the kit.

## The five seams

1. **Words are the app's (`ui::Labels`).** The crate holds no sentence:

   ```rust
   pub struct Labels {
       pub message: fn(&'static str) -> String,
       pub number_range: fn(f32, f32) -> String,
       pub follow_herdr: fn() -> String,
   }
   ```

   It names the ids it draws in `ui::MESSAGE_IDS`, the way stage 1's `Entry`
   keeps its title and description as ids. The app builds `Labels` once per
   call in its facade (`i18n::message`, `fl!("settings-number-range", …)`, and
   `fl!("settings-use-herdr", value = VALUE_MARK)`), and an app test checks
   every id in `MESSAGE_IDS` is an English message — the same guard stage 1
   put on the catalog.

2. **A section's tag is its word plus its source.** `section(theme, title,
   Some((Source::Herdr, "Herdr")))`. The app facade keeps the pages' three-arg
   form and words the tag, so all 27 `ui::section(…)` sites are untouched.

3. **A catalogued row words its entry through the app.** `row(theme, entry,
   &labels)` replaces `catalog::title`/`description` inside `ui::row`; the app
   facade still takes `&Entry` alone, so all 60 `ui::row(…)` and both
   `ui::data_row(…)` sites are untouched.

4. **A typed number's field is the view's element.** `ui::number` takes
   `Option<Editing { input: AnyElement, invalid: bool }>` where it used to take
   `&NumberEdit` holding `Entity<SearchInput>`; the app's `NumberEdit` (its own
   `SearchInput`, its blur subscription) is now declared in `inputs.rs`, where
   the edit begins and ends. `swatch_row`'s hex field takes the same
   `Option<(AnyElement, bool)>` shape. P2 stays unbuilt and unneeded here.

5. **A form piece is a function over the theme.** `control_card` and
   `control_switch` were `impl SettingsWindow` methods reading `self.theme`; the
   crate's take `&Theme`, and the window keeps two 4-line methods that call
   them, so the 2 + 3 call sites did not change.

The kit's macOS native probe moved with it (`src/native.rs`, behind a new
`integration-test` feature the app's own feature enables), because the number
field's probe and the font controls' probes are the same seam; the app's
`controls/fonts.rs` re-exports `native_probe` and reads `bounds`/`clear` from
the crate.

## Two additions to `spaceport-widgets`

Both were forced by code that is the app's *above* the crate, and both are
chrome-wide rather than settings-shaped:

| Moved | Why |
| --- | --- |
| `font_for` (`config.rs:60`) and `fonts::StyledFont` (`fonts.rs:12`) | The swatches' mono hex field needs the configured face. 54 app sites use `text_font(`, and `crate::fonts::StyledFont`/`crate::config::font_for` are now re-exports, so none of them changed. |
| `motion::pop_in` | The dropdown animates its list; the app's `motion.rs` re-exports it for the four other callers (log window, menus, overview, chat jump). Its "unit tests see it in place" switch is now `cfg!(feature = "test-support")`, which is how a dependency can tell it is in a consumer's test build. |

`spaceport-widgets` gained `spaceport-motion` and nothing else.

## Tests

21 tests in the crate: 19 moved from the app's four kit files, and two new.

- `ui/tests.rs`: the rows' parts and segments (moved), plus
  `a_tagged_row_draws_the_word_it_was_given`, and the switch's contrast rule
  (moved, now against `spaceport_settings::contrast`).
- `ui/number/tests.rs` (moved whole), `ui/popover/tests.rs` (moved; the
  `follow_herdr_value` case now builds its label from the app-shaped template
  instead of `fl!`), `ui/swatches/tests.rs` (moved).
- `tests.rs`: the shared `labels()` fixture, and
  `every_id_the_kit_names_is_listed_for_the_app`, so an id the kit draws cannot
  go unlisted.

New in the app: `every_id_the_row_kit_names_is_an_english_message`
(`settings_window/tests/search.rs`), beside stage 1's catalog guard.

The crate's shared fixture words its ids as the English catalog does ("px",
"none", …) rather than with their ids, because the identity spelling makes the
unit label long enough to move the value field under the test's click.

## P1/P2 status after stage 2

**P1 (message lookup below the app) is still unlanded, and the kit no longer
needs it.** The eight ids stage 2 moved are now worded by the app at one seam:

| Id | Worded by |
| --- | --- |
| `settings-source-herdr`, `settings-source-herdr-ui` | `Labels::message`, through `Source::label_id()` |
| `settings-unit-percent`, `settings-unit-pixels`, `settings-unit-seconds` | `Labels::message`, through `Unit::label_id()` |
| `settings-swatch-none` | `Labels::message`, through `swatches::SWATCH_NONE` |
| `settings-number-range` | `Labels::number_range` |
| `settings-use-herdr` | `Labels::follow_herdr`, around `ui::VALUE_MARK` |

`settings_window/`'s production files still spell 144 `fl!` sites (150 before
this branch): six ids left the kit's files — the two tags, the three units and
the "none" swatch — and two moved to the seam that words the kit
(`settings-number-range`, `settings-use-herdr`). The other 142 are the pages',
the controls' and the theme browser's own sentences, which is what stages 5 and
6 wait on.

**P2 (a text field crate) is still unlanded, and the kit no longer needs it.**
Two `Entity<SearchInput>` sites remain in the app, both of them state, not
drawing: `inputs::NumberEdit.input` and the `HexEdit` behind
`pages/sidebar.rs`'s hex field, whose element the facade hands the kit as an
`AnyElement`. Nothing else in the kit touches a text field.

## Validation

Run on this branch, macOS, no desktop available for native screenshots (the
stage changes no pixels: every builder draws the same elements with the same
ids, debug selectors and geometry, and its tests assert those):

- `cargo check -p spaceport-settings-window --all-targets`,
  `cargo check -p spaceport-gpui --all-targets` and
  `cargo check -p spaceport-gpui --all-targets --features integration-test`
  (with `SPARKLE_FRAMEWORK_PATH=target/sparkle-sdk`) — clean, no warnings.
- `cargo clippy -p spaceport-settings-window -p spaceport-widgets -p
  spaceport-settings --all-targets -- -D warnings` and `cargo clippy -p
  spaceport-gpui --all-targets --features integration-test -- -D warnings` —
  clean.
- `cargo test -p spaceport-settings-window` — 21 passed.
- `cargo test -p spaceport-gpui --features integration-test --bins` — 3,513
  passed, 10 ignored, 0 failed.
- `cargo fmt --all --check`, `just check-file-size`, `just check-sorted` and
  `just check-i18n` — clean.
- Not run: `just ci`, the Windows and Linux lint legs, `just test-gui`, and any
  native window check.

## What stage 3 has to know

- The pages now reach the kit only through `settings_window::ui`, which is the
  seam stage 3's `PageState` will be threaded through; the facade's five
  wrappers are where a page's words are built, so a prepared page can carry a
  `Labels` rather than build one per row.
- `ui::Labels` is `Copy` and cheap; nothing caches it today.
- `spaceport-settings-window` has no `Error`: nothing in the kit can fail
  (`parse_number` and `parse_hex_input` answer `Option`), which is why it has no
  `error.rs` — the same shape as `spaceport-reorder` and `spaceport-truncate`.
