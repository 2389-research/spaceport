# Lowering the shared value types out of `crates/sheprd-gpui`

Design for the step [`docs/gpui-split-plan.md`](gpui-split-plan.md) puts in front of
`browser`: moving the settings, snapshot, and painter value types browser, sidebar
and chat read out of the app and below the features. It is a design only: no
production code changed.

**Verdict.** The settings module can move almost whole, and the count is lopsided
in the right direction: `crates/sheprd-gpui/src/config/` is 13,983 lines (5,792
production, 8,191 tests in 94 files) and `src/config.rs` adds another 700, yet
`gpui` appears in just **seven of those 38 files, in seven distinct places** — a
font weight, a font conversion, a padding helper, a colour parser, a settings
watcher, an SSH hook, and the keymap. Only the weight and the keymap are
structural; the rest are a conversion or a reload hook that stays in the app. A
new `sheprd-settings` crate with `serde`/`toml`/`thiserror` and no `gpui` holds
the remaining 14,683 lines.

`state.rs` needs no crate of its own to unblock anything: `LiveState` is already
gpui-free, and the types browser/sidebar/chat read out of it — `snapshot`,
`surface`, `status` — are `sheprd-protocol` values (`ClientShellSnapshot`,
`PaneSurfaceFrame`) plus one six-variant enum. What is worth lowering there is the
coherence/fencing rule set, and that can wait for the first feature that needs it.

`terminal_painter.rs` should be **split into files, not into a crate**. Its 983
lines are one of eight sub-responsibilities in a 2,057-line module, two of which
(168 lines) are gpui-free; a crate for those two moves a hop without removing a
dependency, because the only consumers are GPUI painters. Chat's dependency on
the module — which the split plan counts against chat — is two calls to
`fit_bgra`/`texture`.

One decision has to be made before the settings crate can take the `Config`
aggregate: `Config` holds a parsed `Keymap`, and `Keymap` holds
`gpui::Keystroke`. Section 7 states the smallest form of it.

## 1. A dependency census

Measured on this worktree at `91bda83a7` with:

```sh
find crates/sheprd-gpui/src/config* -name '*.rs' | xargs wc -l        # 15,028
find crates/sheprd-gpui/src/config  -name '*.rs' \
  | grep -v '/tests' | grep -v '/tests.rs$' | xargs wc -l             # 5,792
rg -c gpui crates/sheprd-gpui/src/config/*.rs crates/sheprd-gpui/src/config.rs
rg -o 'config::[A-Z][A-Za-z0-9_]+' crates/sheprd-gpui/src -N --no-filename \
  | sort | uniq -c | sort -rn
```

The brief's 13,983 lines is exactly the `src/config/` directory (5,792 + 8,191);
`src/config.rs`'s 700 lines sit beside it, and the 15,028 that `src/config*`
totals also picks up `src/config_diagnostic.rs` and its tests (345 lines). The
brief's "271 files" is a narrower query than mine: **307** files carry a
`use … config:: …` statement (180 of them outside tests) and **316** name a
`config::` path.

### 1.1 `config` — 14,683 lines, seven `gpui` sites

| File | Lines | `gpui` use |
| --- | --- | --- |
| `config.rs` | 700 | `gpui::FontWeight::NORMAL` in `Default` (`config.rs:247`) |
| `config/fonts.rs` | 510 | `Font`, `FontFallbacks`, `FontWeight` (`:9`); `FontConfig::font() -> Font` (`:303-312`) |
| `config/font_weight.rs` | 98 | `FontWeight` throughout: the CSS name table (`:9-24`), `parse`, `bold`, `describe` (`:62`) |
| `config/watch.rs` | 207 | `follow(cx: &mut gpui::App, …) -> gpui::Task<()>` (`:74-78`) |
| `config/ssh_hosts.rs` | 53 | `install(&SshConfig, cx: &mut gpui::App)` and `apply(&mut gpui::App, …)` (`:46`, `:51`) |
| `config/padding.rs` | 147 | `Padding::pad<E: Styled>` (`:141-147`), `Styled`/`px` (`:4`) |
| `config/sidebar.rs` | 457 | `gpui::Rgba::try_from(value)` in `parse_color` (`:340`) |

The other **30** production files in `src/config/` contain no `gpui` at all
(`config/error.rs` matches a `gpui` grep only through the filename
`config-gpui.toml`, `:65` and `:85`). What each of the seven sites actually is:

1. **`FontConfig.weight` (`config/fonts.rs:287`) and `MissingWeight`.** The only
   place a `gpui` type reaches stored settings. `gpui::FontWeight` is
   `pub struct FontWeight(pub f32)`
   (`gpui-pre-0.3.10/src/text_system.rs:1161`) inside the framework crate, so it
   cannot be depended on alone. Seven code sites read or write it
   (`config/fonts.rs:123,305,415-421`, `config.rs:247,443`,
   `terminal_painter/glyphs.rs:48`).
2. **`FontConfig::font()` (`config/fonts.rs:303`)** — a UI conversion (a family,
   weight and fallback cascade as a `gpui::Font`). Font *resolution* around it is
   already gpui-free: `resolve_fonts(installed: impl FnOnce() -> I)`
   (`config/fonts.rs:402`) takes the installed-family list from the caller
   (`preferences.rs:39`, `menu/settings.rs:150`).
3. **`Padding::pad`** — one generic over `Styled`, used to apply
   `pl/pr/pt/pb`.
