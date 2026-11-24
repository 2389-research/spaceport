# Extracting the menus from `crates/spaceport-gpui`

Design for the extraction named in the split plan's feature table as the
sidebar's biggest remaining blocker (`docs/gpui-split-plan.md:283`) and one of
the app's three large features. Its subject is not a window like settings and
not a plain struct like browser: it is **46 `impl SpaceportWindow` blocks** over
`src/menu.rs`, `src/menu/`, `src/menus.rs`, `src/menus/`, `src/palette.rs`,
`src/palette/`, `src/group_menu.rs`, `src/tab_menu.rs`, `src/pane_menu.rs`,
`src/dialog_input.rs` and `src/close_modal.rs`. It is a design only: no
production code changed, no build run.

**Base revision, and one caveat.** This worktree has no shell, so nothing here
could be rebased or fetched: `git log --oneline -1` was not run, and the counts
below are read from the working tree as it stands. `crates/spaceport-widgets`
does not exist in this tree and neither `src/direction.rs` nor `src/toggles.rs`
nor `src/accessibility.rs` has moved, so **this checkout predates #220**; §2.6
answers the widget question against the app-side files #220 moves rather than
against the new crate. Everything else — line counts, `impl` counts, `fl!`
sites — is measured, not estimated, and §0 gives the commands to re-run on
whatever commit stage 1 starts from.

**Verdict.** Menus are worse than settings, and the difference is not size, it
is shape:

| | browser | settings | menus |
| --- | --- | --- | --- |
| production lines | 4,390 | 11,520 | **20,146** |
| of which entity methods | 4,390 `impl Browser`, invertible | ~9,180 of 11,520 | **~10,100 of 20,146** |
| what blocks it | nothing | text (~150 `fl!`) | text (**~377 `fl!`**), two text-field entities, `endpoint`, `keymap` |
| realistic share that moves | 69% | 54% | **17–22%** |

Only ~1,000 production lines of `src/menu/` are outside `impl SpaceportWindow`,
and the reason is structural: a menu page is *the window's* state. `MenuState`
(`menu/state.rs:37-108`) is one 44-field struct that holds every page's model —
the palette, the tab, group, pane and host menus, the device picker's unfolding,
the theme picker, the close confirmation, the keybind search field, and the
`(selection_epoch, connection_generation)` fence — plus GPUI handles
(`FocusHandle`, three `ScrollHandle`s, an `Entity<SearchInput>`, a
`Subscription`). There is nothing to invert in the browser's sense: there is one
window entity and a page table that names it 38 times.

So the honest program is four parts:

1. **~1,100 production lines are gpui-free policy that is already written as
   such**: the closed page set (`menu/page.rs`), the workspace target and its
   typed request (`menu/workspace.rs:1-157`), the pane/tab/group targets and
   actions (`pane_menu.rs:19-185`, `tab_menu.rs:22-135`, `group_menu.rs:27-97`),
   the selection and appear arithmetic. They move to a new `spaceport-menu`
   crate with no new dependency and no behaviour change.
2. **~2,000 production lines of the palette are pure ranking policy** — nucleo
   scoring, frecency, project planning, Go To row building, preview facts. They
   move to `spaceport-palette`, a sibling crate, and are the single largest
   pure win in the module.
3. **~1,500 lines of popup chrome are a self-contained GPUI paint kit** — panel
   placement, appear/leave motion, the cover and shield, the in-window menu bar.
   They need a GPUI crate (the `spaceport-menu` crate becomes one, as
   `spaceport-browser` and `spaceport-sidebar` did) plus a hosts struct, and
   they cannot move before stage 4.
4. **~16,000 lines — the page bodies, every dialog, every picker — do not move.**
   They are made of `fl!` text (377 sites), `crate::search_input::SearchInput`,
   `crate::dialog_input::DialogInput`, `self.endpoints[self.selected_endpoint]`
   and the terminal painter. That is the same verdict settings reached about its
   pages, at three times the text and with two more blockers on top.

Each stage below is independently shippable and states what to survey before it
starts and what proves it done.

## 0. The measurements, and how to repeat them

Every line count is the file's own length in this tree. Re-run before each
stage, and record the base revision the first line prints:

```sh
git log --oneline -1                                                     # record the base revision
find crates/spaceport-gpui/src/menu crates/spaceport-gpui/src/palette \
     crates/spaceport-gpui/src/menus -name '*.rs' ! -path '*/tests/*' \
     ! -name '*tests.rs' | xargs wc -l | tail -1                         # 11,241 + 3,854 + 1,271
rg -c '^impl SpaceportWindow' crates/spaceport-gpui/src/menu* \
     crates/spaceport-gpui/src/palette* crates/spaceport-gpui/src/{tab,group,pane}_menu* \
     crates/spaceport-gpui/src/menus                               # 32 + 6 + 3 + 1 + 1 + 3
rg -o 'fl!\("' crates/spaceport-gpui/src/menu crates/spaceport-gpui/src/palette \
     crates/spaceport-gpui/src/menus.rs | wc -l                    # ~315
rg -l 'crate::search_input|crate::dialog_input' crates/spaceport-gpui/src/menu*
rg -o 'crate::(direction|toggles|accessibility)' crates/spaceport-gpui/src/menu* \
     crates/spaceport-gpui/src/palette* | wc -l                    # 13
```

