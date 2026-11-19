# Extracting `browser` from `crates/sheprd-gpui`

Design for the first of the three large feature extractions named in
[`docs/gpui-split-plan.md`](gpui-split-plan.md), and the one
[`docs/notes-lower-shared-types-design.md`](notes-lower-shared-types-design.md)
§5 lists first. It is a design only: no production code changed, no build run.

**Verdict.** The tab strip is not on `browser`'s critical path, and it should
not get a crate. `browser` names **five** items of the tab-strip kit, in **two**
of its own files, and the seam between them already exists and is already
narrow: `window/tab_strip.rs:306` asks
`browser_tab_entries(slot, shown, &faces, cx) -> Vec<(TabId, Stateful<Div>)>`,
which is `browser/view/render.rs:22-79`, 58 lines. Option A — a
`sheprd-tab-strip` crate both depend on — is **3,809 production lines**, larger
than the 4,390-line module it is meant to unblock, and every module in §2.3's
right-hand column would have to be lowered beneath it first — two of which
(`browser`, `sidebar`) are the other two big features. It inverts the plan's
own order.

What *does* block `browser` is different from what the plan assumed, and
smaller: **1,398 of its 4,390 production lines are eight `impl SheprdWindow`
blocks** (92 methods) that Rust forbids moving into another crate, and four
things it reads — `core_terminals::panes`, `state_file`, `motion`'s clock, and
the six `crate::Error` variants — have no home below the app.

Measured, the extraction is worth it — not as "4.7k lines leave the app", but
as **4,390 production and 2,078 test lines behind a 612-line mechanical
inversion**, with 786 more lines of entity glue and 1,346 lines of window tests
staying where they are. Section 5 gives the commits; section 7 the one decision
an owner has to make (what `Store`'s file path policy is once it leaves the
app).

## 0. The measurements, and how to repeat them

Every number below is from this worktree at `44c5325e8`:

```sh
find crates/sheprd-gpui/src/browser* -name '*.rs' | xargs wc -l | tail -1   # 7,814
find crates/sheprd-gpui/src/browser* -name '*.rs' \
  | grep -v '/tests/' | grep -v '/view_tests/' | grep -v '/view_tests\.rs$' \
  | grep -v '/tests\.rs$' | xargs wc -l | tail -1                          # 4,750
rg -l 'crate::browser' crates/sheprd-gpui/src --glob '!…/browser*' | wc -l  #   53
rg -c '^impl SheprdWindow' crates/sheprd-gpui/src/browser* -g '!**/tests*'  #    8
```

| | Lines |
| --- | --- |
| `browser*` total | 7,814 |
| production files (19) | 4,750 |
| — of which inline `#[cfg(test)] mod tests` blocks | 360 |
| **production, test bodies excluded** | **4,390** |
| test files (`browser/**/tests/`, `tests.rs`) | 3,064 |
| — of which `view_tests` (headless `Entity<SheprdWindow>` tests) | 1,346 |
| — of which unit test files, which travel with the code | 1,718 |

The plan's "4,748 / 3,064" counts the 360 inline test lines as production and
the 3,064 test-file lines as tests: 4,750 + 3,064 = 7,814 either way. This note
uses the exclude-test-bodies split, because that is what a move actually
transports.

## 1. The import census

### 1.1 The `crate::` surface, counted

`rg -oN 'crate::(\w+)(::\w+)*'` over the 19 production files, plus the four
multi-line `use crate::{…}` lists those files carry, gives the whole list. It is
short:

| `crate::` name | Sites | Where | What it really is |
| --- | --- | --- | --- |
| `crate::Error` | 6 + 5 `InvalidBrowserUrl`, 2 `InvalidBrowserTabs`, 1 each `InvalidCodeFile`/`InvalidGroupLayouts`/`InvalidLocalPage`/`InvalidReviewCheckout` | `browser.rs:58-114`, `store/saved.rs:144,186,212,246,272`, `layouts.rs:47` | six variants of the app's root enum (`error.rs:93-107`), all file-shaped: a saved file this build cannot read |
| `crate::Result` | 9 | `browser.rs`, `store/saved.rs`, `layouts.rs` | the app's `Result<T, Error>` |
| `crate::motion::now` | 8 | `group_motion.rs`, `groups_view.rs:145,153,160,168`, `tab_scroll`, `tab_body` | the QA-slow clock, `motion/slow.rs:40` |
| `crate::motion::progress` | 2 (+2 via `motion::`) | `group_motion.rs:41,127,142,179,181` | `motion.rs:26` |
| `crate::motion::duration`, `ENTER`, `LEAVE` | 2 / 1 / 1 | `group_motion.rs:7`, `tab_body.rs:38` | `motion.rs:20-22`, `motion/slow.rs:45` |
| `crate::i18n::fl` | 6 call sites, **5 messages** | `view.rs:177,186`, `location.rs:36,37`, `closed.rs:369`, `store.rs:197` | `browser.ftl`, 10 lines |
| `crate::state_file` | 7 | `store.rs:7,225,259,269,280`, `layouts.rs:6,32,68,78` | 127 gpui-free lines |
| `crate::preferences::state_dir` | 2 | `store.rs:249`, `layouts.rs:58` | a re-export of `sheprd_client::local_path::state_dir` (`preferences.rs:246`) |
| `crate::config::home` | 1 | `store/saved.rs:270` | `sheprd_client::local_path::home_dir` (`config.rs:372-374`) |
| `crate::SheprdWindow` | 8 `impl` blocks + 6 `use` | see §3 | the app's window |
| `crate::core_terminals::panes::Panes` | 3 | `location.rs:15`, `store/saved.rs:7,60`, `store.rs:482` | 463 lines, 0 gpui |
| `crate::group_terminals::GroupTerminals` | 1 | `view.rs:53` (`Browser.terminals`) | 730 lines, 1 `impl SheprdWindow` block |
| `crate::close_modal::CloseRun` | 1 | `view.rs:55` (`Browser.closing`) | 5-field struct, `close_modal/sequence.rs:26-34` |
| `crate::notifications::safe_text` | 1 | `store.rs:234` | `sheprd_client::text::safe_text` |
| `crate::tab_layout::{Mark, Probe, TabContext, TabFace}` + `TabLayout` | 5 items | `view/render.rs:13`, `tab_body.rs:2`, `groups_view.rs:289` | 1,886 lines — §2 |
| `crate::endpoint::Endpoint` | 2 | `view.rs:97`, `render.rs` via `scope()` | reads `id` and `connection.target` only |
| `crate::controls::Command` | 3 | `tab_move.rs:4`, `groups_view.rs:15,25` | 3 variants used: `MoveTabNext`, `MoveTabPrevious`, `TabNumber` |
| `crate::NavigationTarget` | 3 | `view/agent_tab.rs:3,32,34`, `groups_view.rs:486,522,524` | 36 gpui-free lines (`navigation.rs:1-36`) |
| `crate::window::Flash` | 2 | `view.rs:177,186`, `closed.rs:369` | the window's toast |
| `crate::core_feature::disabled_message` | 1 | `groups_view.rs:730` | a localized sentence |
| `crate::control::Target` | 3 | `view/requests.rs:5` | control-socket targeting |

