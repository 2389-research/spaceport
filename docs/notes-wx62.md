# Stage 4: the section list, footer, scrollbars and input tables below the app

Stage 4 of `docs/notes-settings-extraction.md` §3, executed as crew task wx62.
Base revision `750f74b0f` (*refactor(settings): Draw each page from a prepared
PageState (#227)*), so the row kit (#224), the source snapshot (#222), the draft
and catalog policy (#219) and `PageState` (#227) are already below the app.

The design reads as a free move of three files. Against the code it is not one,
for the reason stage 2 found in the kit: the files it names are `impl
SettingsWindow` methods, and three of the reads that keep them in the app have
no home below it yet.

## 1. Survey: what was actually in the way

| Read | Stage 4's file | Home today |
| --- | --- | --- |
| `panel_resize::{Side, PanelDrag, PanelWidth, handle}` | `navigation.rs` (the drag edge and the width) | **P6, unlanded**: app-side, shared with the sidebar, the chat's terminal resize and the tab strip's dividers |
| `Entity<SearchInput>` | `navigation.rs` (the search box), `inputs.rs` (`NumberEdit`) | **P2, unlanded**: app-side, 668 lines |
| `SpaceportWindow` and the save queue (`persistence`, `save_preference`, `set_spacing`, `set_terminal_padding`, `accept_control_size`, `controls`, `sidebar::HexEdit`) | `inputs.rs` | the app, and stage 1 gave the queue no crate of its own |

Two more findings decided the test list. All three files' tests construct
`SettingsWindow::new(..)` and read private fields (`view.body_scroll`,
`view.navigation_width`, `view.section`), so none could move verbatim:

- `tests/navigation_resize.rs` is not a navigation test: it drags
  `settings-navigation-resize` and asserts `PanelWidth`'s default, its
  follow-the-pointer and its `MAX_SHARE` cap. It is a **`panel_resize` test**
  wearing this file's name, and it stays until P6 moves that module.
- `tests/sidebar_drafts.rs`'s input tests (slider preview, typed hex, Escape)
  drive `PreferenceIo` recordings and `SearchInput`; they need P2 and the save
  queue, and they stay.

## 2. What moved, and what it became

| Was (app) | Is (crate) | Lines |
| --- | --- | --- |
| `settings_window/scrollbar.rs` | `spaceport-settings-window/src/scrollbar.rs` | 140 → 143 |
| `settings_window/navigation.rs` (list + footer) | the same crate's `navigation.rs` | 245 → 314 |
| `settings_window/inputs.rs`'s `NumberPreview`, the dropdown index and the slider preview | the same crate's `inputs.rs` | new, 91 |
| `browser::Thumb` (`tab_scroll.rs`) | `spaceport-widgets/src/thumb.rs`, re-exported as `spaceport_browser::Thumb` | 46 + 44 tests |

The app keeps 449 lines of the same three modules as facades:

| App file | Lines | What it still does |
| --- | --- | --- |
| `settings_window/scrollbar.rs` | 140 → 51 | holds `body_scroll`, `navigation_scroll`, `thumb_grab`; answers the crate's two closures and reads the handle it is given |
| `settings_window/navigation.rs` | 245 → 112 | builds the resize handle and readies the search box; passes the window's select/save/reload; words the ids |
| `settings_window/inputs.rs` | 398 → 286 | `NumberEdit` (its `SearchInput`, its blur), begin/finish, the writes, and the key routing |

The window's fields did not change shape. `body_scroll`, `navigation_scroll`,
`thumb_grab` and `navigation_width` stay where they are, so the ~25 call sites
that read them (`search.rs`, `controls.rs`, the font and theme pages, the native
probes and the theme tests) are untouched.

## 3. The seams

1. **A thumb is answered by the window.** `scrollbar::scroll_area` takes the
   handle and two closures — the pointer going down on the thumb, and dragging
   it — where the app's `scroll_area` took `cx.listener` closures. `Pane`,
   the thumb's geometry, `grab` and `drag` move; the app's three-line methods
   are the handlers. `cx.stop_propagation()` moved into the crate's mouse-down
   handler, which is the one place it belonged.
2. **The section list is drawn from a prepared column.** `navigation::panel`
   builds the framed column (search box, drag edge, scrolled sections) from the
   resolved width and two prepared elements: the window's resize handle (P6's
   `panel_resize::handle`) and its search box (P2's `SearchInput`). The window
   chains its own `.on_drag_move` onto the returned div, so `PanelDrag` never
   enters the crate.
3. **Each row is marked for the native test by the app.** `navigation::sections`
   takes `probe: impl Fn(usize) -> Option<AnyElement>`; the app passes
   `native::probe(index)` under `integration-test` on macOS and none otherwise.
   The probe is absolute and size-full, so appending it after the row's
   contents keeps the bounds the native checks record.
4. **Words are the app's.** `navigation::Labels { message, unsaved }` words the
   ten ids the list and footer name (`MESSAGE_IDS`), the drafts among them, and
   the two messages `Section` and `Group` already carry as data. A footer button
   is the kit's `ui::button` on the footer's own ground
   (`.bg(rgb(theme.background))`), which is the same paint the window's
   `control_choice` produced with its two overrides: same id, debug selector,
   role, aria label, padding, radius, border, hover and disabled opacity.
5. **An index is read through a table.** `inputs::picked(field, index,
   languages)` answers `Choice::Layout` or `Choice::Preference`, including the
   "index 0 follows Herdr" rule every override list shares, so the window's
   `pick_dropdown` is a three-arm match. `NumberPreview::dragged` and
   `after_commit` carry the slider rule — which fields preview, and whether the
   released value stays until the save's reload — out of `commit_number`.

## 4. Tests

Moved into the crate (3 files, 154 app lines):

| Was | Is |
| --- | --- |
| `tests/navigation_groups.rs` | `navigation/tests/groups.rs`: every section under its group; a waiting draft offers Save now and the status names it; every id listed for the app |
| `tests/navigation_scroll.rs` | `navigation/tests/scroll.rs`: a short window scrolls to the last section and clicking it selects it; the sections' thumb follows and drags the list |
| `tests/scrollbars.rs` | `scrollbar/tests.rs`: a pane that overflows shows a thumb that drags it; a pane that fits has none and drags nowhere |
| — | `inputs/tests.rs`: the two tables and the preview's bounds |

The crate's tests drive a small `Harness` entity that answers the same closures
the window does, and assert the same debug selectors and geometry the app's did
(`settings-sections`, `settings-group-*`, `settings-section-*`,
`settings-sections-thumb`, `settings-footer-*`).

Stayed app-side, and the design's exit criterion is amended to say so:

- `tests/navigation_resize.rs` — a `panel_resize` test (P6). Its name is
  misleading; **whoever runs the P6 program should move `panel_resize` into
  `spaceport-widgets` and take this file with it.**
- `tests/sidebar_drafts.rs`'s input tests — P2 and the save queue.
- `tests/navigation_words.rs` (new, 31 lines): every id the list and footer name
  is an English message with the arguments it takes, and every section icon is
  in the app's registry — the same guard stage 2 put on the kit's ids.

## 5. `Thumb` moved to the widgets

`browser::Thumb` was already shared vocabulary — its own doc says "the review's
vertical scrollbar uses it too" — so it is now `spaceport_widgets::thumb::Thumb`
and `spaceport_browser::Thumb` is a re-export, the way the app's `motion`
re-exports `pop_in` after #224. That keeps every browser call site unchanged and
keeps `spaceport-browser` out of the settings window's dependency tree; a
settings crate depending on the browser crate for one geometry struct would have
inverted the natural order. `spaceport-browser` gained `spaceport-widgets`, and
`spaceport-settings-window` gained `spaceport-i18n` for `Language` in `picked`'s
signature. Neither adds a cycle: widgets depends on settings and sidebar, not on
browser.

This does not complete P6; it removes one of its reasons.

## 6. Where P2 and P6 stand

| | Blocks now | What is left |
| --- | --- | --- |
| **P6** (`panel_resize`) | the section list's drag edge and width, and `tests/navigation_resize.rs` | `Side`, `PanelDrag`, `PanelWidth` and `Grip` are app-side and shared by four windows; moving them is its own program |
| **P2** (`SearchInput`) | the search box, `NumberEdit`, `HexEdit`, and the key routing in `settings_key` | two prepared-element seams (the box and the hex field) already carry the drawing; the state stays app-side |

## 7. What stage 5 has to know

- The footer and the list are crate functions over the theme plus closures; the
  theme browser's pages call them through `super::navigation`, so a page that
  wants the footer's words needs `navigation::Labels`, not `fl!`.
- `scroll_area` is the crate's now: any new scrolling pane passes a `Pane` and
  two closures rather than an `Entity<SettingsWindow>`-typed listener.
- `spaceport-settings-window` still has no `Error`: nothing in the three modules
  can fail.

## 8. Validation

Run on this branch, macOS, no desktop available for native screenshots (the
stage changes no pixels: every builder draws the same elements with the same
ids, debug selectors and geometry, and the tests assert them):

- `cargo check -p spaceport-settings-window -p spaceport-gpui --all-targets
  --features integration-test` (with `SPARKLE_FRAMEWORK_PATH=target/sparkle-sdk`)
  — clean, no warnings.
- `cargo clippy -p spaceport-settings-window -p spaceport-gpui --all-targets
  --features integration-test -- -D warnings` — clean.
- `cargo test -p spaceport-settings-window` — 32 passed.
- `cargo test -p spaceport-gpui --bin spaceport settings` — 278 passed, 0
  failed (was 280; three files moved out and two tests came in).
- `cargo test -p spaceport-widgets -p spaceport-browser` — 32 and 95 passed.
- `cargo fmt --all --check`, `just check-file-size`, `just check-sorted` and
  `just check-i18n` — clean.
- Not run: `just ci`, the Windows and Linux lint legs, `just test-gui`, and any
  native window check.