| | Lines | Files |
| --- | --- | --- |
| `menu/` production | **11,241** | 39 |
| `menus.rs` | 492 | 1 |
| `menus/in_window.rs` + `render.rs` | 779 | 2 |
| `menus/in_window/native.rs` (`integration-test`, Linux) | 154 | 1, not counted |
| `palette/` production (`palette.rs` + 15) | **3,854** | 16 |
| `group_menu.rs` | 394 | 1 |
| `tab_menu.rs` | 768 | 1 |
| `pane_menu.rs` + `moves.rs` + `processes.rs` | **1,664** | 3 |
| `dialog_input.rs` | 305 | 1 |
| `close_modal.rs` | 483 | 1 |
| `notice.rs` (the notices menus raise) | 166 | 1 |
| **total production** | **20,146** | 65 |
| test files under these roots (~60, sampled) | ≈20,000–25,000, not counted | 60 |

The split plan's 20.6k and this 20,146 differ by `theme_picker.rs` (623, opened
from `Page::Themes` but designed with settings) and rounding; use this table's
roots so the number is reproducible.

Two measurements carry the verdict:

- **`impl SpaceportWindow` density.** 32 blocks in `menu/`, 6 in `palette/`, 3
  in `pane_menu/`, 1 each in `tab_menu.rs`, `group_menu.rs`, `menus/in_window.rs`
  and `menus/in_window/render.rs` — 46 in all. Measuring each block as "first
  `impl SpaceportWindow` line to end of file" gives **8,667 of `menu/`'s 11,241
  lines (77%)**, 1,391 of `palette/`'s 3,854 (36%) and ~1,187 of `pane_menu/`'s
  1,664 (71%). The nine `menu/` files with no `impl SpaceportWindow` at all
  total 1,037 lines: `page.rs` 151, `state.rs` 221, `setup.rs` 271, `colors.rs`
  120, `page_title.rs` 77, `menu.rs` 91, `host_menu/actions.rs` 41,
  `appear/kept.rs` 40, `devices/error.rs` 25.
- **`fl!` sites: ~377 across the program.** `menu/` ~230, `menus.rs` 55 (the
  application menu bar alone), `palette/` 30, `pane_menu.rs` 20, `tab_menu.rs`
  15, `close_modal.rs` 12, `theme_picker.rs` 8, `group_menu.rs` 7. Settings
  reached ~150 in 11,520 lines; this is 2.5× the text in 1.75× the lines,
  because a menu *is* a list of sentences.

## 1. Code context: what the menus are

### 1.1 The two halves, because they move at different speeds

`src/menu.rs:3-30` declares nineteen modules and `src/menu.rs:44-50` re-exports
what the rest of the app names (`Page`, `WorkspaceAction`, `MenuState`, `Cover`,
`accent`/`danger`/`online`/`tint`, `VersionNotice`, `SettingsHint`,
`enter_key`). Every file below is one of two kinds, and the kind decides the
stage it can reach:

**A. The app's own surfaces** — no Herdr concept is in the row's identity, so
they only need a place to live and words to say:

| File | Lines | Shape |
| --- | --- | --- |
| `menu/chrome/render.rs` | 685 | `impl SpaceportWindow`: panel, cover, shield, page body |
| `menu/chrome.rs` | 363 | `impl`: opens/paints pages with shared geometry; `ChromeAction` + labels (345-359) |
| `menu/chrome/keys.rs` | 257 | `impl`: 200-line `menu_key_down` page dispatch |
| `menu/settings.rs` | 796 | `impl`: Keybinds page, keymap reference, shortcut editor host, config watch |
| `menu/shortcut_editor.rs` | 297 | `impl`: the inline binding editor |
| `menu/colors.rs` | 120 | **pure**: `tint`/`accent`/`danger`/`online` over `Config::Theme` |
| `menu/appear.rs` + `appear/kept.rs` | 273 | **model**: the enter/leave state machine, and what a closing page draws from |
| `menu/page.rs` | 151 | **pure**: `Page` (38 variants), `WorkspaceAction`, `WorkspaceMenuAction`, `pointer_anchored`, `pane_menu`, `in_header`, `icon` |
| `menu/page_title.rs` | 77 | **pure+text**: page names for accessibility (`Role`, `SharedString`) |
| `menu/state.rs` | 221 | model + GPUI handles (above) |
| `menu/devices/picker.rs` | 860 | `impl`: the device picker's lines, folding animation, keys |
| `menu/devices.rs` | 367 | `impl`: device scope, entry points, footer offers |
| `menu/devices/rail.rs` | 286 | `impl`: Add Device's frame and source rail |
| `menu/devices/add_device.rs` + `add_device/core.rs` | 621 | `impl`: the form and its confirmed background work |
| `menu/devices/core_catalog.rs` | 243 | `impl` + 60 **pure** (`short`, `unavailable_reason`) |
| `menu/devices/wsl.rs` | 531 | `impl`: WSL distributions as devices |
| `menu/devices/setup.rs` | 271 | **pure**: the device catalog through the installed `herdr` CLI |
| `menu/devices/activity_card.rs` | 105 | `impl`: the picker's activity card |
| `menu/devices/error.rs` | 25 | **pure**: `thiserror` enum for removing/renaming a device |
| `menu/sessions.rs` + `sessions/management.rs` | 994 | `impl`: local sessions, unfold, add/delete forms |
| `menu/whats_new.rs` | 149 | `impl`: the daemon's update notice |
| `menu/version_mismatch.rs` | 206 | `impl`: refused-handshake notice |
| `menu/open_link.rs` | 104 | `impl`: ask before an unlisted link scheme |
| `menu/worktree_note.rs` | 94 | `impl`: the client-local checkout note |
| `menu/mute.rs` | 73 | `impl`: mute a device, workspace or tab (client-local) |
| `menus.rs` | 492 | **data**: the application menu tree (`Vec<gpui::Menu>`), 55 `fl!`, dispatched as GPUI actions |
| `menus/in_window.rs` + `render.rs` | 779 | `impl`: Linux/Windows bar and dropdown |
| `palette/` | 3,854 | see §1.2 |
| `close_modal.rs` | 483 | `impl`: the close confirmation |
| `dialog_input.rs` | 305 | **an entity**: the one-line text draft every dialog uses |