Two findings worth stating plainly, because both contradict the plan:

1. **`browser` reads no `Config` field and no `Theme` field.** The one
   `config::` path in the whole module is `config::home()` in
   `store/saved.rs:270`, which is `sheprd_client::local_path::home_dir` with a
   different name. The settings design's §1.4 row "`browser` + window strip:
   `corners` (5), `Theme` (1), `home` (1)" is right, and it says the *window
   strip* owns four of those six. **The `sheprd-settings` crate does not
   unblock `browser`**; it unblocks the tab-strip kit, which §2 shows `browser`
   does not need.
2. **`browser` holds no gpui `Entity`.** `grep 'Entity<'` over the production
   files returns nothing; `Browser` is a plain struct that is a *field* of
   `SheprdWindow` (`window.rs:131`), built at `window/construct.rs:110`. That
   is unusually good news for a GPUI module and it is what makes §3's borrow
   struct possible at all — but it also means `Context<Self>` in browser's
   methods is `Context<SheprdWindow>`, not `Context<Browser>`.

### 1.2 Which of those have a crate after the settings crate lands

| Need | Home after `sheprd-settings` | Action |
| --- | --- | --- |
| `config::home`, `preferences::state_dir` | `sheprd-client` already | call `sheprd_client::local_path::*` directly; 2-line change |
| `notifications::safe_text` | `sheprd-client::text::safe_text` already | 1-line change |
| `endpoint::Endpoint` | app | borrow struct: `&[Endpoint]` + `selected_endpoint` (§3) |
| `LiveState` (`live.snapshot`, `live.activation.fenced()`) | app (`sheprd-live` not yet, per settings §2.2) | borrow struct: `Option<&ClientShellSnapshot>` + a `fenced: bool` |
| `state_file` | **none — new crate** | 127 lines, 0 gpui, 9 production modules use it and browser's own tests a tenth |
| `core_terminals::panes::Panes` | **none — new crate** | 463 lines, 0 gpui, browser *and* `core_terminals` use it |
| `motion::{now, duration, progress, ENTER, LEAVE}` | **none — new crate** | 90 gpui-free lines of a 217-line module, 56 app files use it |
| `NavigationTarget` | **none** — app, 36 gpui-free lines | moves with whoever needs it; `sidebar` needs it too |
| `crate::Error`'s six variants | app root, must move | new `browser::Error` + one `#[error(transparent)] Browser(…)` in `error.rs` |
| `i18n::fl`'s 5 messages | app (`docs/i18n.md:126`: messages are the app's) | return message ids or typed values; the app writes the sentence |
| `tab_layout` | app | **stays** — §2 |
| `group_terminals::GroupTerminals` | app | move it out of `Browser` into its own window field — §4.3 |
| `close_modal::CloseRun` | app, or `browser` | it is 5 fields of `Scope`/`Pick`/`String`/`Instant`, so it can ride along |
| `SheprdWindow` | app, always | §3 |

So **three new crates** stand between `browser` and a move, totalling 680
production lines: `Panes` (463), `state_file` (127), and `motion`'s clock (90).
None of them is a feature extraction; each is a gpui-free leaf that `browser`
cannot reach, and each is also wanted by `sidebar` (`motion`, `Panes`) and by
five other app modules (`state_file`).

### 1.3 What the 53 importers want back

53 files name `crate::browser` — 23 production, 30 test. The root API they
reach for, by reference count outside `browser*`:

| Name | Refs | | Name | Refs |
| --- | --- | --- | --- | --- |
| `Store` | 37 | | `Slot` | 10 |
| `CoreGroupKey` | 28 | | `Shown` | 10 |
| `Location` | 24 | | `Scope` | 7 |
| `Tab` | 21 | | `GroupIds` | 4 |
| `TabId` | 20 | | `PageOpened` | 3 |
| `Pick` | 19 | | `Layouts` | 2 |
| `WebUrl` | 19 | | `Thumb` | 2 |
| `GroupId` | 16 | | `Fold`, `Listed`, `ClosedHerdrTab` | 1 each |

Eighteen names: that is a narrow root API and it is the shape the plan's
"re-export a narrow root API" step wants. `Browser` itself is named twice
(`window.rs:131`, `construct.rs:110`).

Two consumers are structural rather than incidental and decide the dependency
direction:

- `core_terminals/observation.rs:72-73` builds a `crate::browser::CoreGroupKey`,
  and `core_terminals/workspace_ui.rs:175` returns `Option<&CoreGroupKey>` from
  `selected_core_group()`. `core_terminals` (10,986 lines, staying) depends on
  browser's types.
- `browser` depends back on `core_terminals::panes::Panes` (4 sites) and on
  `selected_core_group()` (6 sites). That is a **cycle today**, and §4.1 breaks
  it by moving `Panes` down.

