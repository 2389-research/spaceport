# Extracting the Settings window from `crates/spaceport-gpui`

Design for the extraction named in the split plan's feature table as a
prerequisite of the three large features, and the first one whose subject is a
*window* rather than a model: `src/settings_window.rs`, `src/settings_window/`,
`src/settings_panel.rs` and `src/settings_panel/`. It is a design only: no
production code changed, no build run.

`crates/spaceport-settings` already owns the *values* (`Config`, `SidebarStyle`,
`preferences::Preference`, `herdr_settings::Settings`, `Theme`); this note is
about the *window* above them. The two are different jobs and the naming
collision is the first thing to fix: the crate that comes out of this program
must not be called `spaceport-settings` again.

**Verdict.** The Settings window is not `browser`. Browser was 4,390 production
lines of *plain struct with `impl SpaceportWindow` methods*, and its extraction
was one 612-line mechanical inversion onto a borrow struct. This module is the
opposite shape: a single GPUI entity, `SettingsWindow` (`settings_window.rs:152`),
with **200+ methods in 35 `impl SettingsWindow` blocks**. Only about **2,340 of
its 11,520 ordinary production lines are not entity methods** — 1,240 of those
gpui-free, 1,100 a GPUI paint kit. There is nothing to invert: the entity already
owns itself, `cx.listener` already closes over it, and `Context<Self>` already
means `Context<SettingsWindow>` in every one of those methods. What blocks the
move is not ownership but **the nineteen kinds of read in §1.3**, seven of which
need a home that does not exist.

So the honest shape is three parts:

1. **~1,240 production lines are gpui-free policy** — the save-queue state
   machine, the sidebar draft diffing, the row catalog, the section list, the
   number arithmetic. These belong in `spaceport-settings`, next to the
   `Preference` and `SidebarStyle` they compute over, and they move with no new
   crate and no behaviour change.
2. **~1,100 production lines are a self-contained GPUI paint kit** — the row
   builder, the number field, the dropdown, the swatches. These need a GPUI crate
   (a new `spaceport-settings-window`) and three small app-wide widget
   prerequisites (`direction`, `toggles`, `accessibility`, 292 lines together)
   that chat, menus and the titlebar also want.
3. **The pages, the controls and the theme browser — ~5,300 ordinary production
   and ~8,000 test lines — do not move** until the localised text they are made
   of has a home below the app. That is the same blocker `chat` has (174
   messages), far smaller here in features and far larger here in call sites:
   **~150 `fl!` sites in 24 production files** over ~300 message IDs. §7 is that
   decision, and it gates stages 5 and up. `docs/i18n.md`'s rule that "messages
   are the app's" was written for crates that return values; a crate that *draws*
   300 sentences has to be given a lookup, and no extraction of this module can
   invent one.

The plan that follows is therefore greedy: two stages that are pure moves with
no new crate of their own, one that adds a small widget layer several features
want anyway, then the gated ones, each with a survey step and an exit criterion.

## 0. The measurements, and how to repeat them

Every line count below is the file's own length, read from this tree. The
commands are the ones to re-run before each stage:

```sh
git log --oneline -1                                    # record the base revision
find crates/spaceport-gpui/src/settings_window* -name '*.rs' | xargs wc -l | tail -1
find crates/spaceport-gpui/src/settings_window -name '*.rs' \
  | grep -v '/tests' | grep -v 'tests\.rs$' | xargs wc -l | tail -1
rg -c '^impl SettingsWindow' crates/spaceport-gpui/src/settings_window*
rg -l 'crate::settings_window|settings_panel' crates/spaceport-gpui/src --glob '!**/settings_window*'
rg -o 'fl!\("settings[^"]*"' crates/spaceport-gpui/src/settings_window* | wc -l
```