**B. Herdr-integration surfaces** — the row's identity, parameters and wording
are Herdr's own, and the action becomes a `spaceport_client::Method`:

| File | Lines | Herdr concept |
| --- | --- | --- |
| `menu/workspace.rs` | 428 | `WorkspaceTarget` + `WorkspaceRename`/`TabCreate`/`WorkspaceCreate` requests; `left_checkout` carries Herdr's worktree key |
| `menu/workspace/popover.rs` | 459 | the workspace tiles, `Collapse`/`Expand`/`Mute` |
| `menu/workspace/render.rs` | 175 | the four dialogs' bodies |
| `menu/workspace/requests.rs` | 87 | carrying a dialog's request to the daemon |
| `tab_menu.rs` | 768 | `TabClose`/`TabRename`/`TabMove`/`TabCreate` |
| `pane_menu.rs` | 759 | `PaneSplit`/`PaneSwap`/`PaneZoom`/`PaneInputSet`/`PaneRename`/`PaneClose` |
| `pane_menu/moves.rs` | 300 | moving a pane; `pane_move::host_for` |
| `pane_menu/processes.rs` | 605 | the pane's processes and ending them |
| `group_menu.rs` | 394 | a group's tabs and splits |
| `menu/devices/host_menu.rs` + 5 | 1,832 | a saved device: rename/remove through the `herdr` CLI, forwards, tags, theme, mute |
| `menu/settings.rs`'s reload half | ~180 of 796 | `ServerReloadConfig`, Herdr's `config.toml` |
| `palette/go_to.rs` | 269 | every workspace, tab, pane and agent on each host |

The distinction matters because of what each half *reads*. Class B already
speaks `spaceport_client::protocol::ClientShellSnapshot` and
`spaceport_client::Method` — both below the app today — and its remaining read
is an endpoint id. Class A reads app entities (`SearchInput`, `DialogInput`,
`TerminalPainter`) and the app's own word list, which is the expensive part.

### 1.2 The palette, which is a different program inside the same feature

`palette.rs:311-350` holds `Palette`: an `Entity<SearchInput>`, `nucleo`'s
`Matcher`, two `Task<()>`, a `UniformListScrollHandle`, a `Subscription`,
`Rc<RefCell<TerminalPainter>>`, `Arc<ClientShellSnapshot>` sources and a
`Keymap`. Beside it, and separable:

| File | Lines | Shape |
| --- | --- | --- |
| `palette/search.rs` | 231 | **pure**: `Fields`, `Query`, `Hit`, nucleo scoring, bounded haystacks, byte-range highlights |
| `palette/frecency.rs` | 271 | **pure + file**: half-life decay, `Boosts`, bounded JSON store |
| `palette/last_seen.rs` | 242 | **pure**: the last frame per tab, bounded by `MAX_TABS` |
| `palette/preview.rs` | 284 | **mostly pure**: Go To's facts and span geometry over a snapshot |
| `palette/go_to.rs` | 269 | **pure**: every destination as rows |
| `palette/projects.rs` + `project_create.rs` + `canonical.rs` | 425 | **pure + fs/worker**: discovery, planning, path canonicalising |
| `palette/agent_filter.rs` | 161 | **pure** (110) + 19 `impl` |
| `palette/keys.rs` | 134 | **pure** `PaletteKey::of` (17-58) + 82 `impl` |
| `palette/error.rs` | 52 | `thiserror` enum |
| `palette/render.rs` | 426 | `impl`: paints prepared entries |
| `palette/activate.rs` | 205 | `impl`: runs the chosen row |
| `palette/project_open.rs` | 287 | `impl`: opening/creating a project through the connection |
| `palette/resume.rs` | 50 | `impl`: reopening the palette as left |
| `palette.rs` | 817 | `Filter`, `Entry`, `Action`, `Identity`, `Target`, `rank_rows` (pure) + `Palette` and its methods |

`rank_rows` (`palette.rs:415-431`) is already the prepared-view shape the
settings design asks for: it takes `&[Entry]` and a `&mut Matcher` and returns
`Vec<Hit>`. `Entry`/`Action`/`Identity`/`Target` (`palette.rs:120-260`) are pure
but for `SharedString`.

## 2. Dependency audit: what menu code reads

### 2.1 `SpaceportWindow` — the dominant one

46 `impl SpaceportWindow` blocks (§0). The fields they read, by grep over the
menu roots:

| Field | Production sites | Who |
| --- | --- | --- |
| `config` (`Config`) | ~55 | every page: font size, line height, `palette.double_shift`, `keybinding_source`, `show_window_controls`, `notifications` |
| `endpoints` / `selected_endpoint` | ~15 | `devices/picker.rs:677,682`, `sessions.rs:111,154`, `mute.rs:15,32,64`, `devices.rs:115`, `host_menu.rs:93,391`, `workspace/requests.rs:62`, `palette/activate.rs:152,196`, `project_open.rs:64,236`, `pane_menu/moves.rs:103,154` |
| `live` (`LiveState`) | 3 | `mute.rs:29`, `workspace/requests.rs:35`, `palette/activate.rs:152` |
| `theme` (`Config::Theme`) | 6 | `host_menu.rs:182,208`, `add_device.rs:55`, `sessions/management.rs:147`, `shortcut_editor.rs:57`, `settings.rs:114` |
| `keymap()` | 4 | `palette.rs:328`, `palette/keys.rs:66`, `settings.rs:329`, `shortcut_editor.rs:159` |
| `menu` (`MenuState`) | all | the page itself |
| `sessions`, `device_themes`, `device_tags`, `catalog`, `port_forwards`, `usage` | 1–3 each | the picker, host menu, forwards, `Page::Usage` |

**Good news:** menus read **no** activity-indicator state. `Indicators`,
`mark_layer` and `StatusMark` do not appear in the menu roots at all — the only
match is a comment (`menu/colors.rs:47`) — so the daemon-owned status dot, the
app's other big projection, is not a menu blocker.

### 2.2 Two text-field entities, needed by nearly every page

| Entity | Production uses |
| --- | --- |
| `crate::search_input::SearchInput` (`Entity`, `Changed` events) | `menu/state.rs:91`, `menu/settings.rs:111-121`, `menu/sessions/management.rs:4,146`, `menu/devices/host_menu/theme.rs:96` |
| `crate::dialog_input::DialogInput` (305 lines, the one-line draft) | `menu/state.rs:8`, `menu/appear/kept.rs:7`, `menu/workspace.rs:5` |

No form, rename, tag or filter moves without these two. They are the menu
program's equivalent of settings' `fl!` blocker, and they are **not** part of
this program: they are a text-field extraction of their own (the settings design
names the same need at `docs/notes-settings-extraction.md:594`).

### 2.3 `crate::endpoint` — read as a type in four places, as state everywhere

Production `crate::endpoint` mentions in the menu roots are only 7:
`host_menu.rs:78,104,116` (WSL prefix, `LOCAL`), `devices/picker.rs:568` (a
parameter type), `palette/go_to.rs:228` (a `&[Endpoint]` parameter),
`palette/project_open.rs:57,139,250` (`LOCAL`). The real coupling is
`self.endpoints[i]` (§2.1), i.e. `endpoint::Endpoint`'s own state — which is why
the split plan lists `endpoint` (16.2k) beside `menu` as a mutual blocker.

### 2.4 Already below the app: the good half of the audit

| App path | What is actually there today |
| --- | --- |
| `crate::config::{Config, Theme, LayoutMode, TabLayoutMode, ChatLayoutMode, KeybindingSource, mix, corners, font_for}` | re-exports of `spaceport-settings` (`src/config.rs:27-45`) |
| `crate::controls::{Command, COMMANDS}` | `use spaceport_commands as controls;` (`src/main.rs:133`) |
| `crate::motion::{now, duration, progress, ENTER, LEAVE, Toggle, unfold, fold, pop_in}` | `spaceport-motion` |
| `crate::icons::{MUTED, AgentIcon}` | `spaceport-icons` |
| `spaceport_client::{Method, protocol::{ClientShellSnapshot, AgentStatus, …}}` | `spaceport-client`, with a closed `Method` enum (`method.rs:16-90`) |
| `spaceport_sidebar::device_scope::DeviceScope` | `spaceport-sidebar` |
| `spaceport_settings::palette::PaletteConfig` | `spaceport-settings` |

So the *settings, commands, motion, icons and protocol* a menu needs are already
extracted. What is missing below the app is **state** (`endpoint`), **text
fields**, **keymap** and **text**.

### 2.5 The blockers, ranked by how many lines they hold

| Blocker | Holds | Evidence |
| --- | --- | --- |
| `fl!` (~377 sites) | all page bodies and dialogs | §0 |
| `MenuState` as one aggregate | every page | `menu/state.rs:37-108` |
| `crate::search_input` + `crate::dialog_input` | every form and picker | §2.2 |
| `endpoint::Endpoint` state | picker, sessions, host menu, palette activation, mute | §2.3 |
| `crate::keymap` (506 + `keymap/`) | Keybinds page, shortcut editor, palette navigation | `palette.rs:328`, `settings.rs:159` |
| `crate::terminal_painter` | Go To's preview pane | `palette.rs:340-342` |
| `crate::processes` (`Watch`, `Listing`, `Identity`) | the pane process list | `pane_menu/processes.rs:10` |
| `crate::sessions` (407) | the sessions popover | `menu/sessions.rs` |
| `crate::pane_move` | pane moves | `pane_menu/moves.rs:154` |
| `crate::worktree::left_checkout` (14) | `WorkspaceTarget` | `menu/workspace.rs:88` |
| `crate::fonts::StyledFont` (49) | every panel | 6 files |
| `crate::contrast::ink_on_chrome` | `menu/colors.rs:32` | one call |
| `crate::titlebar::{show_controls, movable_with_controls}` | the in-window bar | `in_window/render.rs:18,66,216` |
| `crate::notifications::safe_text` | two notices | `whats_new.rs:7`, `version_mismatch.rs:8` |

### 2.6 The widget layer (#220), measured where it still lives