4. **`sidebar::parse_color`** — parses `#rgb`/`#rrggbb` by way of
   `Rgba::try_from`; `config/sidebar_style.rs:130` already has `parse_hex`
   returning the same `u32`.
5. **`config/watch.rs` and `config/ssh_hosts.rs`** — app-side service reload:
   a `gpui::App` spawn that fingerprints the two files off the UI thread and
   hands one selected field to an `apply` function. Glue, not settings.
6. **The `Keymap`.** Not visible in `config/` at all, which is why a `grep gpui`
   census misses it: `Config.keybindings: Keymap` (`config.rs:161`), built in
   `parse_layers` by `Keymap::with_overrides(&settings.keybindings,
   &settings.pane_keys, &base.keys)` (`config.rs:630-632`), and `Keymap` holds
   `Vec<Keystroke>` and `Vec<(Keystroke, PaneKey)>`
   (`keymap.rs:133-145`) behind `use gpui::{KeybindingKeystroke, Keystroke,
   Modifiers}` (`keymap.rs:28`). The *settings shape* of the same table is
   gpui-free: `Binding` (`keymap.rs:90-93`, a `String` or `Vec<String>`) and
   `type PaneKeys = BTreeMap<String, String>` (`keymap.rs:47`), and `Config`
   already keeps both raw (`keybinding_overrides`, `pane_keys`, `config.rs:160,176`).

What makes config *feel* un-splittable is not gpui, it is two knots that a move
has to solve with small, explicit edits:

| Knot | Where | What breaks | Smallest fix |
| --- | --- | --- | --- |
| `config` ⇄ `herdr_settings` | `config/theme.rs:235`, `config/notifications.rs:231-234`, `config/preferences.rs:176,288`, `config/backdrop.rs:164`, `config/keybindings.rs:29` use `herdr_settings::{Settings, ToastDelivery, Error}`; `herdr_settings.rs:1-26` and `herdr_settings/palette.rs` use `config::{ClipboardToastPosition, Theme, view_settings_path}` | neither can move alone | both move into one crate, `sheprd-settings`; `herdr_settings` also names 20 `crate::lenient::*` paths in `#[serde(deserialize_with = "…")]` strings (`herdr_settings.rs:228,286-…`) which become `sheprd_lenient::*` |
| `config` ⇄ app features | `config/chat.rs:40,196-232` (`chat::PROGRAM_ENV`, returns `chat::Error`), `config/backdrop.rs:6` (`backdrop::Error`), `config/devices.rs:60` (`endpoint::saved_profile_id`), `config/tab_layout.rs:37`+`config/layout.rs:143`+`config/chat/launcher.rs:32` (`i18n::message`), `error.rs:227` (`Error::Settings`) | a crate below cannot name a feature above | see §4.3 |

`herdr_settings` is the second half of the same body of code and has to travel
with it: 2,998 lines (≈1,590 production, ≈1,410 tests) whose only `gpui` use is
`RightClickPassthrough(Option<Modifiers>)` (`herdr_settings.rs:26,180-215`).

The type most importers name is `Theme` — 77 `config::Theme` references, more
than twice the next — and it is pure data: `background: u32`, `foreground: u32`,
`palette: [u32; 256]` (`config/theme.rs:322-342`). Painters convert with
`gpui::rgba((theme.primary() << 8) | alpha)` themselves
(`terminal_painter.rs:55-64`).

### 1.2 `state.rs` — `LiveState` and the snapshot types

`src/state.rs` is 798 lines with **zero** `gpui` references, as are its three
submodules:

| Piece | Lines | `gpui` | Depends on |
| --- | --- | --- | --- |
| `state.rs` (`LiveState`, `ConnectionStatus`, `apply(ClientEvent)`, sanitizers, `coherent`) | 798 | 0 | `sheprd_client` events and protocol, `crate::{i18n, notice, notifications, Error, links, sound::MAX_PENDING, OwnedNavigationTarget}` |
| `state/activation.rs` (`Fence`, `Transition`, 5 s `TIMEOUT`) | 317 | 0 | `crate::OwnedNavigationTarget`, protocol |
| `state/projection.rs` (coherent pair, `latest_snapshot`) | 199 | 0 | protocol |
| `state/agent_recency.rs` | 56 | 0 | protocol |

`LiveState` is 99 lines of fields (`state.rs:61-159`): the newest metadata and
surface as `Arc`s, the supports-* capability flags, and four app mailboxes (sound
events, clipboard writes, bells, notices/toasts) plus two dialog answer slots.
`AgentRecency` aside, it is a message-shaped value: `ClientEvent::apply`
(`state.rs:501`) is a match, and `only_surface_changed` (`state.rs:288`) is a
field-by-field repaint predicate. Nothing in it is an `Entity`.

The "snapshot types" it is built on are already below the app:
`Arc<ClientShellSnapshot>` and `Arc<PaneSurfaceFrame>` are
`sheprd_client::protocol` types, re-exported from `sheprd-protocol`'s wire types
(`crates/sheprd-protocol/src/wire.rs:312`). Nothing about the snapshot needs a
new home.

What the features actually read out of `LiveState` today, in production code:

| Feature | Read | Where |
| --- | --- | --- |
| `browser` | nothing by name — it reaches `self.live` through `impl SheprdWindow` | `state::activation::Transition` appears only in `browser/view_tests/tab_steps.rs:99` |
| `sidebar` | `&LiveState` per host, and `ConnectionStatus` | `sidebar/order.rs:6,17`, `sidebar/agents.rs:127` |
| `chat` | `LiveState` for routing, and one answer slot | `chat/discovery.rs:4,16,29` (`local`, `route`), `chat/start.rs:360` (`take_answer(Answer::Script, …)`) |