| | Lines |
| --- | --- |
| `settings_window.rs` | 729 |
| `settings_window/` production files (44) | 11,791 |
| — of which `native.rs` + `native/` (macOS, `integration-test` only) | 1,327 |
| **ordinary production** | **11,520** |
| test files under `settings_window/` (66) | ≈9,750 (derived: the brief's 22.6k minus the measured 12,847 production total) |
| `settings_panel.rs` + `settings_panel/` (5 test files) | 174 production, 194 + 4 files of tests |

Per-file production lines, because the stages are sized from this table:

| File | Lines | Shape | File | Lines | Shape |
| --- | --- | --- | --- | --- | --- |
| `themes.rs` | 885 | entity | `themes/appearance.rs` | 247 | entity |
| `settings_window.rs` | 729 | entity + `Render` | `themes/grid.rs` | 258 | entity + 35 pure |
| `persistence.rs` | 649 | entity | `themes/system.rs` | 222 | entity |
| `sidebar.rs` | 530 | entity + 180 pure | `ui/number.rs` | 304 | kit + 40 pure |
| `controls/fonts.rs` | 527 | entity | `ui/popover.rs` | 216 | kit |
| `pages/sidebar.rs` | 461 | entity render | `ui/swatches.rs` | 193 | kit + 10 pure |
| `inputs.rs` | 398 | entity | `ui.rs` | 384 | kit |
| `controls/backdrop.rs` | 390 | entity | `remote_history.rs` | 320 | entity |
| `controls/backdrop/ruler.rs` | 240 | entity | `controls.rs` | 305 | entity |
| `pages/general.rs` | 362 | entity render | `pages/theme.rs` | 277 | entity render |
| `pages/status_bar.rs` | 196 | entity render | `pages/notifications.rs` | 225 | entity render |
| `pages/tabs.rs` | 188 | entity render | `controls/preferences.rs` | 184 | entity |
| `pages/indicators.rs` | 173 | entity render | `pages/fonts.rs` | 156 | entity render |
| `navigation.rs` | 245 | entity | `catalog.rs` | 215 | **pure + data** |
| `search.rs` | 193 | entity + 90 pure | `section.rs` | 166 | **pure + data** |
| `scrollbar.rs` | 140 | entity | `persistence/lifecycle.rs` | 263 | **pure** |
| `pages/configuration.rs` | 137 | entity render | `layouts.rs` | 132 | entity + 4 fns |
| `controls/chat_env.rs` | 131 | entity | `persistence/operation.rs` | 96 | ~pure |
| `launch_at_login.rs` | 99 | entity (Windows) | `pages/window.rs` | 100 | entity render |
| `controls/indicators.rs` | 86 | entity | `pages/language.rs` | 90 | entity render |
| `controls/forms.rs` | 67 | kit | `native.rs` + `native/` | 1,327 | probe (test only) |

Two findings worth stating before anything else:

- **`settings_panel.rs` is not a panel.** It is 174 lines of `impl
  SpaceportWindow` that load Herdr's *shared* settings on a worker
  (`settings_panel.rs:17-56`) into `SpaceportWindow.settings` (`window.rs:278`),
  then re-apply theme/sort. It never draws anything. It belongs with
  `spaceport-settings`' `herdr_settings` and the window's own state, not with
  this extraction, and its four test files are `SpaceportWindow` tests. Drop it
  from the scope; it is listed here only because its name says otherwise.
- **`pages/gitlab.rs` is dead.** It is 1 line and empty, `pages.rs:3-14` declares
  no `mod gitlab`, and the string `gitlab` appears nowhere else under
  `settings_window*`. `tests/gitlab.rs` is likewise unclaimed by `tests.rs`.

## 1. Code context: what the Settings UI reads

### 1.1 The entity

`SettingsWindow` is a `gpui` entity with ~65 fields (`settings_window.rs:152-227`)
and one `Render` impl (`:619-729`). Its state, by responsibility:

| Group | Fields (in `settings_window.rs:152-227`) |
| --- | --- |
| appearance it edits | `config: Config`, `keymap: Keymap`, `loaded_config: Config`, `theme: Theme`, `shared: Option<herdr_settings::Settings>` |
| the window's own chrome | `caption: Caption`, `focus: FocusHandle`, `body_scroll`, `navigation_scroll`, `thumb_grab`, `navigation_width: PanelWidth`, `viewport_width` |
| navigation | `section: Section`, `search_results: Option<Vec<usize>>`, `nav_search: Entity<SearchInput>` |
| sub-editors | `themes: ThemeBrowser`, `controls: Controls`, `chat_env: ChatEnvSettings`, `dropdown: Option<Dropdown>`, `number_edit`, `number_preview`, `hex_edit` |
| drafts and their revisions | `theme_intent`, `theme_revision`, `layout_intent`, `sidebar_intent`, `load_revision`, `theme_load_failed`, `theme_loading`, `theme_waiting`, `theme_light` |
| persistence | `persistence: Lifecycle`, `save_completion: Receiver<SaveCompletion>`, `saved_sample: Option<Sample>` |
| app plumbing | `source: WeakEntity<SpaceportWindow>`, `new_window_target: Option<ConnectTarget>`, the `_source`/`_appearance`/`_theme_guard`/`_search`/`_quit` subscriptions, `_watch: Option<Task<()>>` |

`open()`/`open_with` (`:76-145`) is the app's entry point: it centres a 960×780
window, reuses a retained model after a detached save, installs
`on_window_should_close`, and marks the window with `crate::taskbar::mark`
(`:120`). `key_bindings()` (`:51-66`) binds close on two platforms' chords.

### 1.2 What it reads from `spaceport-settings` (already below the app)

Every `crate::config::` and `crate::herdr_settings::` path in this module is a
re-export: `src/config.rs` is the façade over `spaceport_settings`, and
`src/herdr_settings.rs:12` is literally `pub(crate) use
spaceport_settings::herdr_settings::*`. The list, by file:

| Path | Sites |
| --- | --- |
| `crate::config::{Config, Theme, ThemeName, FOLLOW_HERDR}`, `corners`, `mix`, `contrast::Contrast` | `settings_window.rs:45-50`, `themes.rs:6-13`, `ui.rs:6-9`, `persistence.rs:10-14`, `pages/*` throughout |
| `crate::config::{FontFace, FontConfig, FONT_SIZE_RANGE, MAX_PADDING, Axis, Side, Padding, SoundConfig}` | `controls.rs:16-21`, `inputs.rs:11-15`, `controls/backdrop.rs:6-9`, `pages/window.rs:13` |
| `crate::config::{LayoutMode, TabLayoutMode, ChatLayoutMode, ChatLauncher, Feature, StatusIcons, AgentsPlacement, SelectMode, SidebarStyle, SidebarOverrides, DeviceFooter, ClipboardToast, ClipboardToastPosition, Backdrop}` | `layouts.rs:6`, `persistence/lifecycle.rs:4`, `controls/indicators.rs:6`, `pages/{general,sidebar,tabs}.rs`, `catalog.rs:15-18` |
| `crate::config::preferences::Preference` | 9 production sites, 14 test files |
| `crate::config::watch::{Watch, Fingerprint}`, `crate::config::view_settings_path` | `persistence.rs:28-45` |
| `crate::herdr_settings::{Settings, Edit, RemotePaneHistory, TabBarPosition}` | `persistence/operation.rs:4`, `remote_history.rs:9`, `themes.rs:9`, `pages/tabs.rs` |

This half is **not** a blocker: it is already one crate down and the move is a
path rename.

### 1.3 What it reads from the app

This is the whole dependency audit, counted in production code only. Test-only
reads (`crate::sidebar::layout_tests::fixture_window` in 20 test files,
`crate::config::preferences::Preference` in tests) are excluded.

| Read | Sites | file:line | What it is |
| --- | --- | --- | --- |
| `SpaceportWindow` | ~15 | `settings_window.rs:152,166,229-380,394-411,585-600` | the window it was opened from: `.config`, `.keymap`, `.theme`, `.settings.shared`, `.endpoints`, `.integrations.busy`, `.load_integrations`, `.menu.page`, `.theme_save_in_flight()` |
| `crate::app::InitialAppearance` | 8 | `settings_window.rs:232,331,334,344,353,454`, `themes.rs:577`, `sidebar.rs:287`, `layouts.rs:80` | the GPUI global `{ config, keymap, theme, error }` (`app.rs:41-48`) that every window reads and Settings publishes |
| `crate::app::light_appearance(cx)` | 7 | `themes.rs:422,616,716`, `themes/system.rs:66,157,209`, `pages/theme.rs:105` | whether the system is light (`app.rs:78-83`); picks the side of a theme pair |
| `crate::log_window::set_appearance` | 2 | `settings_window.rs:463`, `themes.rs:583` | tells the log window to follow |
| `crate::actions::rebind_keys` | 1 | `settings_window.rs:466` | reinstalls key bindings after a keymap change |
| `crate::menus::{changed, install}` | 3 | `settings_window.rs:443,468`, `layouts.rs:85` | reinstalls menus |
| `crate::menu::{Page, accent}` | 3 | `settings_window.rs:449`, `themes.rs:563` (`Page::Themes` fences previews), `controls.rs:262` (`menu::colors::accent`, 2 lines over `theme.palette[4]`) | reads the *source window's* menu state |
| `crate::window::run_window_command`, `crate::controls::Command`, `crate::RunCommand` | 6 | `settings_window.rs:645-672` | close / settings / new-window commands |
| `crate::app::open_additional_window` | 1 | `settings_window.rs:657` | "New window" from this window |
| `crate::titlebar::{options, header, frame}` | 3 | `settings_window.rs:115,640,722` | native titlebar and frame (277 lines) |
| `crate::caption::Caption` | 2 | `settings_window.rs:153,621` | the macOS caption buttons (45 lines) |
| `crate::taskbar::mark` | 1 | `settings_window.rs:120` | platform taskbar marking |
| `crate::panel_resize::{PanelDrag, Side, handle, PanelWidth, SETTINGS_NAVIGATION}` | 5 | `navigation.rs:10`, `settings_window.rs:192,303` | the draggable section-list edge (145 lines) |
| `crate::font_picker::FontTarget` | 2 | `controls.rs:19`, `native/fonts.rs:7` | 9-line enum: `All` or one `FontFace` |
| `crate::keymap::Keymap` | 3 | `settings_window.rs:157`, `persistence.rs:18`, `layouts.rs` tests | the parsed keymap, held so a published appearance carries a matching pair |
| `crate::search_input::{SearchInput, Changed}` | 8 | `controls.rs:20,35,36,58,68`, `themes.rs:11,123,153`, `inputs.rs:15,182`, `sidebar.rs:9,173,481`, `ui/number.rs:6,82`, `ui/swatches.rs:9,46`, `controls/chat_env.rs:13,26` | a 668-line IME-aware text field entity |
| `crate::i18n::{fl, message, english, Localize}` | **~150 `fl!` + ~10 `message`** | 24 production files, §1.4 | localised text |
| `crate::direction::Logical` | 12 files | `settings_window.rs:41`, `ui.rs:5`, `ui/*`, `themes/*`, `controls/forms.rs:5`, `controls/backdrop.rs:10` | start/end padding, rows, text alignment (148 lines) |
| `crate::toggles::switch`, `crate::accessibility::disabled` | 2 | `ui.rs:233`, `ui.rs:243` | the switch track and the disabled contract (98 + 46 lines) |
| `crate::agent_skill::{AgentSkill, Choice, executable, install, remove, text}` | 6 | `pages/general.rs:15,282-286` | the managed CLI skill row (`agent_skill.rs`, 317 lines) |
| `crate::core_feature::restart_required` | 1 | `pages/general.rs:116` | 35-line Core opt-in gate |
| `crate::sidebar::paint::PAINTERS` | 1 | `pages/indicators.rs:137` | the status-icon painters (`sidebar/paint.rs`, 57 lines, app module) |
| `crate::browser::Thumb` | 1 | `scrollbar.rs:7` | already `spaceport_browser::Thumb` through the façade |
| `crate::launch_at_login` | 4 | `launch_at_login.rs:8` | Windows platform module |
| `crate::Error` / `crate::Result` | ~12 | `themes.rs:297,731`, `sidebar.rs:315`, `persistence.rs`, `remote_history.rs:69,241`, `layouts.rs:100`, `pages/{status_bar,sidebar,theme}.rs` | the app root error, only ever `crate::Error::from(settings_error)` and `MissingHome` in tests |

### 1.4 i18n

Thirteen `.ftl` files carry this window's words (`locales/en-US/settings*.ftl`),
≈300 message IDs: `settings.ftl` (~100), `settings-theme.ftl` (~55),
`settings-general.ftl` (~30), `settings-sidebar.ftl` (~28),
`settings-fonts.ftl` (~30), `settings-indicators.ftl` (20),
`settings-notifications.ftl` (18), `settings-tabs.ftl` (14),
`settings-status-bar.ftl` (12), `settings-integrations.ftl` (6),
`settings-configuration.ftl` (5), `settings-remote.ftl` (1), and
`settings-panel.ftl` (2, used by `settings_panel.rs`, not this window).

Two styles already coexist, and the difference decides §7:

- **Data**: `Section::message()` (`section.rs:85-96`), `Group::message()`
  (`:42-50`) and every `Entry.title`/`Entry.description`
  (`catalog.rs:18-46,111-191`) are message **IDs** resolved through
  `crate::i18n::message(id, &[])`. All 75 catalog rows and 10 sections already
  pass IDs, not sentences.
- **Sentences**: ~150 `fl!("settings-…")` sites in 24 production files build
  text inline, inside element builders.

### 1.5 What the rest of the app wants back

Eight production names, and that is the entire reverse-import surface:

| Name | Site |
| --- | --- |
| `open(cx.weak_entity(), cx)` | `menu/settings.rs:132` |
| `key_bindings()` | `actions.rs:138` |
| `theme_pending(cx)` | `theme_picker.rs:68`, `settings_panel.rs:113,151` |
| `theme_load_revision`, `layout_load_revision`, `sidebar_load_revision` | `menu/settings.rs:177-179` |
| `apply_loaded_theme`, `apply_loaded_layout`, `apply_loaded_sidebar` | `menu/settings.rs:195-197,241,252` |
| `verify_native`, `capture_branding`, `verify_core_opt_in` (`integration-test`) | `smoke_preferences.rs:32`, `smoke_local_controls.rs:87-89` |

The eight `*_revision`/`apply_loaded_*` functions are the draft-fencing seam
between this window and the main window's own theme picker (`menu/settings.rs`,
796 lines): the same globals serve both, so they cannot move alone.