In this tree the widget layer is not yet a crate, so the measurement is of
*call sites* — which is also the cost #220 removes from the menu program:

| App path | Production sites in the menu roots |
| --- | --- |
| `crate::direction::{Logical, glyph, is_rtl, anchor}` | 6: `palette/render.rs:4`, `pane_menu/moves.rs:7,79`, `pane_menu/processes.rs:7`, `menu/chrome.rs:12,242`, `menu/chrome/render.rs:7,575` |
| `crate::toggles::{checkbox, radio}` | 3: `host_menu.rs:521`, `host_menu/tags.rs:179`, `devices/wsl.rs:266` |
| `crate::accessibility::{disabled, update}` | 4: `menus/in_window/render.rs:123,131,140,368`, `menu/workspace/popover.rs:356` |

**13 production call sites, 292 lines behind them.** Menus are the largest
consumer of the P3 widget layer (`docs/notes-settings-extraction.md:259`), so
#220 shrinks stage 4 by that much and by nothing else: no menu code can be
written against `spaceport_widgets` until a menu crate exists for it to be a
dependency of, and the app facade (`crate::direction::Logical`) keeps every call
site compiling unchanged in the meantime. Measure this table again after #220:
the counts should be identical and the paths should become `spaceport_widgets::`.

## 3. Natural seams, ranked by leverage

Each seam names the pattern it reuses. The two proven patterns are
`BrowserHosts` — "what one window offers the browser for a single call", a plain
borrow struct (`crates/spaceport-browser/src/hosts.rs:1-38`) — and chat's
`Labels` — a struct of strings the app fills, because a crate below the app has
no messages (`crates/spaceport-chat-core/src/labels.rs:1-30`).

**S1. The palette's ranking model. ~2,050 lines, pure, zero new blockers.**
`palette/search.rs` (231), `palette/keys.rs:17-58`, `palette/frecency.rs` (271,
needs `spaceport-state-file`), `palette/last_seen.rs` (242),
`palette/preview.rs`'s `facts` (105-190), `palette/go_to.rs` (269),
`palette/projects.rs` + `project_create.rs` + `canonical.rs` (425),
`palette/agent_filter.rs` (110), and `palette.rs:40-260,415-431`
(`Filter`, `Entry`, `Action`, `Identity`, `Target`, `rank_rows`). Reads a
`ClientShellSnapshot`, a `Config::Theme` for `Look`, and `Command`. Writes no
words: every `fl!` here is a label or a badge, so the crate returns the variant
and the app keeps its `match` to `fl!`. **Highest leverage in the program.**

**S2. The page set. 151 lines, pure, and used by everything else.**
`menu/page.rs` is already a closed enum with `pointer_anchored`, `pane_menu`,
`in_header` and `icon`. It moves whole, and it is the type every other moved
piece names.

**S3. Targets and typed requests. ~600 lines, pure.** This is the pattern
`pane_move` and `backdrop` already set ("a value type for what the crate reads,
and a parameter for what the app owns"):

```rust
// crates/spaceport-gpui/src/pane_menu.rs:91-146
impl Action {
    fn request(self, target: &Target) -> Option<(Method, Value)> {
        Some(match self {
            Self::SplitRight | Self::SplitDown => (
                Method::PaneSplit,
                json!({ "target_pane_id": target.pane, "direction": …, "focus": true }),
            ),
            …
```

`menu/workspace.rs:33-84` (`WorkspaceTarget::new/group_key/request`, with its
`boot_id` fence and `StaleWorkspace`), `pane_menu.rs:19-87`, `tab_menu.rs:22-99`,
`group_menu.rs:27-97` and `menu/devices/host_menu/actions.rs` are the same shape.
Note that these build raw `json!` against a *closed* `Method` enum
(`spaceport-client/src/method.rs:16-90`): typed parameter structs are the
AGENTS.md-preferred fix and belong with the move, not after it.

**S4. Selection stepping. ~120 lines, pure, currently duplicated four times.**
The wrap arithmetic is written out in `menu/chrome/keys.rs:194-215` (workspace
tiles), `:216-232` (the application menu), `palette/keys.rs:113-135`, and again
inside `tab_menu`/`pane_menu`/`picker` key handlers. One `Step { at, len }` with
`up`/`down` in the moved crate replaces them.

**S5. Appear and leave. 273 lines, near-pure.** `menu/appear.rs:37-80` keeps
`Option<Shown>`, `since: Option<Instant>`, `leaving`, `kept`, `Toggle` — the only
GPUI values are `Pixels` and `Rgba`. Moves with `motion` already below it;
`f32` in place of `Pixels` if the crate stays gpui-free.

**S6. Colour policy. 120 lines, pure.** `menu/colors.rs` computes accents and a
WCAG-checked danger colour from a `Theme`, and `spaceport-settings` already has
`contrast.rs` and its tests. This is a move into `spaceport-settings`, not into
the new crate.

**S7. The device CLI. 271 lines, no GPUI at all.** `menu/devices/setup.rs` runs
the installed `herdr` CLI (`claim_saved`, `remove`, `rename`, `run_bounded`,
`last_line`) — bounded output, a 15s timeout, an environment allowlist. It
belongs beside `spaceport-remote`'s host scripts
(`crates/spaceport-remote/src/host_script.rs:182`).

**S8. The picker's line model. ~300 lines.** `devices/picker.rs:18-76` (`Line`,
`Pick`, `selectable`, the stable `Pick` identity across arriving rows) and
`Folding` (29-49, an `Instant` and a height) plus `menu/sessions.rs`'s `Entry`/
`Row` (`picker.rs:8`). Needs an id-only hosts struct, not `Endpoint`.

