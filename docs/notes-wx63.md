# Menus stage 4: the chrome becomes `spaceport-menu`'s GPUI half

Executes stage 4 of [`notes-menus-extraction.md`](notes-menus-extraction.md)
(S9, "the popup chrome, ~1,530 lines, GPUI-bound, movable only as a kit") on top
of #223 and #225, which put the page set, the targets, the typed requests, the
appear state and the picker rows below the app, and on #220, which put the
logical sides and the AccessKit helpers in `spaceport-widgets`.

`crates/spaceport-menu` gains `gpui`, `spaceport-settings` and
`spaceport-widgets` and becomes the GPUI crate it is already named for, as
`spaceport-browser` and `spaceport-sidebar` did. The three files the stage names
move whole and keep their bodies: `menu/chrome.rs`'s geometry, `menu/chrome/
render.rs`'s panel, surface, placement and overlay, and `menus/in_window/
render.rs`'s bar and dropdown — with three of that file's five pieces of window
state (`Geometry`, `OpenMenu`, and their methods) moved with it, because a paint
that reads a menu tree cannot live above the type that holds it.

## 1. The survey the design asked for

**What `menu_panel`/`menu_page_body` read of `self`.** The design's list was
right and slightly short. Panel assembly reads `self.config.ui` (the font and
its size), `self.theme` (`active`, `surface`, `foreground`, `muted`), and these
`MenuState` reads: `anchor`, `selected`, `session_edit`, `cover`,
`appear.state` (`panel`, `rise_height`, `backdrop`), plus twelve derived facts
the old `Layer` carried. Three of those facts are not computable from a page and
a viewport, so the app prepares them: `host_forwards` (the host menu's forward
list), `palette_previews` (Go To's preview, which depends on the palette's own
filter) and `popover` (the worktree popover's measured caption width, shaped by
the window's text system). `menu_page_body` itself reads nothing the chrome
owns: it is the page dispatch chain, so it stays, and the crate hands the app a
panel to append it to.

**Whether `menus/in_window/render.rs`'s `crate::titlebar` reads can be prepared
into `Layer` fields.** They can, and they are not layer facts — they are window
facts, so they became [`chrome::Metrics`](../crates/spaceport-menu/src/chrome.rs)
instead: `chrome` (`titlebar::HEIGHT`), `usage_width` (`usage::PANEL_WIDTH`) and
the viewport. The two `show_controls` calls are one prepared `controls_shown`
bool, `titlebar::controls`' element is passed by value (an `AnyElement` is not
`Clone`), and `titlebar::movable_with_controls` stays where it is, because it is
the window's own press handling and applies to the bar after the crate returns
it.

## 2. The seam

`spaceport_menu::chrome` and `spaceport_menu::bar` take a borrow struct each —
the `BrowserHosts` shape — and hand back elements:

- `chrome::Hosts<'a, K>`: the appear state (`Appear<K>`), the anchor, the
  backdrop, the three prepared facts above, the metrics, the font, the theme,
  and `now`.
- `chrome::layer(&Hosts, Page) -> Layer`: the measured facts (unchanged, the
  twelve plus the viewport), now computed in one place from data instead of
  from a window.
- `chrome::panel`, `placement`, `leaving`, `overlay`, `anchor_footer_panel`,
  `rows`, and `dims`, which replaces `SpaceportWindow::menu_dims`'s formula.
- `bar::Hosts<'a>`: the headings' words, `controls_shown`, the bar's fill, the
  measured `Geometry`, the open `OpenMenu`, the font, the theme and the metrics.
- `bar::bar`, `bar::dropdown`, `bar::place`.

Behaviour is passed in, not written here: `interact` closures take each row or
heading's element back and attach the window's own hover, press and AccessKit
actions. That keeps every `cx.listener`, every `fl!` and every command
dispatch on the app side, which is the rule stage 1 set for labels.

Three things were deliberately left where they were:

- **`Cover` and its canvas.** It is read by the app's tests, the layout tests
  and the Linux native check, and the crate would have had to gate it on a
  cargo feature to reach the app's `cfg(test)` builds — the re-export gate this
  repository has been bitten by. `menu_panel_cover` and `menu_panel_shield`
  therefore stay in the app, appending to the panel the crate returns.
- **`Page::Menu`'s `Item`s.** Their labels are `fl!` and their `ALL` list is a
  test's, so the app words them and passes `chrome::Row { label, selector }`.
- **`application_shortcut`.** It reads the keymap.

## 3. Where it landed

| Crate | File | Lines |
| --- | --- | --- |
| `spaceport-menu` | `src/chrome.rs` (hosts, layer, text, metrics, constants) | 188 |
| `spaceport-menu` | `src/chrome/{panel,overlay,items}.rs` | 360 / 40 / 47 |
| `spaceport-menu` | `src/bar.rs` (`Geometry`, `OpenMenu`, bar, dropdown, placement) | 478 |
| `spaceport-menu` | `src/tests/chrome.rs` (5 geometry tests) | 135 |
| `spaceport-gpui` | `src/menu/chrome/render.rs` (hosts, page dispatch, cover, shield, handlers) | 318 |
| `spaceport-gpui` | `src/menus/in_window/render.rs` (hosts, rows, handlers) | 292 |

Moved out of the app in the same change, because they were only ever the
chrome's: `rail::{COMPACT_BELOW, WIDTH, HEIGHT}` (now `ADD_DEVICE_*`),
`devices::{MENU_GAP, MENU_WIDTH}` (now `DEVICES_MENU_*`), `menu::MENU_MARGIN`,
and `appear::offset`'s app-side wrapper. `MenuState` keeps its `application`,
`application_bar` and `appear` fields, whose types are now the crate's, and
gains `session_modal(page)`, which is the one derived fact three callers needed.

## 4. What stays

`MenuState`, every `impl SpaceportWindow`, every `fl!`, the `Page` dispatch
chain, the `Item` list, the cover and shield, `application_shortcut`, and the
`chrome/keys.rs` input handling.

## 5. Validation

Run on macOS arm64 with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0` and
`SPARKLE_FRAMEWORK_PATH` from `scripts/release/sparkle_framework.py acquire`:

- `cargo check` and `cargo clippy --all-targets` for `spaceport-menu` and
  `spaceport-gpui`, with and without `--features integration-test`: clean.
- `cargo fmt --all -- --check`: clean.
- `just check-file-size`, `just check-sorted`, `just check-i18n`: OK.
- `cargo test -p spaceport-menu`: 35 passed, including five new geometry tests.
- `cargo test -p spaceport-gpui --bin spaceport menu`: 290 passed, which
  includes `menus::in_window::tests::{navigation, config_change}` and
  `menu::appear::tests`.

**Deferred, and honest about it.** The bar's own layout tests —
`menus/in_window/tests/{bar, dragging, resizing, shortcut_labels}.rs` — are
`cfg(any(target_os = "linux", windows))` and cannot run on this machine, and so
can the Linux `in_window/native.rs` check. What was done instead: those files
and `native.rs` were compiled here by lifting their `cfg` for one check, which
found the two import fixes they needed (`Rc`/`RefCell`/`Cell` had come through
`in_window.rs`'s imports) and nothing else; the `cfg`s were then restored. The
`verify_shortcut_editor` call at the end of `native.rs` could not be reached
that way — it is Linux-gated itself — so its one line is unverified here. The
native menu smoke on macOS is not possible for a bar macOS does not draw.