## 2. Dependency audit

### 2.1 Row by row: does it block, and does a home exist?

| Need | Blocks? | Home today | Action |
| --- | --- | --- | --- |
| `Config`, `Theme`, `SidebarStyle`, `Preference`, `herdr_settings` | no | **`spaceport-settings`** | path rename |
| `corners`, `mix`, `contrast::Contrast`, `parse_hex`, `FONT_SIZE_RANGE` | no | `spaceport-settings` | path rename |
| `browser::Thumb` | no | `spaceport-browser` | path rename |
| `Direction` | the *type* no, the `Logical` **trait** yes | `spaceport-i18n` has `Direction`; the trait is app-side (`direction.rs`, 148 lines) | **P3** |
| `i18n::fl!` (~150 sites) | **yes** | app only; `spaceport-i18n` has `Catalog::text`/`text_with` but no current-locale lookup and no ID check | **P1** (the §7 decision) |
| `Entity<SearchInput>` (8 sites) | **yes** | app only, 668 lines, itself reading `caret`, `text_editing` (186), `input::ViewInputHandler`, `fonts::StyledFont`, `actions` | **P2** |
| `SpaceportWindow` (15 sites) | **yes** | app | **P4** source snapshot |
| `InitialAppearance` + `light_appearance` (15 sites) | **yes** | app (`app.rs:41-83`), carries `Keymap` | **P5** appearance service |
| `log_window`, `actions::rebind_keys`, `menus`, `menu::Page` | **yes** (services) | app | **P5** |
| `titlebar` (277), `Caption` (45), `taskbar::mark`, `panel_resize` (145) | **yes** (chrome) | app | **P6** |
| `toggles::switch`, `accessibility::disabled` | **yes** | app, 144 lines, 10+ consumers | **P3** |
| `font_picker::FontTarget`, `menu::colors::accent`, `core_feature::restart_required` | mild | app, 9 + 2 + 35 lines | route into the crate as values |
| `agent_skill` (6 sites) | **yes** | app, 317 lines | keep app-side: pass `Choice` in, take a `Change` out |
| `sidebar::paint::PAINTERS` | **yes** | app, 57 lines | keep app-side: the page passes painted elements in |
| `launch_at_login` | no | app, Windows only | stays app-side as a `#[cfg(windows)]` sub-view |
| `keymap::Keymap` | **yes** | app (`keymap.rs`) | **P5**: the crate never names it |
| `crate::Error` | mild | app root | **P7**: the crate's own `Error` for the three file-shaped failures it raises |