**S9. The popup chrome. ~1,530 lines, GPUI-bound, movable only as a kit.**
`chrome.rs` (363) + `chrome/render.rs` (685) + `menus/in_window/render.rs` (484):
`menu_layer` measures (44-79), `menu_panel` assembles (81-90), `Layer` carries
twelve facts (`chrome/render.rs:670`). This is `spaceport-browser`'s terminal
shape — a GPUI crate taking a hosts struct — and it is stage 4, not stage 1.

**S10. The application menu tree. 492 lines, data.** `menus.rs:105-490` builds
`Vec<gpui::Menu>` and dispatches `Command`-carrying GPUI action structs
(`RunCommand { command }`, `SetLayout { mode }`). It could become data — a table
of `(Command, fl! id, checked-predicate)` — with the app installing it, which is
also how the 55 `fl!` sites there would leave. Worth it only after `Command`-as-
identity lands elsewhere; defer.

## 4. The staged plan

Each stage is one PR, shippable alone, with a survey first and an exit criterion.
Nothing here changes behaviour: every stage is a move plus a facade, as the
settings stages 1–2 were.

### Stage 0 — Survey (no code)

Rerun §0's commands on the commit you start from; record base revision, the 46
`impl` count, the ~377 `fl!` count, the 13 widget sites, and whether
`crates/spaceport-widgets` exists yet (if it does, re-measure §2.6 by path).
Diff the result against this note and correct it in the same PR.

### Stage 1 — `spaceport-menu`: the page set, targets and requests

The pure half of §3's S2, S3 and S4, with **no text**.

- New crate `crates/spaceport-menu` (`spaceport-client`, `spaceport-settings`,
  `serde_json`, `thiserror`, `test-support`), its own `Error`/`Result`, and
  `#[cfg(test)] mod tests;` split into topic files under `tests/` as AGENTS.md
  asks.
- `menu/page.rs` → `spaceport_menu::page` (`Page`, `WorkspaceAction`,
  `WorkspaceMenuAction`, `pointer_anchored`, `pane_menu`, `in_header`, `icon`).
  `Page::Usage(crate::usage::Provider)` and `Page::Dialog(WorkspaceAction)` are
  already-cross-feature payloads; keep them.
- `menu/workspace.rs:16-157` → `spaceport_menu::workspace`: `WorkspaceTarget`
  (`new`, `group_key`, `request`), `known_worktree`, `left_checkout`,
  `suggested_tab_name`, `suggested_workspace_name`, `chosen_label`, `heads_group`.
  Move `crate::worktree::{checkout_root, left_checkout}` (14 lines) with it.
- `pane_menu.rs:19-185`, `tab_menu.rs:22-135`, `group_menu.rs:27-97` →
  `spaceport_menu::{pane, tab, group}`: `Target` + `Action` + `ACTIONS` + the
  filtered `actions()` list, with `request` returning `(Method, Value)`.
- `Step` (S4) and one test file per moved model.
- **Labels stay app-side.** `Action::label`/`Page`'s title do not move in this
  stage: each moved `Action` keeps its variant, and a new app-side
  `menu/labels.rs` holds the `match` to `fl!`. That is `docs/i18n.md`'s rule —
  the crate returns the value, the app writes the words — and it keeps stage 1
  from inventing a 400-field `Labels`.
- Facade: `crate::menu::{Page, WorkspaceAction, …}` re-export exactly as
  `src/config.rs` does, so no importer changes.

**Exit criterion.** `cargo check -p spaceport-menu -p spaceport-gpui`,
`cargo test -p spaceport-menu`, `cargo test -p spaceport-gpui menu`,
`just check-file-size`, `just check-sorted`, `just check-i18n`, `cargo fmt`;
`cargo metadata` refreshes the lock. The app diff is `use` lines plus moved
bodies; `git diff` shows no changed `fl!` string and no changed `json!` payload.

### Stage 2 — `spaceport-menu`: appear, the picker's lines, sessions' rows

S5 and S8. `menu/appear.rs` + `appear/kept.rs`, `devices/picker.rs:18-76`
(`Line`, `Pick`, `Folding`), `menu/sessions.rs`'s `Entry`/`Row`, and the
`devices/core_catalog.rs:33-52` text helpers (which return `String` today —
convert them to the value the app words).

Survey first: which reads of `Endpoint` these need, and whether an id-only
`MenuHosts` borrow struct (the `BrowserHosts` shape) covers them; if not, stop at
the pieces that are truly pure and leave the rest to stage 5.

**Exit criterion.** As stage 1, plus: the picker's fold/unfold tests move to
pure time-stepped tests (`Instant` in, `f32` out), and `menu/devices/tests/fold.rs`
still passes unchanged.

### Stage 3 — `spaceport-palette`

S1, the biggest single win. `search`, `keys`' `PaletteKey`, `frecency` (over
`spaceport-state-file`), `last_seen`, `preview`'s facts, `go_to`, `projects`,
`project_create`, `canonical`, `agent_filter`, and `palette.rs`'s `Filter`,
`Entry`, `Action`, `Identity`, `Target`, `rank_rows`.

Survey first: whether `SharedString` in `Entry` becomes `String` (it should — it
is a label) and how many `fl!` sites in these files are *data* (badges like
`palette-badge-project`) versus words the app must say.