## 2. The shared tab strip

### 2.1 What the strip is

| Piece | Production | Tests | gpui shape |
| --- | --- | --- | --- |
| `src/tab_layout.rs` + `src/tab_layout/` (7 layouts + `panes/` chrome) | 1,886 | 11 | 41 `div()`, 33 `Stateful<Div>`, 22 `AnyElement`, `ClickHandler = dyn Fn(&ClickEvent, &mut Window, &mut App)` (`tab_layout.rs:151`) |
| `src/window/tab_strip.rs` + `src/window/tab_strip/` (`curved`, `dividers`, `launcher`, `launcher/deck`, `agent_title`) | 1,923 | 2,788 | 34 `div()`, 10 `Stateful<Div>`, 18 `AnyElement`, 10 `&mut Window`, 2 `&mut App` |
| **total** | **3,809** | **2,799** | 75 `div()`, 43 `Stateful<Div>`, 40 `AnyElement` |

Both halves are `Div`-shaped all the way down: there is no gpui-free core to
peel out. `tab_layout`'s trait `TabLayout` returns `Stateful<Div>` from `tab`
(`tab_layout.rs:226-237`) and `new_tab` (`:243-254`), so even the abstraction
boundary is a GPUI element.

**Two consumers, and neither is the sidebar:**

- `window/render.rs:700` — `render_group`, the only caller of
  `render_tab_strip`. It draws one group: strip above or below a body.
- `core_terminals/split.rs:14,85` — `self.tab_layout().panes()`, the pane chrome
  inside a herdr-core tab split into panes.

`sidebar` does **not** use any of it. It has a parallel row-layout system of its
own (`sidebar/layout.rs` 604, `sidebar/layouts/` 1,486 production lines), which
is the reason the sidebar is the last of the three features and not a consumer
of this kit.

### 2.2 What `browser` actually names

Five items, in two of browser's own files, plus one doc link:

| Site | Names |
| --- | --- |
| `browser/view/render.rs:13` | `tab_layout::{Mark, Probe, TabContext, TabFace}` — builds a `TabFace` per browser tab and calls `layout.tab(base, face, state, faces)` (`:66-72`) |
| `browser/groups_view.rs:289` | `self.tab_layout().min_width()`, inside `grow_tab` |
| `browser/tab_body.rs:2` | a doc comment referring to `TabLayout::connected` — no code |

No other browser production file names the kit. (`browser/view.rs` names
`crate::close_modal::CloseRun` at `:55`, which is a different thing entirely.)

The strip calls back into browser in four places, and each is a plain getter:
`browser_tab_labels` (`tab_strip.rs:192`), `tab_body` (`:223`), `grow_tab`
(`:279`), `browser_tab_entries` (`:306`).

So the whole interface between the strip and browser is: **four getters in, one
element list and four layout names out.** It is already a seam; it is not a
coupling that a crate has to be invented for.

### 2.3 Option A measured — a `sheprd-tab-strip` crate

For `tab_layout` and `tab_strip` to compile below the app, these must already be
below the app. Each is a real site, counted:

| Need | Sites today | Below the app? |
| --- | --- | --- |
| `config::{FontConfig, TabLayoutMode, Theme, corners}` | 5 `use` + `crate::config::corners` ×2 | ✅ `sheprd-settings` |
| `backdrop::Fill` | `tab_layout.rs:24`; `Fill` returns `gpui::Hsla` (`backdrop.rs:166-199`) | ❌ app |
| `browser::{Slot, TabId}` | `tab_layout.rs:25`, `:96`, `:100-143`, `:181` | ❌ **the module being extracted** |
| `sidebar::{Indicators, status_indicator}` | `tab_layout.rs:29` + 5 layout files | ❌ **the last feature to extract** |
| `icons::{AgentIcon, MUTED}` | `tab_layout.rs:30`, `:417` | ❌ app |
| `i18n::fl` | `tab_layout.rs:27,20,456` (2 ids) | ❌ app macro, `$crate`-bound |
| `crate::{TAB_HEIGHT, TAB_WIDTH, TAB_MARGIN}`, `fonts::StyledFont`, `mark_layer::hidden_under_pointer`, `menu::SettingsHint` | 40 `TAB_WIDTH` uses, 1 each of the rest | ❌ app |
| the strip half: `window::{Flash, chrome_focus::Region, tab_drag::StripDrag, Ends}`, `herdr_settings::Settings`, `config::{status_icons, edge_rail, terminal_padding}`, `focus`, `notification_mutes::Mutes`, `worktree_identity`, `controls::Command` | `tab_strip.rs:24-38` and throughout | ❌ app |

Seven of the eight rows are ❌, and two of those are the two features the plan
schedules *after* browser. Concretely:

- `Slot` and `TabId` are browser's. Either the strip crate is built first and
  browser depends on it (fine), or browser is built first and the strip depends
  on browser (fine) — but the strip's `Probe::Page(TabId)` and `Slot::id:
  GroupId` mean one of the two must own the other. There is no third option
  without making the strip generic over an opaque key, which `Probe`'s
  `"browser-tab-status-{id}"` formatting (`tab_layout.rs:103`) shows is not
  free.
- `sidebar::{Indicators, status_indicator}` is only needed for a Herdr tab's
  agent-status dot (`tab_layout.rs:344-360`, `:388-390`). Extracting it means
  extracting `sidebar/agents.rs:265-410` — from the module with 263 importers.

**Cost of Option A: 3,809 production lines of new crate** (larger than browser's
4,390 before the inversion is counted), **plus `backdrop`, `icons`, `i18n`'s
macro, `constants`/`fonts`/`mark_layer`/`menu`, `window`'s chrome,
`herdr_settings`, `notification_mutes`, `worktree_identity` and `controls`
lowered first — and, after those, `browser` and `sidebar` themselves.** Benefit
to the `browser` extraction: **zero lines.** Every one of browser's five kit
uses sits in `view/render.rs` and `groups_view.rs`, both of which are
`impl SheprdWindow` render code that must stay app-side under any option (§3.4).

### 2.4 Option B measured — browser keeps a thin app-side renderer

Under Option B the app keeps the strip, keeps `browser/view/render.rs` and
`browser/groups_view.rs`, and the browser crate holds no `tab_layout` name at
all.

| | Lines |
| --- | --- |
| `tab_layout` moved | **0** |
| strip files moved | **0** |
| new crates for the strip | **0** |
| browser files that stay for the strip | `view/render.rs` 79, `tab_body.rs` 150, and `grow_tab` (10 lines) inside `groups_view.rs` |
| browser production lines that therefore stay | **239**, all already `impl SheprdWindow` |

### 2.5 Recommendation

**Option B.** A `sheprd-tab-strip` crate costs 3,809 production lines and the
first half of §2.3's list as prerequisites, to save browser 239 lines it would
have had to keep anyway; it cannot even be built before the sidebar's status
indicators have a home, and the sidebar is scheduled last.
`browser/view/render.rs`'s `browser_tab_entries` is already exactly the seam
Option A would create, and it is 58 lines.

If the strip is ever extracted, the honest route is the other direction: extract
it *with* `core_terminals` (its second consumer) after `sidebar` has lowered
`Indicators`, and treat it as its own program, not as browser's prerequisite.
That is a decision for a later wave and this note does not schedule it.

## 3. The borrow struct

### 3.1 Why a struct rather than parameters

`browser` reads `self.endpoints` (9 sites), `self.selected_endpoint` (10),
`self.live` (8, of which `live.snapshot` ×6 and `live.activation.fenced()` ×1),
`self.primary_group()` (6), `self.parked_focuses()` (1), `self.selected_core_group()`
(8), `self.core_enabled()` (5), `self.tab_layout()` (1), and `self.hosts()` (2).
Passing those as nine parameters to 64 methods is the cost the settings design
already rejected for its Option B. The precedent is `sheprd-sound`'s
`Live<'a>` (`crates/sheprd-sound/src/policy.rs:25-39`): "The app owns the state;
this crate only reads it."

### 3.2 `BrowserHosts<'a>`

Exactly the fields the 64 non-entity methods read, no more. `Browser` keeps
owning everything mutable:

```rust
/// What one window offers the browser for a single call. The app owns the
/// window and the store; this crate only reads them.
pub struct BrowserHosts<'a> {
    /// Every endpoint this window knows, in `SheprdWindow.endpoints` order.
    /// The browser reads an endpoint's `id` and `connection.target` only.
    pub endpoints: &'a [Endpoint],
    /// Which of `endpoints` this window shows, `SheprdWindow.selected_endpoint`.
    pub selected_endpoint: usize,
    /// The newest metadata of the shown endpoint, or none before it connects.
    pub snapshot: Option<&'a ClientShellSnapshot>,
    /// The shown endpoint's selected herdr-core group, when it has one; takes
    /// the place of both the endpoint and the workspace while it is set.
    pub core_group: Option<&'a CoreGroupKey>,
    /// Whether this build shows herdr-core features: tabs that need it are
    /// left out of strips when false.
    pub core_enabled: bool,
    /// The window's tab store, `cx.global::<Store>()`.
    pub tabs: &'a Store,
    /// The narrowest a tab draws in the layout in use, `TabLayout::min_width`,
    /// which a growing tab starts from.
    pub tab_min_width: f32,
    /// `motion::now()`, read once per call so every animation in it agrees.
    pub now: Instant,
}
```

Field-by-field justification, all from §1.1's census:

| Field | Replaces | Sites |
| --- | --- | --- |
| `endpoints` | `self.endpoints` | 9 |
| `selected_endpoint` | `self.selected_endpoint` | 10 |
| `snapshot` | `self.live.snapshot` | 6 |
| `core_group` | `self.selected_core_group()` | 8 |
| `core_enabled` | `self.core_enabled()` | 5 |
| `tabs` | `store(cx)` inside `browser_tab_ids`, `browser_tab_labels`, `group_shown` | 4 |
| `tab_min_width` | `self.tab_layout().min_width()` | 1 |
| `now` | `crate::motion::now()` | 8 |

Two fields are deliberately **not** in it:

- **`live.activation.fenced()`** (`groups_view.rs:470`). One site, and it gates
  input: `input_ready()` and `navigate()` stay app-side with the methods that
  take `Context<Self>` (§3.4), so `step_tab`'s check happens there.
- **`GroupTerminals`.** It is `Browser`'s own field today
  (`view.rs:53`), but it holds `ConnectionBridge`/`LiveState`/`Presentation`/
  `terminal_painter` — terminal machinery, not browser state. §4.3 moves it out
  of `Browser` into a `SheprdWindow` field of its own, which makes
  `browser → group_terminals` disappear entirely instead of becoming a host
  field. `primary_group()` and `parked_focuses()` (`group_terminals.rs:144`,
  `:172`) then stay app-side methods on `SheprdWindow`, and the one browser site
  that calls them is inside an entity method anyway.

### 3.3 Who constructs it, and the two-borrow problem

One place, as `sheprd-sound` does with `LiveState::sound_live`: an
`impl SheprdWindow` accessor.

The naive
`self.browser.settle(&self.browser_hosts(cx))` **does not compile**: the host
borrows all of `self` for its lifetime, so `self.browser` cannot be borrowed
mutably across it even though the two are disjoint fields. Destructuring fixes
it, because the borrow checker splits on fields when it can see them:

```rust
impl SheprdWindow {
    /// The window's browser and what it may read of the window in one call.
    /// Destructured so the two borrows are known to be of different fields.
    fn browser_parts(&mut self, cx: &App) -> (&mut Browser, BrowserHosts<'_>) {
        let SheprdWindow {
            browser, endpoints, live, config, ..
        } = self;
        let hosts = BrowserHosts {
            endpoints,
            selected_endpoint: /* copied, `usize` */,
            snapshot: live.snapshot.as_deref(),
            core_group: /* from core_terminals, read before the split */,
            core_enabled: /* `config` */,
            tabs: /* the global, read before the split */,
            tab_min_width: /* the layout, read before the split */,
            now: motion::now(),
        };
        (browser, hosts)
    }
}
```