### 2.2 The new prerequisites, sized

| # | Prerequisite | Lines | Kind | Other consumers |
| --- | --- | --- | --- | --- |
| **P1** | message lookup below the app: move `i18n::{text, message, english}` and the process-wide current `Catalog` (`i18n.rs:33-74`, `i18n/current.rs` 83 lines) into `spaceport-i18n`, keeping `fl!`'s compile-time ID check in the app's `build.rs` (133 lines, generates `messages.rs` from `locales/`) and `i18n/check.rs` (82) | ~170 + 215 | crate change + a decision | **chat** (174 messages), every feature that draws |
| **P2** | text field: `search_input.rs` (668) + `caret` + `text_editing.rs` (186) + the `ViewInputHandler` half of `input.rs` | ~1,100 | new GPUI crate | chat composer, palette, find, `dialog_input` — `SearchInput` and the shared model landed in `crates/spaceport-textfield` (wx82); `DialogInput` is still app-side |
| **P3** | widget layer: `direction.rs` (148), `toggles.rs` (98), `accessibility.rs` (46) | 292 | new GPUI crate (`spaceport-widgets`) | chat (`view/widgets/components.rs:115,370`), 5 menus, titlebar, sidebar, `close_modal` |
| **P4** | source snapshot: a value `SourceWindow { config, keymap, theme, shared, target, integrations_busy, theme_page_open }` plus a `Subscription` the app installs on the source window | ~60 | the app's side of the seam | — |
| **P5** | appearance service: what the crate calls to say "publish this config+theme", "rebind keys", "install menus", "the log window follows" | ~80 | trait or closure struct | `menu/settings.rs`, the other writer of the same globals |
| **P6** | window chrome: `titlebar` (277), `caption` (45), `panel_resize` (145), `taskbar::mark` | ~470 | GPUI crate or prepared elements | every window |
| **P7** | the crate's own `Error` (3 variants) + `impl Localize` | ~30 | the crate | — |