So `ConnectionStatus` (`state.rs:19-51`) plus the two read methods are the entire
cross-feature surface; `LiveState` itself is only reachable today because every
feature is `impl SheprdWindow`. That is the fact that decides §2: a state crate is
not on the critical path.

### 1.3 `terminal_painter.rs` — 983 lines, eight sub-responsibilities

It is one line under the file ceiling because the `impl TerminalPainter` body was
never split, not because the module is one thing:

| Sub-responsibility | Lines | `gpui` |
| --- | --- | --- |
| module wiring: 6 `mod`/`use` and 15 re-exports (`:1-33`) | 33 | — |
| `Tint`, `Highlight`, the tint alphas (`:32-73`) | 42 | `Rgba` in `Tint::color` only |
| `PaintTiming`/`PaintDiagnostics` (`:75-122`) | 48 | no |
| `TerminalPainter` fields, `Default` (`:125-165`) | 41 | `Font`, `Bounds`, `Pixels` |
| `link_bounds` (`:167-192`) | 26 | no (`SurfaceRect` out) |
| `paint_glyphs` (`:194-225`) | 32 | `ShapedLine`, `Window`, `paint_glyph` |
| `background_spans` (`:227-275`) | 49 | no (`u32` colours + `Theme`) |
| `composition_origin`, `byte_index` (`:277-291`) | 15 | `Point<Pixels>` |
| appearance/cache/metrics (`:294-391`: `set_appearance`, `reset_cache`, `verify_native_cache`, `shape`, `metrics`, `configure`, `cell_width`) | 98 | `Window`, `App`, `ShapedLine` |
| `paint_link` (`:393-437`) | 45 | yes |
| `release_idle_images` (`:438-446`) | 9 | `Window` |
| `paint_frame` (`:447-791`) | 345 | yes |
| `paint_images` (`:792-815`) | 24 | yes |
| `paint_text` (`:816-917`) | 102 | yes |
| `paint_composition`, `composition_bounds` (`:918-982`) | 65 | yes |

Its existing files:

| File | Lines | `gpui` | Responsibility |
| --- | --- | --- | --- |
| `terminal_painter/area.rs` | 89 | **0** | `Span`, `Layer`, `Part`, `cell_ranges`, `clip`, `covers`, `whole` — which cells, and which layer of them, a paint covers |
| `terminal_painter/decorations.rs` | 79 | **0** | underline/strikethrough offsets, dashed-link spans, all in `f32` |
| `terminal_painter/glyphs.rs` | 123 | yes | `GlyphCache` over `gpui::ShapedLine`, bold stepping |
| `terminal_painter/grid.rs` | 78 | yes | `Ground`, `grid_rect`, `grid_corners`, `background_extent` in `Bounds<Pixels>` |
| `terminal_painter/graphics.rs` | 294 | yes | `CellSeparator`, box/prompt graphics |
| `terminal_painter/images.rs` | 411 | partly | `ImageCache`/`PlacedImages` (gpui textures) plus `fit_bgra` (`:188`), which is `image::RgbaImage` only |

`fit_bgra`/`texture` (`images.rs:188,405`) are what chat wants: `chat/rich/math.rs:64`
and `chat/visual.rs:300` both do
`terminal_painter::texture(terminal_painter::fit_bgra(pixels))`. Two call sites,
one of which (`fit_bgra`) is gpui-free and one of which is three lines over
`RenderImage::new`. The module's public surface is otherwise small —
`rg -o 'terminal_painter::[A-Za-z_]+'` finds 15 distinct names across 20 files.

### 1.4 Which value types browser, sidebar and chat actually read

The split plan's line — browser "reads `endpoint` and `group_terminals` and
paints the tab strip from them" — is right about the shape and worth restating
exactly, because **neither module contains a value type browser could take**:

| Name | What it is | Lines | Verdict |
| --- | --- | --- | --- |
| `endpoint::Endpoint` (`endpoint.rs:153-194`) | `id`, `label`, `connection: ConnectionBridge`, `link`, `live: LiveState`, `history`, `toasts`, `config_diagnostic`, `sounds: sound::Policy`, retry bookkeeping | 20 fields | runtime holder; the *value* browser reads out of it is `id`, `label` and `connection.target` (`ConnectTarget`, already `sheprd_client`) |
| `group_terminals::GroupTerminals` (`group_terminals.rs:133-139`) | `primary: Option<GroupId>`, `parked: Vec<Parked>`, `retry` | 7 | runtime holder; the *values* the strip reads are `Option<String>` (the tab a group focuses, `:154`) and `bool` (connecting, `:181`), passed today as the `focused` closure at `browser/groups_view.rs:547` |
| `state::LiveState` | see §1.2 | 798 | runtime mailbox; the *values* are `snapshot`, `surface`, `status` |
| `browser::store::{Tab, TabId, Scope}` / `location::Location` | `Tab { id, scope, workspace_id, location, title, origin }` (`browser/store.rs:173-183`), `Location::{Devices, CoreTerminal { panes, observe }}` (`browser/location.rs:8-21`) | 20 + 19 | browser's own; they move *with* browser |
| `config::{Theme, TabLayoutMode, LayoutMode, …, corners}` | see §1.1 | — | **these** are the shared value types the plan means |
| `tab_layout::{Mark, Probe, TabContext, TabFace, TabLayout, layout_for}` | 1,624 lines of `Div`/`Stateful<Div>` shaping with `ClickHandler = dyn Fn(&ClickEvent, &mut Window, &mut App)` (`tab_layout.rs:151`) | 1,624 | a GPUI paint kit browser *and* the window's tab strip share (`window/tab_strip.rs:306` calls `browser_tab_entries`, `browser/view/render.rs:12` is the only production file importing it) — a blocker the plan does not list |

