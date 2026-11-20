# Render breakdown (issue #128)

## Review-branch integration boundary

The measurements below describe the original PR 140 branch, merged as `eba200955`.
They do not measure the combined PR 125 tree.
That integration retains its prepared popup renderer and shortcut-page renderer.
Those renderers preserve inert departure controls and frozen page ownership.
Its sidebar toggle also stays local, without a daemon fallback.
See [the integration evidence](evidence/review-main-integration.md) for combined-source checks and limits.

Measuring and splitting the six 280–370-line render functions, `poll_endpoints`,
`fn command`, and `SheprdWindow::new`, without a behavior change. Each number
below is the function's own line count (first line of `fn` through its closing
brace), measured with the same script before and after.

All the counts were taken in this worktree at the commit each section names.
`just check-file-size` passes with the largest touched file at 947 lines
(`sidebar/render.rs`, untouched here).

## Status

| # | Function | File | Before | After | State |
| --- | --- | --- | --- | --- | --- |
| 1 | `render_keybinds` | `menu/settings.rs` | 391 | 31 | done |
| 2 | `render_rail` | `sidebar/rail.rs` | 415 | 11 | done |
| 3 | `render_sidebar` | `sidebar/render.rs` | 911 | 911 | **not done**, plan below |
| 4 | `render_menu_layer` | `menu/chrome.rs` | 553 | 24 | done |
| 5 | `render_tab_strip` | `window/tab_strip.rs` | 332 | 11 | done |
| 6 | `render` (window) | `window/render.rs` | 763 | 7 | done |
| 7 | `render_chat_composer` | `chat/view/composer.rs` | 281 | 35 | done |
| 8 | `poll_endpoints` | `endpoint/polling.rs` | 320 | 28 | done |
| 9 | `fn command` | `window/commands.rs` | 345 | 25 | done |
| 10 | `SheprdWindow::new` | `window/construct.rs` | 312 | 213 | done, with the sorted literal noted below |

The issue's table measured these before the browser inversion (#135). At the
base commit of this branch, `render_sidebar` was 911 lines rather than 344, and
window `render` 763 rather than 327: browser render code now lives in
`window/render.rs` and `sidebar/render.rs`. Both targets still exist at the
paths the issue names, and neither was moved into `browser/`, so neither was
skipped. Nothing under `crates/sheprd-gpui/src/browser/` was touched.

## Decomposition map

### 1. `render_keybinds` → 31 lines

`menu/settings.rs`. Splits into the page's data and its parts:

| Function | Lines | Reads |
| --- | --- | --- |
| `keybind_groups` | 78 | the catalog, the keymap, the daemon's custom commands |
| `keybind_body` | 49 | the query, one section at a time |
| `keybind_row` | 86 | one row's keys, name, and edit button |
| `keybind_header` | 55 | the title, subtitle, close button |
| `keybind_search_area` | 19 | the field and the count |
| `keybind_footer` | 8 | the page's footnote |
| `keybind_section_heading` | 10 | one section's heading |
| `keybind_section` (free) | 87 | which section a command lists under |

`keybind_section` is the 40-variant grouping match that used to sit inside the
`COMMANDS` loop. It stays exhaustive over `Command`, so a new command fails to
compile until it has a section. `KeybindRow` replaces the anonymous
`(keys, description, info)` tuple the page threaded through four closures.

### 2. `render_rail` → 11 lines

`sidebar/rail.rs` keeps the marks and measurements the rail shares with the
sidebar's wide column; drawing moved to `sidebar/rail/render.rs`:

| Function | Lines | Reads |
| --- | --- | --- |
| `rail_measure` | 43 | width, fitted face, shared columns, mutes — once per frame |
| `rail_spaces` | 35 | one host box and its workspace rows at a time |
| `rail_host_box` | 81 | one host's letter and box |
| `rail_workspace_row` | 121 | one workspace's marks |
| `rail_agents` | 32 | the agents list after every host |
| `rail_agent_row` | 113 | one agent's marks |
| `rail_frame` | 80 | the two lists, the divider, the new-workspace button |

`RailMeasure` carries the fitted font, the column grid, and the resolved mutes,
so the columns a row is measured with and the marks it draws agree without
resolving anything twice.

### 3. `render_sidebar` (911) — not done

This is the largest function in the issue and the one whose body is most
entangled: the host loop carries eight pieces of cross-iteration state (the two
scroll lists, the drop preview's rows and requests, the dragged unit, the
highlighted rows, the nested agent focus, the host row positions, and the
"filtered" flag) through a per-workspace-unit body of about 200 lines.

Planned decomposition, in the same file tree as the rest:

1. `sidebar_measure` — the prelude: width, split, look, layout, menu target,
   content x, search state, the two scroll handles, and the tag-grouping mode.