Only P1, P2 and P3 are *shared* prerequisites — each has consumers far outside
this module — which is what makes them worth building. P4-P7 are this
extraction's own cost and exist only to cross the app/crate line.

### 2.3 The two knots, and how they untie

| Knot | Where | Smallest untie |
| --- | --- | --- |
| `Loaded.keymap: Keymap` | `persistence.rs:17-23`; `Loaded` is what every loader returns and it holds a `Keymap` | `spaceport-settings` already keeps the raw `keybinding_overrides` and `pane_keys`, and `docs/notes-lower-shared-types-design.md` §7 chose Option A (keymap derived app-side). The crate's loader returns `LoadedSettings { config, theme, shared, error }`; the app's `Loaded` adds `keymap`. 1 type, 7 constructors, `persistence.rs:238-330` |
| `fl!` inside renderers | §1.4 | §7's decision. There is no third option: a crate cannot expand a `macro_rules!` that names `$crate::i18n` |

## 3. The staged plan

Ordered so each stage is a PR on its own and each stage's files are ones its
predecessors did not touch. Every stage starts the same way:

> **Survey first**: `git log --oneline -1`; re-read the files named in the stage;
> re-count with §0's commands; confirm the boundary you are about to rely on has
> no `gpui` in it (`rg gpui <crate>/src`). Do not begin from this note's line
> numbers.

### Stage 1 — the draft and save-queue policy moves into `spaceport-settings`

**What moves** (~1,240 production lines, gpui-free):

| Item | From | Lines |
| --- | --- | --- |
| `SidebarKey`, `SidebarTouched`, `SidebarIntent`, `diff_preferences`, `touched_preferences` | `sidebar.rs:15-166,209-226` | ~180 |
| `Lifecycle`, `Save`, `Goal`, `Event`, `Effect`, `Kind`, `QUEUE_LIMIT = 64` | `persistence/lifecycle.rs:8-263` | 263 |
| `Request`, `Operation`, `SharedWrite`, `Reconciled`, `reconcile` | `persistence/operation.rs:6-95` | 96 |
| `Section`, `Group`, `ALL`, `nav_item`, `group`, `icon`, `slug`, `lead`, `selector`, `message` | `section.rs:1-166` | 166 |
| `Entry`, `Availability`, `CATALOG` (75 rows), `search` | `catalog.rs:16-215` | 215 |
| fenced drafts: `theme_pending`, `apply_theme_draft`, `theme_load_revision`, `apply_loaded_theme`, `filter_names`; `layout_load_revision`, `apply_loaded_layout`; `sidebar_load_revision`, `apply_loaded_sidebar` | `themes.rs:53-105,256`; `layouts.rs:17-45`; `sidebar.rs:186-206` | ~230 |
| `NumberField::{selector, range}`, `DropdownField::selector`, the position/on-off tables | `inputs.rs:22-90`, `pages.rs:27-167` | ~150 |

Three inversions, each one parameter:

1. `cx.try_global::<SidebarDraft>()`, `::<ThemeDraft>()`, `::<LayoutDraft>()` and
   `::<InitialAppearance>()` become one `Drafts` value the app owns and passes:
   the crate returns *what to do* (`apply_loaded_sidebar(config, &drafts,
   revision)`), the app keeps the GPUI globals. Same shape as `spaceport-sound`'s
   `Live<'a>` (`docs/gpui-split-plan.md`, "Three couplings had to invert").
2. `catalog::search` calls `i18n::message` **and** `i18n::english`
   (`catalog.rs:197-215`). It takes a lookup for both, so the app passes its two
   and the crate stays message-free.
3. `Entry::title()`/`description()` become id accessors on the crate's side;
   `ui::row` — stage 2's — resolves them.