Two values need reading *before* the destructure — the core group (it lives in
`core_terminals`) and the `Store` global (it is a `cx` global) — so the accessor
starts with three lines of `let` before the split. That is the one awkward part
of this design and it is confined to one function.

**The important consequence:** the ~30 browser methods other files call (§1.3)
keep their `impl SheprdWindow` signature untouched. Existing call sites outside
`browser*` — `window/commands.rs`, `window/render.rs`, `control/server/tabs/*`,
`group_menu/*`, `endpoint/lifecycle_tests/*` — do not change at all. Only
browser's own files change, and only by gaining `let (browser, hosts) = …`.

### 3.4 What does not fit: the 28 entity methods

Of browser's 92 `impl SheprdWindow` methods, **28 take `&mut Context<Self>`** and
**64 take neither**. That split is the whole inversion budget:

| | Methods | Lines | Becomes |
| --- | --- | --- | --- |
| no `Context<Self>` | 64 | **612** | `impl Browser` methods taking `&BrowserHosts<'_>` — mechanical: `self.browser.` → `self.`, `self.live.snapshot` → `hosts.snapshot` |
| `&mut Context<Self>` | 28 | **786** | `impl SheprdWindow` shims that keep their signature, call `browser_parts`, and then `cx.notify()` / `cx.emit` / `window.focus` |

The 786 lines are the top of the file sizes and they are the ones that genuinely
need the entity: `reopen_closed_tab` (72), `settle_groups` (66),
`terminal_focus_moved` (62), `observe_closed_herdr_tabs` (59),
`browser_tab_entries` (57 — it builds elements with `cx.listener`, so it must
stay), `forget_closed_workspaces` (52), `close_browser_tab` (46), `step_tab`
(39), `show_browser_tab_in` (36), `split_group` (32), `move_tab_to_group` (28),
`close_group` (23).

`terminal_focus_moved` (`groups_view.rs:642`) is browser's — it routes the
daemon's focus change into the right group — but it stays app-side like the rest
of the 28, because it calls `primary_group`/`set_primary_group`, which commit 2
below moves onto the window.

Two files are neither browser's nor the entity's: `view/requests.rs`
(`control_target`, `shows_daemon`, `endpoint_showing` — 64 lines of control-socket
targeting) and `view/agent_tab.rs` (`go_to_workspace`, `shows_workspace` — 32
lines of window navigation). **96 lines of browser's 1,398 are app navigation
filed in the wrong module** and should move to `window/` or `control/` as the
first, free step of this work.

## 4. What moves whole vs stays

### 4.1 The four prerequisites, each a separate crate or move

| # | What | Lines | gpui | Deps | Users today |
| --- | --- | --- | --- | --- | --- |
| P1 | `core_terminals::panes` → **`sheprd-panes`** | 463 | 0 | `serde`, `sheprd-backend` (`TerminalId`), `thiserror` (3 variants) | browser (4 sites), `core_terminals` (12), `chat/core_pane.rs` |
| P2 | `state_file` → **`sheprd-state-file`** | 127 | 0 | `serde`, `serde_json`, `tempfile`, `thiserror` | browser (2), `agent_skill`, `notification_mutes`, `overview/marks`, `palette/frecency`, `window_state`, `worktree_notes`, `main` |
| P3 | `motion`'s clock → **`sheprd-motion`** | 90 | 0 | none | `motion/slow.rs` (61) + `progress`/`ENTER`/`LEAVE` (`motion.rs:20-31`); 56 app files, browser (13 sites) |
| P4 | `navigation::NavigationTarget` | 36 | 0 | `sheprd-client` | browser (3), sidebar, `window/` |

`Panes` **should not ride along with `browser`.** It is read by
`core_terminals` (`core_terminals.rs:218,240,262,297,355,392,726`,
`commands.rs:10,132`, `split.rs`) — a module that stays in the app — and by
`chat/core_pane.rs`. A `sheprd-panes` crate makes both readers equal clients;
riding along would force `core_terminals` to depend on `sheprd-browser` for a
pane tree, which is the cycle §1.3 already has to break. Its own error type
(`panes/error.rs`, three variants, `Duplicate`/`Full`/`Missing`) moves with it
and needs no `Localize` arm — nothing shows those sentences to a user today.

`state_file` is the only prerequisite whose *policy* is a decision rather than a
move (§7).

### 4.2 What rides with `browser`

| File | Production | gpui | Note |
| --- | --- | --- | --- |
| `groups.rs` | 575 | **no** | the layout algebra: `Layout`, `Slot`, `Pick`, `Shown`, `GroupId`, `GroupIds`, saved layouts. Its 454 test lines are pure data and move unchanged. |
| `group_motion.rs` | 201 | **no** | open/fold shares; needs `sheprd-motion` |
| `store/saved.rs` | 308 | **no** | the saved-file schema, incl. the retired `code`/`review`/`vs_code`/`px0` variants the tests pin (`store/tests/retired_*.rs`, 232 lines) |
| `location.rs` | 40 | **no** | `Devices` / `CoreTerminal { panes, observe }`; 2 messages |
| `browser.rs` | 86 | **no** | `WebUrl` (`:34-100`) + the re-export list |
| `store.rs` | 519 | `App`/`Global` only | `Tab`, `TabId`, `Scope`, `CoreGroupKey`, `Store`; 32 methods |
| `layouts.rs` | 181 | `App`/`Global` only | `Layouts`, the saved-groups global |
| `hidden.rs` | 96 | `WindowId` only | tear-off hiding; `WindowId` is the one handle that travels |
| `tab_appear.rs` | 149 | `SharedString` | grow/shrink of a tab in its strip |
| `tab_scroll.rs` | 183 | `ScrollHandle`, `Pixels` | the strip's sideways scroll and thumb |
| `tab_body.rs` | 150 | `Bounds`, `Pixels` | the sliding body under a connected strip |
| `closed.rs` (data half) | 275 | no | `ClosedHerdrTab` is 4 `String`s; the other half is `impl SheprdWindow` |
| `view.rs` (type half) | 124 | no | the `Browser` struct and `PageOpened` |
| `groups_view.rs` (helpers) | 49 | no | `neighbour`, `stepped` — pure |
| **production moving** | **≈2,936** | | |
| **plus test lines** | **≈2,078** | | `groups/tests` 454, `store/tests/*` 795, `closed/tests` 243, `tab_body/tests` 90, `code/tests` 56, `saved/tests` 51, `tab_scroll/tests` 19, `view/tests` 10, inline 360 |