By reference count, what each feature wants from `config` is entirely the pure
part:

| Feature | `config::` names it uses | Notes |
| --- | --- | --- |
| `sidebar` | `LayoutMode` (39), `Config` (12), `Theme` (10), `StatusIcons` (10), `corners` (6), `Density` (3), `sidebar::{AgentLayout, SpaceLayout}` (3), `FontConfig` (2), `AgentToken` (2), `AgentsPlacement` (2), `TokenStyle`, `Style`, `SidebarStyle`, `SidebarOverrides`, `SidebarLayout`, `SelectMode`, `Layout`, `DeviceFooter`, `mix` | only `Config` itself has a `gpui` field |
| `chat` | `ChatLayoutMode` (20), `corners` (12), `Theme` (9), `mix` (7), `chat` (6), `FontConfig` (2), `FONT_SIZE_STEP` (1) | all pure |
| `browser` + window strip | `corners` (5), `Theme` (1), `home` (1) | all pure |

## 2. Target layering

### 2.1 `sheprd-settings` — one new crate, and the only one needed now

Named `sheprd-settings` rather than `sheprd-config` because it owns both files:
Spaceport's managed `config.toml`/`config.local.toml` and `view.toml`, and
Herdr's shared release-namespace `config.toml` that the GUI reads and edits
(`herdr_settings.rs:1-12`). "Settings" is also the word `sheprd-client` already
uses for the file layer (`sheprd_client::settings_files`, `config.rs:102-103`)
and `sheprd-usage` for its config sub-struct (`crates/sheprd-usage/src/settings.rs`).

What moves in — production lines:

| From | Lines | Contents |
| --- | --- | --- |
| `src/config.rs` | 700 | `Config` (58 fields, 50 of them public), `Settings`, `default()`, `path`/`local_path`/`load`/`load_startup`/`overrides_text`, `parse_layers`, `diagnostic`, `corners`, the re-export list |
| `src/config/` (37 files) | 5,792 | every typed table, `error.rs`, `files.rs` (write/migrate), `edit.rs` (one setting by dotted key), `preferences.rs` (Settings-window writer), `daemon.rs` (`view_settings_path`), `theme.rs` (687: built-ins, Ghostty files, Omarchy desktop theme), `watch.rs`'s loader half |
| `src/herdr_settings/` | 1,590 | Herdr's shared settings model, palette adaptation, guarded writes, remote pane history |
| `src/contrast.rs` (+`system_tests.rs`, `windows.rs`) | 384 | `Contrast`, `ink`/`ink_on_chrome` WCAG math — `theme.rs:523` needs it and chat/sidebar use it directly |
| `src/worktree_identity.rs` | 150 | only `Worktrees` (the `[worktrees]` value): the `Div` drawing stays |
| `src/palette.rs:44-69` | 26 | only `PaletteConfig` (+`validate`): the palette view stays |
| `src/keymap.rs:47,90-93` | 20 | only the raw `PaneKeys` type and `Binding` enum |