**Exit criterion**: `cargo test -p spaceport-settings` runs the moved pure tests
(`section_order`, `catalog_rows`' data half, `search`, `save_intents`'
saturation test, `sidebar_drafts`' key tests); `cargo check -p spaceport-gpui
--tests` is clean; every name still resolves at its old path through
`settings_window.rs`'s re-export list, so no call site outside `settings_window/`
changes; `just check-file-size`, `just check-sorted`, `just check-i18n` pass.

### Stage 2 — the row kit gets a GPUI crate

**New crate** `spaceport-settings-window` (GPUI, like `spaceport-sidebar`:
`gpui`, `spaceport-settings`, `spaceport-widgets`, `thiserror`, `test-support`).

**What moves** (~1,100 production lines):

| Item | From | Lines |
| --- | --- | --- |
| `Source`, `tag`, `section`, `Row`, `row`, `data_row`, `toggle`, `button`, `segmented`, `segmented_elements`, `ROW_HEIGHT` | `ui.rs:26-384` | 384 |
| `Look`, `Number`, `NumberChange`, `Unit`, `number`, `stepper`, `parse_number`, `slider_value`, `stepped_value`, `NumberEdit`, the preview thread-local | `ui/number.rs` | 304 |
| `Dropdown`, `dropdown`, `popover_list`, `follow_herdr_value` | `ui/popover.rs` | 216 |
| `HostColorChange`, `parse_hex_input`, `swatch_row` | `ui/swatches.rs` | 193 |
| `Selector`, `Tracked`, `track_bounds`, `RowBounds`, `SCROLL_MARGIN` | `search.rs:13-31,130-193` | ~90 |
| `control_card`, `control_switch` | `controls/forms.rs` | 67 |

**Preconditions**: **P3**. This is a real prerequisite, not a preference:
`ui.rs:233` calls `crate::toggles::switch(theme, 20., checked)` and `ui.rs:243`
calls `crate::accessibility::disabled`, and `Logical` is on every element
builder. All three are already used by chat (`chat/view/widgets/components.rs:115,370`),
five menus and the titlebar, so that crate is wanted without this extraction. If
P3 is deferred, this stage is deferred — do not copy the switch into the
settings crate.

`scrollbar.rs` needs `spaceport_browser::Thumb` (already a crate) but stays for
stage 4 with the window, because it borrows `ScrollHandle` from the entity.

**Exit criterion**: `cargo test -p spaceport-settings-window` runs the moved unit
tests (`ui/tests.rs` 147, `ui/number/tests.rs` 319, `ui/popover/tests.rs`,
`ui/swatches/tests.rs`) with no `SettingsWindow` in sight; the app's
`settings_window::ui::` path is a `pub(crate) use` of the crate; the native
window draws the same rows (before/after screenshots, light and dark, plus a
narrow window).

### Stage 3 — the pages read a prepared page instead of the window

The settings analogue of browser's `BrowserHosts`, and the stage that makes
stages 5-6 mechanical. It ships **with nothing moved out of the app** — a change
of shape only, so it is reviewable on its own.

- Introduce `PageState` (the values one page draws: the `Config` fields it reads,
  `Option<&herdr_settings::Settings>`, the theme, and the flags `availability`
  needs) and `PageText` (the resolved words for one row: title, description,
  reason). Each `render_x` becomes `fn render_x(&self, page: &PageState, cx:
  &mut Context<Self>) -> Div`, and the `self.` reads inside it become `page.`
  ones, one field at a time.
- `available(availability, cx)` (`search.rs:48`) becomes
  `PageState::available(availability)`; `shared_gate` (`pages.rs:45`) becomes
  `PageState::shared_gate`.
- `preference_toggle` (`controls/preferences.rs:98`) and `control_choice`
  (`controls.rs:246`) stay entity methods — they own the save path and build
  `cx.listener` closures — but take the values they need.

**Exit criterion**: no `render_*` reads `self.config`, `self.shared`,
`self.theme` or `self.keymap` directly (`rg -n 'self\.(config|shared|theme)\.'
crates/spaceport-gpui/src/settings_window/pages` returns only handlers, not
renderers); every test in `tests/pages_behaviour.rs` passes unchanged.

### Stage 4 — navigation, footer and inputs join the crate

`navigation.rs` (245), `inputs.rs` (398), `scrollbar.rs` (140) and `pages.rs`'s
shared helpers move into `spaceport-settings-window`. These are the only files
that drive the window's *own* input — the section list's drag, the open dropdown,
the number field, the body scrollbar — and they are exactly the code that stays
with the view under any option, so moving them is free once stage 2 exists.
Preconditions: **P4** for `source`, **P6** for `panel_resize::{PanelDrag,
handle}`.

**Exit criterion**: the crate holds `navigation`, `footer`, `inputs`, `scrollbar`
and their tests (`tests/navigation_resize.rs`, `navigation_scroll.rs`,
`navigation_groups.rs`, `scrollbars.rs`, and `sidebar_drafts.rs`' input half); the
app's `SettingsWindow` is down to fields, `Render`, `open`, drafts and the pages.

### Stage 5 — the theme browser

`themes.rs` (885) + `themes/{appearance 247, grid 258, system 222}` = 1,612
lines, the largest single block and the one with the most independent state
(`ThemeBrowser` at `themes.rs:122`; `Grid` at `themes/grid.rs:11` with
`CACHE_LIMIT = 64`, `BATCH = 12`). Preconditions: **P2** (it owns an
`Entity<SearchInput>`, `themes.rs:123,153`) and **P5** (it publishes previews
through `InitialAppearance` and `log_window`).

**Exit criterion**: the crate's theme browser is driven by its own
`#[gpui::test]`s (`themes/tests.rs` 489, `themes/tests/{scrolling,system}.rs`,
`themes/grid/tests.rs` 152); the preview-fencing rules that today live in
`tests/theme_drafts.rs` and `tests/mixed_follow.rs` move rather than copy.

### Stage 6 — the pages, once the words are below the app

The eleven page modules (2,369 production lines) plus `controls/*` (1,323) move
last. This is the stage **P1** gates and where §7's decision is cashed in. If P1
is answered "no", this stage does not happen and the extraction ends after stage
5 — which is a legitimate stopping point and the one §4's second number assumes.