**Exit criterion.** `cargo test -p spaceport-palette` carries
`palette/tests/{agent_filter,background_ranking,frecency_ranking,go_to_search,notes,preview_layout}.rs`
and `search/tests.rs` as pure tests; `cargo test -p spaceport-gpui palette`
still passes with the render and activate paths untouched.

### Stage 4 — the chrome becomes a GPUI crate

S9, and only after #220's `spaceport-widgets` is a dependency: `chrome.rs`,
`chrome/render.rs`, `menus/in_window/render.rs` — `spaceport-menu` gains `gpui`
and becomes the crate it is already named for, as `spaceport-browser` did.
Requires `fonts::StyledFont` (49 lines) and a `MenuHosts`/`Layer` split so the
paint takes prepared data.

Survey first: what `menu_panel`/`menu_page_body` actually read of `self` (today
it is `self.config`, `self.menu`, `self.theme` and twelve `Layer` facts), and
whether `menus/in_window/render.rs:18,66,216`'s `crate::titlebar` reads can be
prepared into `Layer` fields.

**Exit criterion.** The in-window bar and the popup chrome draw from the crate;
`menus/tests/{bar,dragging,resizing,shortcut_labels,navigation,config_change}.rs`
and `menu/appear/tests.rs` pass unchanged; native menu smoke on macOS and the
Linux `native.rs` check are run or explicitly deferred with a note.

### Stage 5 — the page bodies that could follow, and the ones that cannot

Only after four things exist that this program does not build: a text-field
crate (SearchInput + DialogInput), `spaceport-endpoint`, `spaceport-keymap`, and
a text story for ~377 sites. Order when they do, cheapest first:
`HostMenu`'s actions → `workspace/popover.rs` → `tab_menu`/`group_menu` page
bodies → `pane_menu` (`processes.rs` after `crate::processes`) → `settings.rs`
(the Keybinds page, after `spaceport-keymap`) → the device picker, Add Device,
WSL and sessions last (they are the ones made of `Endpoint` state).

`devices/setup.rs` (S7, 271 lines) is the exception: it can go to
`spaceport-remote` at any time, independently of every stage above.

## 5. Honest sizing

| | Lines | Share |
| --- | --- | --- |
| stages 1–3 (models, requests, palette) | ~3,400–3,900 | 17–19% |
| stage 4 (chrome kit) | ~1,530, *if* #220 lands and the hosts split works | +7% |
| realistically movable in this program | **~4,300–4,900** | **21–24%** |
| stays app-side | **~15,300–15,800** | **76–79%** |

Settings concluded that ~5,300 of 11,520 lines stay (46% stays, 54% moves).
Menus are the other way round: **three quarters stay**, and the quarter that
moves is the smallest, most mechanical part — target capture, `json!` payloads,
fuzzy scoring. Say so plainly to whoever plans the sidebar around this: after
this program the sidebar's `menu` blocker is smaller by the page set and the
ranking, not by the pickers.

What stays, and exactly why:

- **The pickers and forms, ~4,300 lines** (`devices/picker.rs` 860,
  `sessions*.rs` 994, `add_device*` 621, `devices.rs` 367, `rail.rs` 286,
  `wsl.rs` 531, `host_menu*` 1,832): `Endpoint` state, text fields, workers.
- **The dialogs and editor, ~1,900 lines** (`settings.rs` 796,
  `shortcut_editor.rs` 297, `close_modal.rs` 483, `dialog_input.rs` 305): text
  fields, `Keymap`, `fl!`.