Roughly **2,936 of 4,390 production lines (67%) and 2,078 of 3,370 test lines
(62%) move.** The rest is `impl SheprdWindow` glue and element building.

`close_modal::CloseRun` (5 fields, `close_modal/sequence.rs:26-34`) rides along.
It is `Scope`/`Pick`/`VecDeque<Pick>`/`Option<String>`/`Option<Instant>` and has
no gpui and no app type; its driver, `close_modal/sequence.rs`'s
`impl SheprdWindow` (98 lines), stays with the app beside `open_tab_close`.

`reorder` and `truncate` are already crates and need nothing: `sheprd_reorder`
is named at `tab_move.rs:6` and `store.rs:10` (`Beside`), `sheprd_truncate` at
`view.rs:105-106` (`tab_title`). A browser crate inherits both as ordinary
dependencies.

### 4.3 What stays

| File | Lines | Why |
| --- | --- | --- |
| the 28 `Context<Self>` methods | 786 | `cx.notify()`, `cx.listener`, `window.focus` — the entity |
| `view/render.rs` | 79 | builds `Stateful<Div>` per browser tab with `cx.listener` |
| `view/agent_tab.rs`, `view/requests.rs` | 96 | not browser at all; window navigation and control targeting (§3.4) |
| `group_settle.rs` | 79 | one `Context<Self>` method (counted in the 786 above) that drives `browser.layouts` and the window's connections |
| `group_terminals.rs` | 730 | terminal machinery; see below |
| the strip's four browser getters | — | already listed in §2.2 |
| `view_tests` | 1,346 | drive `Entity<SheprdWindow>`; kept as the app's integration tests for the crate, exactly as `sound` kept `connection/tests/sounds.rs` |

**`GroupTerminals` moves out of `Browser`, into the window.** Today
`Browser.terminals: GroupTerminals` (`view.rs:53`) and all 14 of its uses are
`self.browser.terminals.…` inside `group_terminals.rs` itself. That field is the
one thing that would drag `ConnectionBridge`, `Presentation`, `LiveState` and
`terminal_painter::{Ground, ImageTarget, PlacedImages}` into a browser crate.
Making it `SheprdWindow.group_terminals` is a 1-line field move plus 14
`self.browser.terminals` → `self.group_terminals` edits, and it turns
`browser ↔ group_terminals` from a cycle into one edge
(`group_terminals → browser` for `GroupId`/`Scope`/`Pick`/`Shown`/`Slot`). It is
worth doing on its own merits, before any crate exists.

**i18n: 5 messages, 6 call sites.** All five live in
`locales/en-US/browser.ftl` (10 lines):