**Exit criterion**: the app's `settings_window/*.rs` holds only `open`,
`key_bindings`, `SettingsWindowHandle`, the `Render` impl and the re-export list;
the app's remaining settings tests are the ones that drive a `SpaceportWindow`
fixture, and they become the crate's integration tests.

### Stage 7 — record it

`docs/gpui-split-plan.md`'s "What moved" row, the crate table in `AGENTS.md`,
the next-extraction table, and this note's status.

### What each stage unblocks

| Stage | Unblocks |
| --- | --- |
| 1 | nothing directly, but it is the only place the save-queue rules can be tested without GPUI, and `sidebar`'s draft work reads `SidebarIntent` |
| 2 | `chat`'s settings-like widgets, and the second consumer of the `ui::row` pattern |
| 3 | stages 4-6; nothing else |
| 4-6 | the three features' shared need for a Settings crate that is not the app |

## 4. What stays app-side, honestly

Two answers, because §7 changes it.

**If P1 lands (the words go below the app)** — the app keeps:

| Item | Lines | Why |
| --- | --- | --- |
| `open`, `open_with`, `SettingsWindowHandle` | 76 | creates the window, owns the retained-model rule, `WindowOptions`, `on_window_should_close` |
| `impl Render for SettingsWindow` | 110 | the root `div`, `key_context`, `on_action` routing, `titlebar::frame` |
| `key_bindings()` | 15 | registered by `actions.rs:138` |
| `launch_at_login` sub-view | 99 | `#[cfg(windows)]` platform code |
| `native.rs` + `native/` | 1,327 | `integration-test` probes; they drive the real window |
| the source-window and service glue (P4/P5 implementations) | ~140 | `SpaceportWindow` reads, appearance/menu services |
| **total** | **≈1,770** | of which 1,427 platform/test-only, so **≈340 ordinary production lines stay** |

That is *smaller* than browser's 1,395 — and that is the finding: unlike the tab
strip, this window's glue is thin because the entity owns itself. The cost is not
in what stays; it is in the seven prerequisites.

**If P1 does not land** — stages 5 and 6 are impossible, and the app also keeps
`pages/` (2,369), `controls/` (1,323), `themes/` (1,612) and their ~60
`#[gpui::test]` files: **≈5,300 ordinary production and ≈8,000 test lines**. The
extraction then stops after stage 4 with ~2,500 lines moved, and the honest
statement is that the Settings window is a GPUI view made of 300 sentences, which
`docs/i18n.md:126`'s rule keeps in the app.

## 5. Risk ranking, and stage 1's task list

### 5.1 Risks, worst first

1. **The message decision (P1) is not a refactor.** Moving the lookup into
   `spaceport-i18n` moves *ownership of the sentence*, against
   `docs/i18n.md:126` ("crates below the app have no messages") and against
   `scripts/check-i18n.py`'s rule that every crate's text is checked. Either that
   rule is amended — `spaceport-i18n` becomes the one crate allowed to hold the
   lookup, everything else still returns values — or the pages never move.
   Nothing after stage 4 starts before it is answered.
2. **`fl!`'s compile-time check.** `fl!` expands to `$crate::i18n::check(id, …)`
   plus `$crate::i18n::text` (`i18n.rs:33-44`), and `check` reads a table
   `build.rs` generates from `locales/`. A crate calling a lookup loses the
   ID/argument check unless the table moves with it. Verify a typo'd ID still
   fails the build before accepting stage 6.
3. **Two writers of one global.** `menu/settings.rs` and this window both write
   `InitialAppearance` and both read the `*_revision`/`apply_loaded_*` seam
   (§1.5). Stage 1 moves that seam; if `Drafts` is not the *single* writer, the
   fencing `tests/mixed_follow.rs` and `tests/theme_drafts.rs` pin will drift.
   Count both writers before stage 1 and keep one constructor.
4. **`Loaded` is returned by every loader.** `persistence.rs:238-330` builds it in
   six places plus `tests.rs:53`'s fixture, and `menu/settings.rs:177-252` builds
   its own. Removing `keymap` is its own commit, before any file moves.
5. **`SearchInput`'s fan-in.** P2 is ~1,100 lines with five other consumers; if
   attempted inside this program it will dominate it. None of stages 1-4 needs
   it; stage 5 does. Treat P2 as its own wave.
6. **Element identity and debug selectors.** The tests select by
   `debug_selector` (`ui.rs:157-260`, `search.rs:18`) and by
   `Section::selector()` (`section.rs:144`). A move that renames an id passes
   visual review and fails `tests/catalog_rows.rs` and `tests/sidebar_drafts.rs`.
   Diff the selector strings, not the code.
