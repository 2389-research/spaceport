# wx56: `spaceport-menu`, the menus' stage 2

Branch `feat/wx56-menu-appear`, cut from `d423974b1` (main after #223, which
created `crates/spaceport-menu` from stage 1; the design is
`docs/notes-menus-extraction.md`). This is the design's stage 2, its seams S5
and S8: the state machine that brings a popup in and takes it out again, the
device picker's lines, and the session rows those lines carry. No behaviour
change: no `fl!` string and no line of paint differs.

## 1. What moved

| From `crates/spaceport-gpui/src` | To `crates/spaceport-menu/src` |
| --- | --- |
| `menu/appear.rs:38-148` `Shown`, `Appear` (`observe`, `panel`, `offset`, `keep`, `backdrop`, `leaving`, `settle`, `moving`), `POPOVER_SLIDE`, `DIALOG_RISE` | `appear.rs` |
| `menu/appear/kept.rs` `leaves` | `appear.rs::leaves` |
| `menu/devices/picker.rs:20-70` `Line` (`selectable`), `Folding` (`shown`), `Pick` | `picker.rs` |
| `menu/sessions.rs:20-62` `Entry` (`row`), `Row`, `Spec` | `sessions.rs` |
| `menu/sessions/management.rs:7-11` `Target` | `sessions.rs` |
| `menu/appear/tests.rs`'s eight model tests | `tests/appear.rs` |

Facades keep every old path working: `crate::menu::appear::{Appear, Shown,
leaves, offset, popover, backdrop, Kept}`, `crate::menu::devices::picker::{Line,
Folding, Pick}`, `crate::menu::sessions::{Entry, Row, Spec, Target, Edit}`, so
`MenuState`'s `appear`/`folding` fields and every importer are unchanged.

Production lines: the five app files went 2,123 → 1,939 (−184); the crate holds
260 production lines across the three new modules and 231 test lines across the
two new test files. `impl SpaceportWindow` blocks: 49 before, 49 after.
`fl!("` sites over `menu/`, `palette/` and `menus.rs`: 375 before, 375 after.

## 2. Decisions the design left open, and what they came to

- **`Kept` stays app-side.** It trades `MenuState`'s `error`, `input` and
  `unfolded_device` in and out for the length of a leaving frame, and `input` is
  the `DialogInput` entity; `MenuState` is off-limits, so the crate cannot hold
  it. Instead the crate's state machine is generic over it — `Appear<K>` — and
  keeps the policy that belongs with the leave ("only the page that just closed
  keeps what it read", "lets go of it once it has gone") in one place. The app's
  `Appear` is a one-field wrapper around `Appear<Kept>`.
- **The settled-in-tests seam stays app-side.** `Appear.live` lets unit tests
  lay menus out where they settle unless one asks to see them move; a
  `#[cfg(test)]` field in the crate would be absent when the app's own tests
  build it, and a plain field would need a default that differs between
  production and tests. So the crate's `observe` takes `still: bool` and the
  app's wrapper supplies it from `live` under the same `#[cfg]` pair as before.
- **`Pixels` became `f32` at the boundary**, as the design's S5 allows: the
  anchor is only ever compared (`Shown::Popover { x, y }`), `offset` returns a
  distance, and `rise_height` is a `Rc<Cell<Option<f32>>>`. The app converts in
  three places — `popover(anchor)`, `offset(dims, shown) -> Pixels`, and
  `rise_height.get().map_or(room, px)` — and paints exactly what it did.
- **`spaceport-motion` is used as it is**: `Toggle`, `ENTER`, `LEAVE` and
  `progress`, with no second copy of the curve. `Folding::shown` keeps timing
  through `duration(LEAVE)`, so QA > Slow Animations still slows a fold.
- **`Spec`'s words are `String`.** `label` and `detail` were `SharedString`;
  the app builds both from `fl!`/state words and paints them, and `String`
  renders as text, so no paint line changed. Five `.into()` calls became
  no-ops and were removed.
- **Three fields of the crate's `Appear` are public** — `since`, `leaving`,
  `kept`: the app's window tests hold a motion at its start rather than waiting
  it out, and the leaving frame takes the kept value out and puts it back.
- **`Row` and `Pick` gained `Debug`**, so a test can assert the identity of a
  picker line through them.

## 3. What the task list did not name, and why it moved anyway

- **`menu/sessions.rs`'s `Entry`/`Row`/`Spec` and `Target`.** The task and the
  design's stage-2 line both name `picker.rs:18-76` as "the pure half", but
  those lines are not self-contained: `Line::Session(Entry)` carries
  `Entry::Row { row, spec }`, `Row::Add` carries `Target`, and `Spec` carried
  two `SharedString`s. Moving `Line`/`Pick` without them was impossible, and
  the design's stage 2 already names "`menu/sessions.rs`'s `Entry`/`Row`". The
  owner confirmed moving the whole cluster (`spaceport_menu::sessions`), which
  keeps the lines whole and the crate's boundary honest.
- **`menu/devices/core_catalog.rs:33-52` did not move**, though the design's
  stage 2 names the text helpers there. Both carry words of their own:
  `short`'s `fmt::write` failure and `unavailable_reason`'s `None` case both
  return `fl!("core-terminals-machine-not-current")`. A crate below the app
  says no sentence (`docs/i18n.md`), so they stay until a stage gives them a
  value to return instead of a message.

## 4. Tests

Moved to the crate as pure tests: the popover's slide, a popover opened
elsewhere appearing again, the dialog's rise and its dimming, a dialog closed
midway, All Devices sinking out of the footer, a session form closing at once
rather than the picker sinking, a closed page letting go of what it read, other
menus not lingering, and `still` taking a change where it settles (9).

New pure tests for the picker (`tests/picker_lines.rs`, 4): only a row is
something to choose (notes and errors are skipped by the keys), a line is
recognised by what it chooses rather than where it sits (`Pick` identity), and
a fold closing from where it was and then going, including one that began shut.

Stayed app-side, because they drive a window: `unit_tests_see_menus_settled`
(now the wrapper's seam), `the_dimming_keeps_its_color_and_scales_its_alpha`
(the `Rgba` the app builds), and the three window tests
`all_devices_draws_lower_while_it_rises`,
`the_devices_popover_is_drawn_as_it_goes_and_leaves_the_menu_reset` and
`the_dimming_lifts_behind_the_picker_a_dialog_returns_to`.
`menu/devices/tests/fold.rs` still drives fold and unfold through the window
and passes unchanged.

## 5. Verification

From the worktree root, with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`:

```sh
cargo check -p spaceport-menu --all-targets
cargo clippy -p spaceport-menu --all-targets -- -D warnings
cargo test -p spaceport-menu                      # 30 passed (17 + 13 new)
cargo clippy -p spaceport-gpui --all-targets --features integration-test -- -D warnings
cargo test -p spaceport-gpui --bin spaceport menu  # 290 passed
cargo fmt --all --check
just check-file-size
just check-sorted
just check-i18n                                   # 2114 messages in 114 files
```

The `integration-test` legs need Sparkle, which was acquired locally with
`python3 scripts/release/sparkle_framework.py acquire target/sparkle-sdk` and
passed as `SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk`. `cargo metadata`
(refreshed by the builds) added `spaceport-motion` to `spaceport-menu` in
`Cargo.lock`.

Not run, and deferred to CI: the whole app test binary, the native and Linux
checks (`just test-gui`, `menus/in_window/native.rs`), the Windows-only paths,
and the release-mode performance budget.