- **The paint of every page, ~2,400 lines**: element building from prepared
  state, which is the app's job by the fork's own rule ("the app keeps the
  views").
- **The application menu bar, 492 lines**: GPUI actions and 55 `fl!` sites.

The one thing this program genuinely unblocks is the **palette**, which is a
feature of its own (3,854 lines, 19% of the module) and whose model is pure
enough to leave without any of those four homes.

## 6. Risks, ranked

1. **Text arrives earlier and bigger than it did for settings.** ~377 `fl!`
   sites against settings' ~150, and 55 of them are in the application menu bar
   alone. Stage 1 avoids the problem by moving values and keeping words
   app-side; stage 3 meets it in the palette's badges; stage 4 cannot avoid it
   (the bar's headings are `fl!`). **Mitigation:** no stage may introduce a
   `Labels` struct of more than ~15 fields; prefer "crate returns the variant,
   app matches to `fl!`".
2. **`MenuState` is one aggregate and every stage touches it.** 44 fields
   (`menu/state.rs:37-108`), and `new()`/`reset()` are append-only lists that
   every open menu PR edits. **Mitigation:** stage 1 does not split it. The
   split is a later, separate change: `MenuState` keeps page *ownership*
   (`Option<Page>` and the fence), and each page's model moves beside its page.
3. **`impl SpaceportWindow` density (77%).** Browser's whole extraction was one
   612-line inversion over 4,390 lines; this is 32 blocks over 11,241, so the
   inversion is browser-shaped work at three times the size — and unlike
   browser it cannot be done in one pass, because the page bodies depend on
   four homes that do not exist.
4. **Two text-field entities gate everything.** `SearchInput` and `DialogInput`
   are needed by every form, picker, rename and filter. Nothing in stages 1–4
   removes that dependency; stage 5 is blocked until a text-field crate exists.
5. **`menu` and `endpoint` block each other.** The picker, sessions, host menu,
   mute, palette activation and project open all read
   `self.endpoints[self.selected_endpoint]`. Either `endpoint` moves first or
   those pages stay; the split plan's order (menu before endpoint) holds only
   because the *pure* part of menu does not need it.
6. **Test coverage is window-level.** ~60 test files under these roots, almost
   all `#[gpui::test]` driving a whole `SpaceportWindow`. A moved model needs
   new pure tests, and the deleted integration coverage must be replaced case by
   case at the same time, or the move silently loses its fence tests (stale
   boot, removed tab, changed command) — the ones that matter most.
7. **Merge-conflict hotspots.** `MenuState::new`/`reset`, `Page`, `menu.rs`'s
   `mod` list, `chrome/keys.rs`'s page dispatch chain, `chrome/render.rs`'s page
   match, and `menus.rs`'s tree. Land stages in quiet weeks, and expect to
   resolve in `Page` and `MenuState` every time.
8. **Crate naming and scope decisions** (§8) if `spaceport-menu` later holds
   GPUI while `spaceport-palette` sits beside it depending on it.

## 7. Stage 1's exact task list

Branch from the base revision stage 0 recorded.

1. `crates/spaceport-menu/Cargo.toml`: `spaceport-client`, `spaceport-settings`,
   `serde_json`, `thiserror`, `test-support` (dev); `[workspace.dependencies]`
   entry; `spaceport-gpui`'s `[dependencies]`; AGENTS.md's crate table row; no
   text, so no `docs/i18n.md` change.
2. `src/lib.rs`: `pub use error::{Error, Result};`, `mod page; mod pane; mod tab;
   mod group; mod step; mod workspace;`, and `#[cfg(test)] mod tests;`.
3. `src/error.rs`: the moved fences as variants — `StaleWorkspace`,
   `EmptyWorkspaceLabel`, `LocalWorktreeNote`, `StalePane` — matching what
   `crate::Error` has today; add `Menu(#[from] spaceport_menu::Error)` to the
   app's root `Error` (sorted).
4. `src/page.rs`: the whole of `menu/page.rs` (151 lines, enums at 8-74, helpers
   at 119-180). Keep `// keep-sorted` markers on `Page`.
5. `src/workspace.rs`: `menu/workspace.rs:16-157` verbatim, plus
   `crate::worktree::{checkout_root, left_checkout}` as a private `worktree`
   module. `WorkspaceTarget::request` keeps returning `(Method, Value)`.
6. `src/pane.rs`, `src/tab.rs`, `src/group.rs`: `Target` (`pane_menu.rs:19-87`,
   `tab_menu.rs:22-99`) — `pane_menu`'s `choices: Vec<crate::pane_move::Choice>`
   is an app type: either move `pane_move::Choice` too or replace it with the
   data the crate needs (ids) and let the app rebuild `Choice`. Decide in the PR
   body; do not leave a `spaceport-gpui` path in the crate. Then `Action`
   (`pane_menu.rs:91-146`, `tab_menu.rs:100-135`, `group_menu.rs:27-97`),
   `ACTIONS`, and the filtered `actions()` lists.
7. `src/step.rs`: `Step { at: usize, len: usize }` with `up`/`down`/`first`/
   `keep`, replacing the four copies; three unit tests including the empty-list
   case (`len == 0` must not divide by zero — the current palette code guards
   with `if count == 0 { return; }` at `palette/keys.rs:114`).
8. App side, mechanical: `menu/page.rs` and `menu/workspace.rs` become
   `pub(crate) use spaceport_menu::{…};` facades; `pane_menu.rs`/`tab_menu.rs`/
   `group_menu.rs` keep their `impl SpaceportWindow` and re-import `Target`/
   `Action`; a new `menu/labels.rs` holds the moved `label()` matches; delete the
   moved bodies. Importers of `crate::menu::Page` change nothing.
9. Tests: move the model-level ones (`menu/sessions_tests.rs`'s target tests,
   `workspace_tests/{targets,naming,group_fold,left_checkout}.rs`,
   `pane_menu/tests/{moves,close_option,excluded_carry}.rs`'s pure parts) to the
   crate; keep the window-driven ones app-side and make them compile against the
   facade.
10. Verify (§4 stage 1's exit criterion), record the counts stage 0 measured
    against the counts after, and state in the PR which tests moved and which
    stayed.

## 8. Open questions for the owner

1. **Crate names.** `spaceport-menu` (model now, GPUI kit at stage 4) plus
   `spaceport-palette`, or one `spaceport-menus` holding both? The split plan
   already has `spaceport-sidebar` as a GPUI crate, so a crate that gains GPUI
   at stage 4 has precedent; the palette is a separate feature and reads
   different things.
2. **Does `pane_move::Choice` move to `spaceport-menu`** (with `spaceport-panes`
   already owning the pane tree) or get replaced by ids at the boundary?
3. **Where do `menu/colors.rs`'s accents belong** — `spaceport-settings`'s
   `contrast.rs`, or a `spaceport-theme` that several features would share?
   `menu/colors.rs:32` and `spaceport-settings`' contrast rules currently both
   exist.
4. **Is `spaceport-remote` the home for `devices/setup.rs`'s `herdr` CLI calls**,
   or does that deserve `spaceport-herdr-cli` (three verbs: claim, rename,
   remove)?
5. **Does the palette's `fl!` stay in the app entirely** (badges returned as
   variants), or does `spaceport-palette` take a small `Labels` for its six
   badge strings? The first is cheaper and matches `docs/i18n.md`; the second
   avoids a 30-arm match in the app.