Dependencies, all already in `[workspace.dependencies]`: `config_loader`,
`serde`, `serde_ignored`, `serde_json`, `thiserror`, `toml`, `toml_edit`,
`tracing`, plus `sheprd-client` (`local_path`, `text::safe_text`,
`valid_profile_id`, `settings_files`), `sheprd-i18n` (`Language`), `sheprd-usage`
(`UsageConfig`, which `config.rs:227` already reaches through
`crate::usage::UsageConfig` — a re-export, so the edge already exists),
`sheprd-commands` (the `COMMANDS` name check at `config.rs:553`) and
`sheprd-lenient` (the wave-7 branch's crate).

What stays in `sheprd-gpui`, in `src/config.rs` as glue:

| Stays | Lines | Why |
| --- | --- | --- |
| `config/watch.rs` | ~150 | spawns on `gpui::App`, returns `gpui::Task` |
| `config/ssh_hosts.rs::install`/`apply` | 10 | takes `&mut gpui::App` |
| `pad(element, Padding)`, `font_for(&FontConfig) -> gpui::Font` | ~20 | free functions: an inherent `impl` on a crate type is not allowed outside it |
| `Label` trait + its four impls | ~30 | `label()` methods render `i18n::message` (`docs/i18n.md:126`) |
| `Config::keymap()` | ~10 | §7 |
| `weight(FontConfig) -> gpui::FontWeight` | ~5 | one `From` impl and the calls at `config/fonts.rs:305`, `terminal_painter/glyphs.rs:48` |

Its error type moves too: `crates/sheprd-settings/src/error.rs` keeps the 33
variants of `config::Error` (`config/error.rs`) and gains the file-shaped ones
config borrows from the app root — `Io`, `Toml`, `TomlEdit`, `Json`, `Path`
(with `at_path`, `error/path.rs:7`), `MissingHome`, `RelativeConfigRoot`,
`ConfigLoad`, `ConfigFile` — exactly as the approved sound design does for
`sheprd-sound` (`docs/notes-sound-extraction-design.md` §3). `herdr_settings::Error`
becomes a nested variant of it. The app keeps one
`#[error(transparent)] Settings(#[from] sheprd_settings::Error)`.

### 2.2 `sheprd-live` — not yet, and here is the trigger

A crate for `ConnectionStatus` plus the coherence rules would be honest but
premature: no feature reads anything from `LiveState` that is not either a
`sheprd-protocol` value or reachable through `impl SheprdWindow`, and AGENTS.md
forbids a crate without a concrete need. The rule set that *should* end up below
gpui is:

| Rule | Where | Test that pins it |
| --- | --- | --- |
| a snapshot and surface are the same boot and revision | `state.rs:792` (`coherent`) | `state/tests/surfaces.rs` (159) |
| one newer projection waits for its cells | `state/projection.rs`, `state.rs:83,92` | `state/tests/atomic_projection.rs` (288) |
| a frame changes only the terminal | `state.rs:288` (`only_surface_changed`) | `state/tests/surfaces.rs` |
| activation fences input until acknowledged | `state/activation.rs` | `state/tests/focus_acks.rs` (178), `activation_retry.rs` (121) |

Trigger to create `sheprd-live` (`ConnectionStatus`, `AgentRecency`,
`projection`, `coherent`, `sanitize_window_title`/`sanitize_daemon_version`, and
`Fence` if `OwnedNavigationTarget` — 36 gpui-free lines, `navigation.rs:1-36` —
travels with it): **whichever feature first needs to read live state from outside
`impl SheprdWindow`**, which is `browser` or `sidebar` per §5. Until then the
gating stays where it is, next to the mailbox it fences.

### 2.3 A painter crate — rejected, with the reason

`area.rs` + `decorations.rs` (168 lines, 0 `gpui`) plus `Highlight`/`Tint` data,
`link_bounds`, `background_spans` and `fit_bgra` would make a coherent
`sheprd-cells` crate. It is still the wrong move now: every caller of those names
(`find.rs`, `copy_mode.rs`, `window/regions.rs`, `scrollback.rs`,
`core_terminals/view.rs`) paints immediately afterwards with `gpui`, so a crate
would add an import hop without removing anything from the app's build graph, and
its types would have to be `pub` for a consumer that does not exist. The gpui-free
half should be *files* inside a split `terminal_painter` (§3) so the boundary is
drawn now; the crate can follow the day a non-GPUI consumer appears (a CLI that
renders a frame, or `sheprd-cells` becoming the shared home for the
`SurfaceRect`/`Span` algebra).

## 3. `terminal_painter`: the split

The module is 2,057 production lines in 7 files, with 2,231 test lines in 15. Two
shapes of split, and only one of them is worth doing:

**Shape A — keep painting, separate the pieces (recommended).** The main file
keeps `mod` wiring, the `TerminalPainter` fields and re-exports, and the rest
becomes eight files beside `area.rs`/`decorations.rs`:

| New file | Moves | Lines | `gpui` |
| --- | --- | --- | --- |
| `highlight.rs` | `Tint`, `Highlight`, the three alphas (`:32-73`), `Tint::color` | 42 | `Rgba` only |
| `diagnostics.rs` | `PaintTiming`, `PaintDiagnostics` (`:75-122`) | 48 | no |
| `composition.rs` | `paint_composition`, `composition_bounds`, `composition_origin`, `byte_index` (`:277-291`, `:918-982`) | 80 | yes |
| `links.rs` | `paint_link`, `link_bounds` (`:167-192`, `:393-437`) | 71 | yes, except `link_bounds` |
| `backgrounds.rs` | `background_spans` (`:227-275`), `cell_colors` helpers | 49 | no |
| `text.rs` | `paint_text`, `paint_glyphs` (`:194-225`, `:816-917`) | 134 | yes |
| `surface.rs` | `set_appearance`, `reset_cache`, `verify_native_cache`, `shape`, `metrics`, `configure`, `cell_width`, `release_idle_images` (`:294-391`, `:438-446`) | 107 | yes |
| `frame.rs` | `paint_frame` (`:447-791`) + `paint_images` (`:792-815`) | 369 | yes |

The main file lands under 150 lines. `paint_frame` is the one that keeps its
`#[allow(clippy::too_many_arguments)]`: it takes 12 arguments today and splitting
it further would need an argument struct, which is a behaviour-shaped change (the
`part`/`images` `Option`s drive three different paint paths) and not part of this
step.

**Shape B — crate.** Rejected in §2.3.

The gpui-free halves are therefore `area.rs` (89), `decorations.rs` (79),
`diagnostics.rs` (48), `backgrounds.rs` (49) and `link_bounds` (26) — 291 lines
that can be unit-tested as `sheprd-cells` later without touching a `Window`.

Chat's dependency disappears as a side effect: with `fit_bgra` in
`images.rs` (already) and `texture`'s body being `RenderImage::new(vec![Frame::new(pixels)])`
(`images.rs:405-407`), the two chat call sites either keep naming the app helper —
chat stays in the app until it is the last feature — or call `gpui` themselves
once chat is its own crate.

## 4. The migration order, as compilable commits

### 4.1 The façade that keeps 316 files still

`src/config.rs` stays as the module path `crate::config` and becomes a re-export
surface:

```rust
//! This app's reading of the settings crate's value types, and its own glue.
pub(crate) use sheprd_settings::{
    Accent, Backdrop, BellConfig, ChatLauncher, ChatLayoutMode, Config, Density, DeviceFooter,
    FontConfig, Layout, LayoutMode, Material, NotificationConfig, OptionKeys, SidebarStyle,
    SoundConfig, StatusIcons, TabLayoutMode, Theme, …, corners, parse_hex, view_settings_path,
};
```

Every `use crate::config::{LayoutMode, Theme}` keeps compiling, and
`use super::{Config, Layout}` inside the moved files keeps compiling too — they
are the same items, one crate down. `src/config.rs` also keeps the glue of §2.1,
so the module still names what the app itself does with settings. The crate table
in the plan's pattern (§6, item 3) is exactly this rule: a narrow root API,
re-exported where the old path was.

Flattening the path (`crate::config::Theme` → `sheprd_settings::Theme`) is then a
*separate, batched* commit per feature area — `sidebar`, `chat`, `window` — and
the façade loses its re-exports last, when nothing names `crate::config`.
Nothing forces that commit to happen at all; it is a readability choice, and
until then the churn is zero.

### 4.2 `the orphan rule` decides which methods move where

An `impl` block cannot be written for a type in another crate, so every gpui- or
i18n-shaped method on a moved type becomes an app-side free function or an
app-defined extension trait:

| Method | Today | After |
| --- | --- | --- |
| `FontConfig::font()` | `config/fonts.rs:303` | `config::font_for(&FontConfig) -> gpui::Font` |
| `Padding::pad<E: Styled>` | `config/padding.rs:141` | `config::pad(element, padding)` |
| `LayoutMode::label`, `ChatLayoutMode::label`, `ChatLauncher::label`, `TabLayoutMode::label` | `config/layout.rs:143`, `config/chat.rs:125`, `config/chat/launcher.rs:32`, `config/tab_layout.rs:37` | `pub(crate) trait Label { fn label(self) -> String; }` in `src/config.rs`, implemented for the four types: call sites (32 `.label()` uses outside tests) keep their spelling and need one added import |
| `Config::diagnostic()` | `config.rs:403` | the crate keeps the typed parts (`unknown_keys`, `missing_fonts`, `missing_weight`, `layout.legacy_sidebar_gap` are already fields) and the app writes the four `fl!` sentences (`config.rs:417-448`) in a `config::diagnostic(&Config) -> Option<String>` — messages are the app's (`docs/i18n.md:126`) |

### 4.3 The knots, resolved one at a time

| Knot | Resolution in this step |
| --- | --- |
| `chat::PROGRAM_ENV` / `chat::Error` in `config/chat.rs:40,196-232` | the reserved-env check (`RESERVED`, `check_env`, `:220-232`) moves up to `chat`'s launch path, where the env is written anyway; `ChatConfig::check()` keeps width/layout/mirror validation and returns `settings::Error` |
| `backdrop::Error` in `config/backdrop.rs:6` | `Backdrop::validate` returns `settings::Error::InvalidBackdrop`; the app's `Localize` covers it beside its `backdrop::Error` arms |
| `endpoint::saved_profile_id` in `config/devices.rs:60` | the `ssh:`/`wsl:` prefix parsing and `sheprd_client::valid_profile_id` call move to `sheprd-settings` (it is pure, `endpoint.rs:57-62`); `endpoint` re-exports it |
| `i18n::message` in four `label()`s | §4.2 |
| `crate::Error` in 7 config files (24 sites) | `settings::Error` with `Io`/`Toml`/`Path`/`at_path`; `Localize` arms move to `impl Localize for sheprd_settings::Error` (legal: `Localize` is the app's trait, `i18n/errors/config.rs:1-8`) |
| `herdr_settings` ⇄ `config` | one crate |
| `Config.keybindings: Keymap` | §7 |

### 4.4 The commits

| # | Subject | Contents | Compiles because |
| --- | --- | --- | --- |
| 1 | `refactor: Give the settings value types a crate below the app` | workspace `members` (`Cargo.toml:2`), `[workspace.dependencies]`, `sheprd-gpui` deps, `AGENTS.md` crate table; `git mv` the 30 gpui-free `config/` files (with `error.rs`) + `contrast.rs`; `Weight` in `font_weight.rs`; `src/config.rs` façade + the §4.2 glue; `parse_color` → `parse_hex`; `Label` trait | every moved name is re-exported from `crate::config`; only the six glue sites in `src/config.rs` changed |
| 2 | `refactor: Move the settings aggregate into sheprd-settings` | `Config`, `Settings`, `files`, `edit`, `preferences`, `daemon`, `notifications`, `devices`, `backdrop`, `worktrees`, `theme`, `omarchy`, `favorite_themes`, `keybindings` (raw), `herdr_settings`; `Config::keymap()` app-side; `PaddingConfig`/`PaletteConfig`/`Worktrees` leaf types | the crate is self-contained; the app calls `sheprd_settings::Config` through the façade |
| 3 | `refactor: Wire the settings crate's services in the app` | `watch::follow` keeps only `select` closures over the crate's `Config`; `ssh_hosts::install`; `Config::diagnostic` wording | app-only code |
| 4 | `docs: Record the settings crate` | `gpui-split-plan.md` "What moved" row + next-extraction table; a `docs/notes-settings-extraction.md` when it lands | docs |
| 5 | *(only when a feature needs it)* `refactor: Move connection status and snapshot fencing below the app` | `sheprd-live` per §2.2 | as §4.1 |
| 6 | `refactor: Split the terminal painter by responsibility` | §3 Shape A; no behaviour change | pure file moves and `pub(super)` widening |

Each commit runs `cargo check -p sheprd-settings -p sheprd-gpui`,
`cargo test -p sheprd-settings`, `cargo fmt --all -- --check`,
`just check-file-size`, `just check-sorted`, `just check-i18n`; `cargo metadata`
refreshes `Cargo.lock`, which the commit includes.

### 4.5 How this composes with the in-flight extractions

| Work | Interaction |
| --- | --- |
| `feat/wave7-extract` (`sheprd-reorder`, `sheprd-lenient`, `sheprd-truncate`, commits `dd07042b0`, `e0a49b0ce`, `0209553b6`) | `sheprd-settings` depends on `sheprd-lenient` for the 20 `deserialize_with` paths in `herdr_settings`, so wave 7 merges first. `sheprd-truncate` and `sheprd-reorder` are already off the app's config path |
| `feat/wave8-sound` (`crates/sheprd-sound`, `src/sound*`) | independent of the move *except* three names: `config::view_settings_path` and `config::SoundConfig` change owner, so the sound design's `app.rs:197` line reads `sheprd_settings::view_settings_path(...)`, and `SoundConfig::MAX_VOLUME` (`config/notifications.rs:71`) cannot be an inherent const on a moved type unless `sheprd-settings` defines the number itself. Two spellings of `MAX_VOLUME` (settings validating `[sound] volume`, `sheprd-sound` clamping playback) must not happen: either `sheprd-sound` depends on `sheprd-settings` for it — it already reads Herdr's settings file through `sheprd-lenient`, so the edge is small — or the bound stays owned by `sheprd-sound` and `SoundConfig` keeps an app-side accessor. Whichever lands second makes that call; the sound extraction is not blocked by either ordering |
| `feat/wave8-jargon` (error display text) | `i18n/errors/config.rs` is the file that becomes `impl Localize for sheprd_settings::Error`; the move is a `git mv` of that file plus a type change, so the two branches conflict only in that file and only if both edit the same arms |
| `sheprd-commands`, `sheprd-usage`, `sheprd-i18n`, `sheprd-client` | already below the app and already dependencies of the moved code (`config.rs:553`, `config.rs:227`, `config.rs:163`, `config.rs:102`) |

## 5. What this unblocks

After commits 1–4, the three features each read settings from a crate, and what
is left in front of them is measurable:

**`browser` first** (53 importers, 4,748 production lines, 10 messages). After the
settings crate it still needs:

| Need | Size | Note |
| --- | --- | --- |
| a shared tab-strip kit | `tab_layout` 1,624 lines, gpui `Div`/`Stateful<Div>` + a `ClickHandler` taking `&mut Window, &mut App` | browser *provides* entries to it (`window/tab_strip.rs:306` calls `browser_tab_entries`); either the kit gets a crate `sheprd-tab-strip` both depend on, or browser keeps a thin app-side renderer and only its store/layout logic moves |
| `core_terminals::panes::Panes` (for `Location::CoreTerminal`) | 463 lines, **0 gpui** | already crate-shaped |
| `state_file` (layouts store) | 127 lines, **0 gpui** | crate-shaped |
| a way to read the window | `self.endpoints`, `self.live`, `self.group_slots()`, `self.primary_group()`, `self.parked_focuses()` | a `&BrowserHosts<'_>`-style borrow struct passed in, as the sound design does with `Live<'a>` |
| `close_modal::CloseRun`, `motion`, `i18n` (10), `reorder`/`truncate` (crates) | small | |

**`sidebar` second** (263 importers — the most-imported module in the crate — 11,195
lines, 75 messages). After settings it still needs `icons` (19 refs), `menu` (55),
`endpoint` (45), `bind_keys` (24), `keymap` (5, for shortcut labels), `direction`
(19), `mark_layer`, `worktree_notes`, `device_tags`, `notification_mutes`,
`usage`, and the same kind of borrow struct for hosts — `(usize, &Endpoint,
&LiveState)` (`sidebar/order.rs:17`) has to become a value it receives.

**`chat` last** (20,440 lines, 174 messages, 12 importers). It needs browser
(11 refs), `notifications`, the `fit_bgra`/`texture` pair (§3), `actions`,
`controls`, `constants`, `motion`, `contrast` (9), `bundled`, and two app
operations that are not values at all: `take_answer(Answer::Script, …)`
(`chat/start.rs:360`) and the routing in `chat/discovery.rs:16,29` that takes a
`&LiveState`. Those are the reason chat needs `sheprd-live` (§2.2) and a window
callback, not just settings.

## 6. Risks

**Serde on live types.** `Config` and its tables are `Deserialize` and stay that
way — that is the crate's job. Nothing in `LiveState` gains serde: it holds
`Arc<ClientShellSnapshot>`, `Arc<Mutex<…>>`-free plain fields, and
`Result<Value, Arc<Error>>` answer slots (`state.rs:16`), none of which is wire
format. Two config behaviours must survive the move verbatim:
`serde_ignored` unknown-key reporting (`config.rs:549`) and the keybinding
name filter against the command catalog (`config.rs:553-560`), which is why
`sheprd-settings` depends on `sheprd-commands`.

**GPUI handles leaking into value types.** There are none today: no `Entity`,
`Context`, `App` or `Window` appears in `config/`, `herdr_settings/`,
`state*`, `navigation.rs`, `notice.rs`, `notifications.rs`, `links.rs`. The
holders that must stay app-side are `Endpoint` (`endpoint.rs:153`),
`ConnectionBridge` (`connection.rs:93`, which owns the `Arc<Mutex<LiveState>>`
inbox, the link inbox and the scrollback inbox) and `GroupTerminals`
(`group_terminals.rs:133`). The risk is a future convenience field — an
`Entity<SomeView>` cached next to a `Tab` — and the guard is that the moved
crate cannot name `gpui` at all, so it fails to compile rather than drifting.

**Snapshot and revision fencing.** AGENTS.md's protocol rules do not relax
because types moved: one snapshot and surface must never mix `boot_id`s or
`projection_revision`s (`coherent`, `state.rs:792`), a newer projection waits for
its cells (`pending_snapshot`, `pending_surface_images`, `state.rs:83,92`), and
`only_surface_changed` (`state.rs:288`) must keep naming every field. If
`sheprd-live` lands, `state/tests/atomic_projection.rs` (288 lines) and
`state/tests/surfaces.rs` (159) move with it — never as copies — so
`cargo test -p sheprd-live` fails the day a rule weakens.

**i18n.** Four things move or change owner:
`label()` goes app-side (§4.2); `message()` IDs stay in the crate as data, the
pattern `sheprd-commands` and `sheprd-usage` already use (`docs/i18n.md:126`);
`#[error]` English text stays in the crate's `error.rs` and the app's
`impl Localize for sheprd_settings::Error` renders it; `scripts/check-i18n.py`
scans every crate (`tracked("crates/*.rs")`, `check-i18n.py:318`), so the moved
files stay checked and their `// i18n: migrated` markers travel with them.
`config.rs:417-448`'s four `fl!` diagnostic sentences are the one place where
settings *chooses* wording, and that moves out.

**The façade can rot.** A `pub(crate) use sheprd_settings::{…}` list in
`src/config.rs` is a second place to add a type. It exists only so 180 non-test
files do not churn in the same commit as a 14,000-line move; commits 2 and 3
should delete the flattened paths' aliases for the types they touch, and the
`gpui-split-plan` "What moved" table should say when the façade is gone.

**Blocking I/O in a library.** `Config::load`, `load_startup`, `Theme::load`,
`herdr_settings::{load, save}` all block, and the app calls them on workers
(`window.rs:139-141` holds the two `Task`s). The crate must carry
`herdr_settings.rs:3-4`'s warning — "callers MUST run them on a background
worker, never in render/input handlers" — in its own doc comment, or the rule
lives only in the caller's memory.

**One more crate's worth of build.** `sheprd-settings` adds a compile unit but
removes nothing from `sheprd-gpui`'s dependency set; its win is faster
`cargo test -p sheprd-settings` (8,191 config test lines without GPUI) and the
feature extractions it enables, not a smaller app binary. Unlike `sound`, this is
not a dependency win and should not be sold as one.

## 7. The decision the owner must make

`Config` cannot move while `Config.keybindings: Keymap` and
`FontConfig.weight: FontWeight` are `gpui` types. The weight is bookkeeping —
a `Weight` newtype in the crate plus one `From<Weight> for gpui::FontWeight`.
The keymap is the real question, and it is one question with two answers:

**Option A (recommended) — `Config` moves whole; the parsed keymap becomes
app-side derived state.** `Config` keeps what it already keeps: the raw
`keybinding_overrides: BTreeMap<String, Binding>` and `pane_keys: PaneKeys`
(`config.rs:160,176`, whose doc comment already says why). The app builds the
`Keymap` from them plus the daemon's `[keys]` after loading:

```rust
// src/config.rs — the app's own step, after sheprd_settings::Config::load().
pub(crate) fn keymap(config: &Config, daemon: &sheprd_settings::Daemon) -> Result<Keymap>
```

and holds it in the window beside `config` (`window.rs:138`). Cost: 24
`config.keybindings` mentions (12 outside tests) become `config.keymap`-style
reads, the five test sites that assign a `Keymap` (`window/prefix.rs:231,324`,
`endpoint/lifecycle_tests/{keyboard,function_keys,shifted_keys}.rs`) set the
window field instead, and the app gains one invariant to document: loading the
file now yields settings, and a keymap is a second step that can still fail.
Benefit: every feature crate reads one `&sheprd_settings::Config`, which is what
"value types below it first" means; the alternative leaves each feature taking
20 field values by hand.

**Option B — `Config` stays in the app; only its field types move.** No keymap
work, no test churn, and the settings crate still holds most of the code — but
`Config` and the seven files that load, edit or resolve it (`files.rs` 331,
`edit.rs` 253, `preferences.rs` 303, `theme.rs` 687, `devices.rs` 181,
`fonts.rs` 510, `daemon.rs` 87) stay with it, so `config.rs`'s 700 lines and
those 2,352 stay in the app. Cost: browser/sidebar/chat cannot name `Config` from
outside the app, so each one receives the fields it reads as an explicit borrow
struct, and "settings are below the features" is only approximately true.

Everything in §2–§4 works under either; only §4.4's commit 2 changes shape, which
is why this decision gates commit 2 and not commit 1. My recommendation is A, on
the grounds that the keymap derivation is a ~40-line function and 12 call sites,
against a permanent per-feature field list.

Two smaller decisions ride along and are safe to default:
`sheprd-settings` may depend on `sheprd-commands` for the keybinding-name check
(no cycle; `main.rs:139` already aliases the crate), and the `MAX_VOLUME` single
spelling of §4.5 goes to whichever of `sheprd-settings`/`sheprd-sound` lands
second.

## What this does not do

No behaviour changes, no new dependencies, no `sheprd-live` or painter crate
until a consumer exists, no flattening of `crate::config::` paths beyond the
façade, and no change to `sheprd-gpui`'s linked dependency set. The commits that
shrink the build are still `sound` and `chat`.