2. `sidebar_rows` — the host loop, calling `sidebar_host` per host and
   `sidebar_workspace` per unit. A `Rows` bundle replaces the eight loop
   locals, and `Host`/`WorkspaceUnit` bundles keep the per-unit parameter list
   under clippy's limit.
3. `sidebar_settle` — core rows, the agents panel, host headers, the pinned
   header, the nested reveal, and the scroll-to-row pass.
4. `sidebar_frame` — the column's own chrome, the sections, the footer, and the
   pointer canvas.

Left undone rather than half-done: the per-unit split is where the drag, drop
preview, nested agent, and hover-menu behavior meet, and a partial pass would
have moved code without reaching the 150-line bar. It wants its own change with
its own paint tests.

### 4. `render_menu_layer` → 24 lines

`menu/chrome.rs` keeps opening, dismissing, and the menu's own items; drawing
moved to `menu/chrome/render.rs`:

| Function | Lines | Reads |
| --- | --- | --- |
| `menu_layer` | 39 | viewport, dims, lift, show progress, popover width — once |
| `menu_panel` | 10 | the five phases below |
| `menu_panel_anchored` | 89 | where the panel hangs and how much room it takes |
| `menu_panel_sized` | 143 | each page kind's width and height |
| `menu_panel_surface` | 40 | border, fill, text, opacity, occlusion, hover |
| `menu_page_body` | 121 | the page's own content, still exhaustive over `Page` |
| `menu_item_list` | 40 | the application menu's own rows |
| `menu_panel_cover` | 26 | what the menu covers, where the platform needs it |
| `menu_panel_shield` | 15 | a leaving menu takes no clicks |
| `menu_placement` | 29 | the pointer anchor, the footer rise, or centred |
| `menu_overlay` | 70 | the layer that keeps the keyboard and dismisses |

`Layer` carries the measured popup state. The exhaustive `match page` that made
a new page fail to compile until it was drawn is preserved in
`menu_page_body`.

### 5. `render_tab_strip` → 11 lines

`window/tab_strip.rs` keeps the strip's shared measurements; the three phases
moved to `window/tab_strip/strip.rs`:

| Function | Lines | Reads |
| --- | --- | --- |
| `strip_measure` | 45 | the listed tabs, their places, the chosen width |
| `strip_listed` | 52 | one `Listed` per tab, with its measured width |
| `strip_scroller` | 125 | the tabs' elements, the closed neighbours, the thumb |
| `strip_tabs` | 73 | the Herdr tabs with their menus and clicks |
| `strip_bar` | 83 | the drag listeners, the leading and trailing controls |

`StripMeasure` carries the measured tabs, so the widths the strip reserves and
the tabs it draws are the same numbers. `StripMutes` resolves the mutes once
per frame for both.

### 6. window `render` → 7 lines

The root compose is now a scheduler of sub-renders. `window/render.rs`:

| Function | Lines | Reads |
| --- | --- | --- |
| `tick_frame` | 15 | what moves on its own this frame |
| `prepare_frame` | 8 | the two halves below, then the find bar, badge, and chats |
| `prepare_paint` | 148 | the frame on screen, its highlights, links, cursors |
| `prepare_groups` | 39 | the sidebar, the group slots, and where each shows its tab |
| `terminal_area` | 117 | the terminal box and every input it takes |
| `render_body` | 104 | one group's body at a time |
| `render_root` | 48 | the window's root |
| `root_events` | 65 | the actions the window answers anywhere |
| `root_body` | 66 | the bars, the body, and the overlays |

The terminal's cells moved to `window/render/cells.rs`:

| Function | Lines | Reads |
| --- | --- | --- |
| `terminal_cells` | 74 | the box, the canvas, and what sits over it |
| `terminal_canvas` | 36 | the layout the terminal reports and its input |
| `terminal_pointer_regions` | 56 | window-wide drag and hover, before any row stops it |
| `paint_cells` | 110 | the input handler and the cells, links, popup, composition |

`Paint` (what the cells draw) and `Areas` (the groups and the sidebar beside
them) are the prepared state; `Body` is what the group pass produces for the
root. Everything after `prepare_frame` reads those structs.

### 7. `render_chat_composer` → 35 lines

| Function | Lines |
| --- | --- |
| `chat_composer_parts` | 52 |
| `chat_read_only_composer` | 39 |
| `chat_attachment_chips` | 86 |
| `chat_uploading_chip` | 26 |
| `chat_send_button` | 40 |
| `chat_round_button` | 13 |

### 8. `poll_endpoints` → 28 lines