| Id | Site | Fate |
| --- | --- | --- |
| `browser-open-workspace-first` | `view.rs:177` | app writes it (`show_flash` is app-side) |
| `browser-too-many-tabs` | `view.rs:186`, `closed.rs:369` | app writes it |
| `browser-tab-devices` | `location.rs:36` | `Location` becomes data; the app maps it to the message (`drawn_title` goes app-side, exactly as the settings design's §4.2 moves `label()`) |
| `browser-tab-terminal` | `location.rs:37` | same |
| `browser-tab-new` | `store.rs:197` | `Store::open` gains a `title: Option<String>` parameter, or the crate returns the id `"browser-tab-new"` as data (`sheprd-commands`' pattern) |

`Location::default_title()` (`location.rs:26-32`) already returns English as
data with `// i18n: data` for the saved file and the CLI, so the split is
already half-done; `drawn_title` (`:34-39`) is the half that calls `fl!`.

## 5. The commit plan

Each commit compiles and each is independently reviewable. Steps 1–2 are free
(reorganising code that stays in the app); 3–5 build the three leaf crates; 6–9
move browser.

### 5.1 The commits

| # | Subject | Contents | Compiles because | Unblocks |
| --- | --- | --- | --- | --- |
| 1 | `refactor: Move app navigation out of the browser module` | `browser/view/agent_tab.rs` → `window/navigate.rs`, `browser/view/requests.rs` → `control/targets.rs`; 96 lines, `git mv` + `mod` wiring | pure file moves inside the app | — (removes 96 lines from the extraction's blast radius) |
| 2 | `refactor: Give a window's terminal groups their own field` | `GroupTerminals` out of `Browser` into `SheprdWindow`; 14 `self.browser.terminals` → `self.group_terminals` | one field, same type, same app | breaks `browser ↔ group_terminals`; needed by commit 9 |
| 3 | `refactor: Give a herdr-core tab's panes a crate` | `sheprd-panes` (P1); `AGENTS.md` crate table; workspace `members` + `[workspace.dependencies]` | `core_terminals::panes` becomes `sheprd_panes`; both readers stay in the app | **sidebar** (Panes), **chat** (`core_pane.rs`) |
| 4 | `refactor: Give the state-file writer a crate` | `sheprd-state-file` (P2); the app's `state_file` becomes a re-export | 10 callers unchanged | 6 app modules; **browser**'s `Store`/`Layouts` |
| 5 | `refactor: Give the animation clock a crate` | `sheprd-motion` (P3): `slow.rs`, `progress`, `ENTER`, `LEAVE`; `motion.rs` re-exports and keeps `pop_in`/`Toggle`/`unfold` | `crate::motion::now` still resolves | **sidebar**, **chat**, **browser** |
| 6 | `refactor: Give native browser tabs a crate` | `sheprd-browser` (part 1): the gpui-free half — `groups.rs`, `group_motion.rs`, `location.rs`, `store/saved.rs`, `WebUrl`; `browser.rs` stays a façade re-exporting them | every moved name is re-exported from `crate::browser::{…}`, so 53 importers do not change | **chat** (14 `browser::` refs) |
| 7 | `refactor: Move the browser tab store into sheprd-browser` | `store.rs`, `layouts.rs`, `hidden.rs`, `tab_appear.rs`, `tab_scroll.rs`, `tab_body.rs`, `closed.rs`'s data half; `Store`/`Layouts` stay `gpui::Global` in the crate | `app.rs:438-447` calls `sheprd_browser::Store::load()` through the façade | — |
| 8 | `refactor: Invert the browser's 64 window methods onto a borrow struct` | `BrowserHosts<'a>`; 64 methods become `impl Browser` (612 lines); `browser_parts` accessor; the 28 `Context<Self>` shims keep their signatures | ~30 external call sites keep their spelling | this is the line that makes the move possible at all |
| 9 | `refactor: Move the browser's groups into sheprd-browser` | `groups_view.rs`'s 49 helper lines, the `Browser` struct, `PageOpened`, `Scope`/`store`/`tab_label` free functions; 5 messages to the app | commit 8 did the inversion; the crate no longer needs `LiveState`, `Endpoint` or `tab_layout` | — |
| 10 | `docs: Record the browser extraction` | `gpui-split-plan.md` "What moved" row and next-extraction table; this note's status | docs | — |

Every commit runs `cargo check -p <crate> -p sheprd-gpui`,
`cargo test -p <crate>`, `cargo fmt --all -- --check`, `just check-file-size`,
`just check-sorted`, `just check-i18n`; `cargo metadata` refreshes
`Cargo.lock`, which the commit includes.

### 5.2 What each unblocks for sidebar (next) and chat (last)

| Step | Sidebar | Chat |
| --- | --- | --- |
| 1–2 | nothing directly; commits 3 and 5 do | nothing |
| 3 `sheprd-panes` | `Panes` (`split.rs`, `core_terminals/sidebar.rs`) | `chat/core_pane.rs` no longer reaches `core_terminals` |
| 4 `sheprd-state-file` | `worktree_notes`, `device_tags`, `overview/marks` — the state sidebar reads | `notification_mutes` |
| 5 `sheprd-motion` | `motion::now` in `sidebar/activity.rs`, `reorder.rs`, `mark_layer` | `motion::Toggle`, `unfold`, `pop_in` stay app-side, so chat keeps the app `motion` module until it moves |
| 6–9 browser crate | **removes `browser` from sidebar's blocker list** — `core_terminals/sidebar.rs:10` names `browser::{Location, Store, TabId}` and the sidebar's fixture window builds `browser::Browser::new` (`sidebar/layout_tests.rs:727-728`), each becoming a one-word path change | **chat's `browser::` refs (14 in 9 files) become crate paths**, which is `chat`'s listed blocker `browser` (§5 of the settings note) |

Sidebar's own blockers after this program — `icons` (19), `menu` (55),
`endpoint` (45), `bind_keys` (24), `direction` (19), `mark_layer`,
`worktree_notes`, `device_tags`, `notification_mutes`, `usage`, and its
`(usize, &Endpoint, &LiveState)` borrow (`sidebar/order.rs:18-22`,
`listed_hosts`) — are untouched
by this note except that `BrowserHosts` gives sidebar a proved pattern to copy,
including the `browser_parts` destructure, which sidebar will need more of
because it borrows `Endpoint` *and* mutates itself.

## 6. Risks

**Entity handles in browser's views — the good news, and the fine print.**
There are none: `Entity<` does not appear in the production files. But the
handles that *do* travel are gpui's own: `WindowId` (`hidden.rs:9`, `:22`, `:50`)
and `ScrollHandle` (`tab_scroll.rs:5`). Both mean `sheprd-browser` must depend
on `gpui`, i.e. it is a **GPUI crate, the first one in this repo**. Every prior
extraction (`ago`, `osc52`, `words`, `lenient`, `sound`, and the settings crate
now building) is gpui-free, so this is a new kind of move and the plan's "no
`gpui` types" pattern does not cover it. The consequences are worth stating so
nobody is surprised: `gpui.workspace` is already pinned at
`Cargo.toml:67` (`gpui-pre =0.3.10`), so the crate adds a compile unit that
depends on gpui but **removes nothing from the app's dependency set** — unlike
`sound`, this is not a build-graph win. The win is `cargo test -p sheprd-browser`
over 2,078 test lines without building 313k lines of app, plus one crate whose
whole surface can be read in an afternoon. Sell it as that.

**The tab strip's daemon-order contract.** The strip's order is *Herdr tabs
first, then browser tabs*: `tab_strip.rs:190-196` chains
`browser_tab_labels` after the snapshot's tabs, and `groups_view.rs:335-352`
(`workspace_tabs`) rebuilds the same order for commands.
`tab_move.rs:1-2` says why: "This client orders them itself, after the
workspace's Herdr tabs, so the daemon's `tab.move` cannot." The rule is pinned
by `view_tests/tab_strip.rs:3` (`dragging_a_browser_tab_reorders_the_workspace_s_tabs`),
which asserts a browser tab dragged over a Herdr tab *stays put*. That test is
a window test and stays app-side — but it now crosses a crate boundary, so the
crate's public API must keep `Pick`, `GroupId`, `Slot` and `Layout::lists`
exposed at exactly the granularity the strip needs. If the inversion of commit 8
makes `group_lists` or `workspace_tabs` private, this contract loses its test.
Keep both public and keep the test.

**Drag/reorder against the fresh `sheprd-reorder` crate.** `sheprd-reorder` is
110 lines of std-only geometry (`Beside`, `Slide`, `slot_for`, `preview`), and
browser already uses it correctly: `tab_move.rs:11-34` computes a `Beside` and
`store.rs` applies it. The gpui drag machinery is *not* browser's and must not
follow it: `StripDrag` (`window/tab_drag.rs:109`, 716 lines of file) owns the
press/lift/carry/drop phases and calls browser only through `places` and
`group_lists`. The risk is an over-eager move that pulls `StripDrag` into the
crate because it mentions `Pick`; it must stay, for the same reason
`browser_tab_entries` must stay — it is fed by `cx.listener` and
`window.on_mouse_event`. The `sheprd-reorder` dependency itself is unchanged by
this extraction, and the one thing to check at review time is that the crate's
`Slide` clock still comes from `motion::now()` at the call site
(`tab_drag.rs`), not from inside the layout crate — AGENTS.md's clock rule and
`docs/clock-audit.md` both apply to the moved code.

**i18n message ownership.** Five messages, and the crate must not carry any of
them: `docs/i18n.md:97` ("Crates below the app have no messages") and
`scripts/check-i18n.py` scan every `crates/*.rs` (`:318`), so a moved file keeps its
`// i18n: migrated` marker and its English must already be gone. `fl!` cannot
be used in another crate at all — the macro is `macro_rules! fl` expanding to
`$crate::i18n::…` (`i18n.rs:31-44`), where `$crate` is `sheprd_gpui`. Three of
the six sites move (the crate returns data), three stay (the app writes them).
`Location::drawn_title` is the only one that needs a decision of shape: put the
`fl!` call in an app-side `fn drawn_title(&Location) -> String`, matching the
`Label` trait the settings design adds at `config.rs`.

**The 360 inline test lines.** AGENTS.md: "Keep test bodies out of production
files, which only declare `#[cfg(test)] mod tests;`." Six browser files still
carry inline modules (`layouts.rs:183` 98, `tab_appear.rs:151` 64,
`tab_scroll.rs:185` 56, `group_motion.rs:203` 51, `browser.rs` 42,
`hidden.rs:98` 37, plus two 2-line `mod tests;` decls). Moving them is a
prerequisite for a clean `src/tests.rs` in the new crate, and doing it in the
same commit as the move would hide 360 lines of churn inside it. Do it in
commits 6–7 per file.

**Two borrows, one `self`.** §3.3 is the only real design risk: 64 methods
moving from `&mut self: SheprdWindow` to `&mut self: Browser` + `&BrowserHosts`
forces every call site inside browser to destructure first. If the destructure
turns out not to typecheck for the calls that reach `core_terminals` (the core
group is behind `self.core_terminals.workspace_ui.selected_group`), the fallback
is to copy `CoreGroupKey` out (it is `Clone`: `store.rs:75-79`) rather than
borrow it, at the cost of one clone per call. That fallback is why
`BrowserHosts.core_group` should be `Option<&'a CoreGroupKey>` only if it
survives review; `Option<CoreGroupKey>` costs one `Clone` per call and removes
the question.

**`Store`'s path policy.** §7.

## 7. The decision the owner must make

`sheprd-state-file` (P2) has to decide what `state_file` knows about *paths*,
and the browser store's two files (`browser-tabs.json`,
`editor-groups.json`) are its first customer:

**Option A (recommended) — the crate is the file layer, and callers resolve
paths.** `sheprd-state-file` owns `read(path, limit)`, `write(path, value)` and
`Writer<T>::start(name, path)`, exactly as `state_file.rs` does today, and the
app keeps calling `preferences::state_dir().map(|dir| dir.join(FILE))` as it
already does at `store.rs:249` and `layouts.rs:58`. `Store::load` becomes
`Store::load(path: Option<PathBuf>)` — the same shape `sheprd-sound`'s
`Settings::load(path)` took (`docs/gpui-split-plan.md` "What three couplings had
to invert"). Cost: 9 production call sites pass a path instead of computing it inline.
Benefit: the crate has no opinion about where state lives, `sheprd_client`
keeps sole ownership of `state_dir`, and the "never write a file a fixture
should not" rule (`app.rs:437-447`) stays visible in one place.

**Option B — the crate resolves paths itself** (`sheprd_state_file::read_named(FILE, limit)`)
by depending on `sheprd-client::local_path`. Less churn at the call sites, but
every crate that writes state then has two ways to name a file, and the fixture
rule above moves into a dependency.

I recommend A, on the grounds that the settings design already chose it for
`sound` and that the alternative gives nine modules a second path API for no
lines saved.

Two smaller calls ride along and are safe to default:

- **Where `Panes` lives.** A `sheprd-panes` crate rather than folding into
  `sheprd-backend`: `sheprd-backend` is the terminal value types herdr-core
  shares *with the app*, and a tab's pane tree is this client's own persisted
  layout, read by nobody below it. Folding it in would widen a crate whose
  charter is "the traits `sheprd-core-client` implements".
- **Where `NavigationTarget` goes.** It is 36 gpui-free lines used by browser,
  sidebar and `window/`. Either it joins `sheprd-panes`' wave in a small
  `sheprd-navigation`, or it stays app-side and `BrowserHosts` carries the two
  `String`/`&str` it needs. Default: leave it app-side; no browser method
  outside the 28 entity ones reads it (§3.4).

## What this does not do

No production code changed and no build run. No tab-strip crate (§2.5), no
`sheprd-live` (settings §2.2 still governs: browser reads `snapshot` and one
`fenced()` flag, which the borrow struct carries), no change to `sheprd-gpui`'s
dependency set except the `[dependencies]` lines `sheprd-browser` adds —
`gpui`, `serde`/`serde_json`/`uuid`/`url`/`thiserror`/`tracing`, and the three
crates of §4.1 — and no statement about whether `sidebar` or `chat` will go faster than
this program did — chat's 20,440 production lines and 174 messages are a
different order of problem.