7. **Blocking I/O in a library.** `persistence.rs` runs loads and saves on
   `cx.background_executor()` and `herdr_settings::Settings::save` blocks
   (`spaceport-settings`' own warning). The crate must carry that warning in its
   doc comment, or the rule lives only in this file.
8. **A dependency's `cfg(test)` is false inside the app's test build.** The
   number field keeps preview state behind it (`ui/number.rs:17-23`). The
   `test-support` feature (`crates/spaceport-sidebar/Cargo.toml:13-17`) is the
   mechanism; plan for it in stage 2 rather than discovering it.

### 5.2 Stage 1's exact task list

Stage 1 ships as **three commits**, the first two of which move no file:

**Commit A — split `Loaded`.**
1. Add `spaceport_settings::window::LoadedSettings { config, theme, shared,
   error }` (moved from `persistence.rs:17-23` minus `keymap`).
2. Keep the app's `Loaded` and give it `keymap` from the app's keymap
   derivation. **Survey first**: `rg -n 'fn keymap' crates/spaceport-settings/src`
   — if `Config::keymap()` does not exist yet (`docs/notes-lower-shared-types-design.md`
   §7 Option A), keep `Loaded.keymap` and hand the *crate* `LoadedSettings`.
3. Update the six `Loaded` constructors, `tests.rs:53`'s `fixture_load`, and
   `controls/fonts/tests.rs:122`.

**Commit B — move the pure model into `spaceport-settings`.**
4. Move `sidebar.rs`'s `SidebarKey`/`SidebarTouched`/`SidebarIntent` +
   `diff_preferences`/`touched_preferences` to
   `crates/spaceport-settings/src/draft.rs`; re-export from
   `settings_window/sidebar.rs` so no call site changes.
5. Move `persistence/lifecycle.rs` whole (263) to
   `crates/spaceport-settings/src/window/lifecycle.rs`. `Kind` keeps
   `LayoutMode`/`SidebarIntent`/`Theme(u64)`, and `AnyWindowHandle` is the only
   `gpui` type left — accept it (precedent: `spaceport-browser` carries
   `WindowId`) or hold a `u64` window id the app maps back. Decide in the survey
   and write the reason in the commit body.
6. Move `persistence/operation.rs` (96) beside it.
7. Move `section.rs` (166) and `catalog.rs` (215); `Entry::title`/`description`
   become id accessors and `catalog::search(query, look)` takes the lookup.
8. Move `inputs::NumberField::{selector, range}`, `DropdownField::selector`,
   `pages::{position_names, position_name, on_off, on_off_choices,
   override_label}` and `themes::filter_names`.
9. Move the fenced-draft helpers (`theme_pending`, `apply_theme_draft`,
   `theme_load_revision`, `apply_loaded_theme`, `layout_*`, `sidebar_*`) taking a
   `&Drafts` parameter; keep the GPUI globals and their writers in
   `settings_window/`, with one `Drafts` constructor in the app.

**Commit C — wire, test, record.**
10. Confirm `cargo tree -p spaceport-settings` shows no `gpui`.
11. Move every pure test with its code; keep every `#[gpui::test]` in the app.
12. `docs/gpui-split-plan.md`: add the row and note that this crate is *not* the
    feature crate.

**Restated as a check**: `cargo check -p spaceport-settings -p spaceport-gpui
--tests` clean; `cargo test -p spaceport-settings` green and larger than before;
`rg gpui crates/spaceport-settings/src` returns only the `AnyWindowHandle`
decision and the pre-existing `font_weight`/`fonts` conversion sites;
`just check-file-size`, `just check-sorted`, `just check-i18n` green; no file
outside `settings_window/`, `crates/spaceport-settings/` and the docs changed.

## 6. The decision the owner must make

**Where does the localised text live?** This gates stages 5-7 and nothing else in
this note changes shape either way.

**Option A (recommended) — the lookup moves into `spaceport-i18n`.** Move
`i18n::{text, message, english}` (`i18n.rs:53-89`), the process-wide current
catalog (`i18n/current.rs`, 83 lines), and the generated ID table with its
`check` (`i18n/check.rs` 82, `build.rs` 133) so that `spaceport-i18n` can answer
`text(id, args)` in the shown locale for any crate. `fl!` stays in the app as a
thin wrapper that calls the crate's `check` and `text`, so no call site changes
and the compile-time ID check is preserved. Cost: ~215 app lines move down, the
`locales/` embedding moves into a build script `spaceport-i18n` owns, and
`docs/i18n.md:126` gains one paragraph ("`spaceport-i18n` holds the lookup; every
other crate below the app still returns values and the app writes its words").
Benefit: the 24 files and ~150 sites of this window's text stop being a blocker,
and `chat`'s 174 messages stop being one too. That is the single largest
unblock available in the repo today.

**Option B — the lookup stays app-side and the pages stay with it.** No crate
below the app ever draws words; a UI crate takes `&dyn Fn(&str) -> String` per
builder or a `Text` handle. Cost: every one of the ~150 sites gains a parameter
threaded through the row kit and every page, which is a bigger mechanical change
than A's 215-line move, and it produces a settings crate whose pages cannot move
anyway. Benefit: `docs/i18n.md` is untouched.

My recommendation is A, on the grounds that the alternative charges the
extraction ~150 parameter sites to preserve a rule the extraction then makes
moot — and that `chat`, which is 8× the text, needs the same answer.

Two smaller decisions ride along and are safe to default:

- **`AnyWindowHandle` in the crate's `Lifecycle`** (§5.2 step 5): accept the
  `gpui` dependency, as `spaceport-browser` did for `WindowId`, or pass a `u64`.
  Default: accept it — the crate is `spaceport-settings` and `spaceport-settings`
  must stay `gpui`-free, so if `Lifecycle` needs a window handle it belongs in
  the GPUI crate of stage 2 instead. Survey this at step 5 before moving the file.
- **`settings_panel.rs`**: leave it where it is. It is window state loading, not a
  panel (§0), and its four tests drive `SpaceportWindow`.

## What this does not do

No production code changed and no build run, so every number here is a file
length, not a `wc -l` of a moved diff. No decision made on §6, which gates stages
5-7. No `spaceport-widgets` or text-field crate designed beyond its boundary and
size. `settings_panel.rs` is out of scope. No statement about whether the three
large features get faster: `chat` needs §6's answer and P2 regardless, and
`sidebar` needs neither.