| Function | Lines | What it does |
| --- | --- | --- |
| `poll_connection_setup` | 33 | recovery, held input, the catalog's writes and reads |
| `poll_connections` | 23 | one pass over the endpoints, then the selection and toasts |
| `drain_endpoint` | 69 | one endpoint's snapshot, mutes, sound, redial |
| `sync_selected_endpoint` | 28 | the selected endpoint's state and the picture hold |
| `renew_activation` | 36 | the renewal a down or handshaking endpoint needs |
| `queue_retry_activation` | 70 | the activation a retried fence still needs |
| `settle_navigation` | 58 | the opened or expired activation and its navigation |

`Polled` carries what the tick changed: the widest redraw, the OSC 52 payloads
to write, and whether the selected endpoint itself moved.

### 9. `fn command` → 25 lines

The dispatcher is now a table. `window/commands/dispatch.rs`:

```text
ROWS: &[Row]      one row per command: `takes` (which commands) and `run`
dispatch          11 lines: the first row that takes the command, or none
```

`Run` has two outcomes: `Done` for the arms that used to `return`, and
`FallsThrough` for `ToggleSidebar` and `Reconnect`, which still ask the daemon
afterwards. Rows whose guard used to fall out of the match return
`FallsThrough` from their handler instead (`EditScrollback`, `ClearPane`,
`MoveTab*`, the name prompts), and two commands that needed a second arm
(`Tab` after `new_tabs_use_core`, `MoveTabPrevious|Next` after
`move_page_tab`) were merged into one handler each so a row still decides the
command once.

`window/commands/handlers.rs` holds 50 handlers, one per behavior: the largest
is 20 lines, and `request_daemon_command` (22) is the request a fall-through
still makes.

### 10. `SheprdWindow::new` → 213 lines

| Function | Lines | What it does |
| --- | --- | --- |
| `window_effects` | 61 | the twelve observers and the frame tick |
| `start` | 25 | what follows the field literal: preferences, reconnect, config |
| `Startup::of` | 29 | config, keymap, theme, error, and the indicators they draw |
| `Launched::of` | 19 | the catalog and endpoints the window was opened against |

The remaining 213 lines are: 46 lines of pre-literal work (launch
normalization, focus, the per-concern constructors, notices), then the sorted
field literal at **145 lines**, then 22 lines that start the window.

The literal stays one sorted initializer. AGENTS.md requires the fields stay
sorted so parallel additions land on different lines, and splitting a struct
literal is not possible; that 145 lines are field initialization only, with
every effectful read hoisted above them (`let configured_terminal_size`,
`gui_config_diagnostic`, `notices`, and the constructors). A reviewer reading
`new` now reads 46 lines of startup order and 145 lines of field names.

`Startup::of` reads `InitialAppearance` once instead of twice, deriving the
indicators from the same read the fields take.

## Per-frame notes (follow-up candidates for gfde/gg9v)

Every split function takes measured state rather than querying. The places that
still do work a frame could skip, in rough order of value:

- `prepare_paint` builds a fresh `Vec` of selection and link highlights and
  recomputes `look`, `marked`, and the regions every frame. A revision-stamped
  cache keyed on the presented frame's boot and projection revision would let a
  frame with an unchanged surface reuse them.
- `terminal_regions` is rebuilt whenever the frame is; `whole` (no regions)
  already skips the canvas paint, and the same test could skip the region
  build.
- `strip_measure` re-measures every tab's shaped width each frame
  (`tab_width` shapes text). Widths depend only on the label, the font, the
  indicators, and the selected shape, so they can be memoized per label.
- `rail_measure` resolves the mutes per listed host per frame, and
  `rail_font` steps the font down to fit. Both are pure functions of config and
  the snapshot, so a change-stamped cache would pay for itself on a narrow rail.
- `keybind_groups` walks the catalog and the daemon's custom commands per frame
  of the keybinds page, including `crate::i18n::command` for each. It only
  changes when the keymap or the daemon's commands do.
- `menu_panel_sized` re-evaluates every page kind's width rule for one page.
  Cheap, but it is the clearest place to key on `Page` alone.
- `poll_endpoints` already runs one phase per tick; the mutes resolution inside
  `drain_endpoint` is the one repeated per-endpoint read that a "nothing is
  muted" fast path could skip.

None of these were implemented here: the issue asks for the split and the
note, and a repaint filter needs its own paint tests per function.

## Verification

Per commit: `cargo check -p sheprd-gpui --all-targets`,
`cargo fmt --all -- --check`, `just check-file-size`, `just check-sorted`,
targeted `cargo test -p sheprd-gpui --bin spaceport -- <module>` runs, and the
full bin suite (3709 passed, 9 ignored) at the head of the branch.

One real regression was introduced and caught while moving the rail: a
`.row().items_center().gap(...).h(...)` chain that had no counterpart in the
original `render_host_box` moved the host box's geometry, which
`sidebar::rail::tests::host_box_dot_and_ring_stay_inside_the_rail` rejected.
It was removed before the commit.
