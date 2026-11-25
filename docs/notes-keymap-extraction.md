# Extracting the keymap from `crates/spaceport-gpui`

Design for task wx76. It is a design only: no production code changed, no build run.

**Base revision, and what this tree is.** This session has no shell, so `git fetch
origin main` and `git log --oneline -1` were not run. The refs were read instead:
`.git/refs/heads/main` and `.git/refs/remotes/origin/main` both hold
`78254c3478e165da408e94cbfc3ed648089768d2`. Whether that is at or past the target
`f778a8626c` cannot be checked without git; the tree's contents say it is a late one
(`crates/spaceport-live`, `spaceport-endpoint`, `spaceport-menu`,
`spaceport-palette`, `spaceport-widgets` all exist; `crates/spaceport-keymap` does
not, which is what this note is for). §0 gives the commands to run before stage 1.

**Verdict.** The keymap is the opposite shape from every module extracted so far,
and the difference is worth stating first because it decides the whole plan. It is
**not** a view, not a state machine, and not entangled with the window. It is one
resolved value plus the rules that produce it, and it names **six `gpui` sites in six
files** — all six are the same three data types (`Keystroke`, `Modifiers`,
`OperatingSystem`) plus one error type. Measured:

| | browser | settings | menus | endpoint | **keymap** |
| --- | --- | --- | --- | --- | --- |
| production lines | 4,390 | 11,520 | 20,146 | 3,068 | **1,446** |
| test lines in the same tree | 1,904 | ~8,000 | ~20,000 | ~13,330 | **1,645** |
| `gpui::` sites in production | — | — | many | 0 in the model | **6, in 6 files** |
| `gpui::test` / `TestAppContext` | — | — | — | — | **0** |
| builds an element or `impl Render` | yes | yes | yes | no | **no** |
| `impl SpaceportWindow` inside the module | — | — | — | ~1,210 | **0** |
| `fl!` sites inside the module | — | ~150 | ~377 | 12 | **0** |
| message IDs it produces | — | — | — | — | **17, all worded app-side already** |
| what blocks it | nothing | text | text, two entities | two app types with no home | **three app-side calls, two of them ~50 lines of pure data** |

Two consequences follow, and they pull in opposite directions.

1. **Almost all of it can move, and cheaply.** 1,446 production lines and 1,645 test
   lines with five external names: `spaceport_commands::{Command, COMMANDS}` (already
   a crate), `spaceport_settings::keys::{Binding, PaneKeys}` (already a crate),
   `spaceport_client::protocol::ClientShellCommand` (already a crate), `toml`, and
   `gpui`'s keystroke values. Nothing in it reads the window, an entity, an `App`, or
   a `Context`.
2. **It cannot become a gpui-free crate.** `gpui::Keystroke` is the vocabulary the
   whole app speaks — the window's input path, GPUI's own `KeyBinding`, the palette's
   key table, every smoke test's `dispatch_keystroke`. A gpui-free `spaceport-keymap`
   would need a mirror keystroke type plus a re-implementation of gpui's parser,
   unparser and `should_match` (§2). AGENTS.md forbids exactly that mirror.

So the plan is four moves into **one new GPUI crate, `spaceport-keymap`**, which like
`spaceport-sidebar`, `spaceport-menu`, `spaceport-settings-window` and
`spaceport-browser` holds GPUI's values and no window, no element, no entity, no word
and no message. The precedent is explicit in AGENTS.md's crate table: a GPUI crate is
allowed when what it holds is GPUI's values.

## 0. The measurements, and how to repeat them

Every count is the file's own length in this tree. Re-run before each stage:

```sh
git log --oneline -1                                                       # record the base revision
find crates/spaceport-gpui/src/keymap* -name '*.rs' ! -path '*/tests/*' \
     ! -name 'tests.rs' | xargs wc -l | tail -1                            # 1,446 production
find crates/spaceport-gpui/src/keymap* -name '*.rs' -path '*/tests/*' -o \
     -name 'tests.rs' | xargs wc -l | tail -1                              # 1,645 test
rg -n 'gpui' crates/spaceport-gpui/src/keymap.rs crates/spaceport-gpui/src/keymap/  # 6 sites
rg -c '^#\[test\]' crates/spaceport-gpui/src/keymap*                       # 55 tests
rg -o 'crate::keymap::' crates/spaceport-gpui/src | wc -l                  # ~185 occurrences
rg -l 'crate::keymap::' crates/spaceport-gpui/src | wc -l                  # ~70 files
rg -o 'fl!\("' crates/spaceport-gpui/src/keymap* | wc -l                   # 0
rg -n 'self\.keymap\(\)|view\.keymap\(\)' crates/spaceport-gpui/src | wc -l  # ~35 sites
```

### 0.1 The module, file by file

| file | Lines | `gpui` | what it is |
| --- | --- | --- | --- |
| `keymap.rs` | 507 | `KeybindingKeystroke`, `Keystroke`, `Modifiers` | `Keymap`, `Shortcut`, `PaneKey`, `with_overrides`, `layer`, the queries (`shortcuts`, `primary`, `bindings`, `pane_key`, `prefix_label`, `is_prefix`, `chord`, `triggers`, `navigates_workspace`), `resolve_pane_keys`, `parse`, `typed_matches`, `has_modifier`, `default_pane_keys` |
| `keymap/custom.rs` | 282 | `Keystroke`, `Modifiers` | daemon `[[keys.command]]` reach: `Reach`, `CustomBinding`, `custom_command`, `custom_bindings`, `holds_directly`, `bound_directly`, `direct_keystrokes`, `custom_label`, `unsupported_label`, `prefix_word`, `available_commands` |
| `keymap/daemon.rs` | 543 | `Keystroke`, `Modifiers` | the daemon `[keys]` table: `ACTIONS` (52 rows), `Target`/`Indexed`, `Trigger`, `DaemonKeys::{default, from_table, from_profile}`, `entries`, `prefixes`, `triggers`, the Herdr→GPUI keystroke translator, `set_modifier` |
| `keymap/daemon/error.rs` | 16 | — | four variants: `Missing`, `NoKeys`, `Parse`, `TooLarge` |
| `keymap/error.rs` | 57 | `InvalidKeystrokeError` ×2 | 13 variants, `#[source]` on the two keystroke parses |
| `keymap/host.rs` | 41 | `InvalidKeystrokeError`, `Keystroke`, `Modifiers`, `OperatingSystem` | `HOST_OS` (a `cfg!` const), `parse_keystroke`, `secondary`, `secondary_key` (`#[cfg(test)]`) |
| **production** | **1,446** | | |
| `keymap/tests.rs` | 586 | — | 15 tests + the shared fixtures (`overrides`, `one`, `keystroke`, `no_keys`, `with`, `list`, `running`, `custom`, `pane_keys`, `sent`, `key`) |
| `keymap/daemon/tests.rs` | 393 | — | 13 tests: the `[keys]` reader |
| `keymap/daemon/tests/detach.rs` | 34 | — | 2 tests |
| `keymap/daemon/tests/literal_plus.rs` | 83 | — | 5 tests |
| `keymap/tests/custom_defaults.rs` | 52 | — | 1 test |
| `keymap/tests/custom_reference.rs` | 159 | — | 6 tests |
| `keymap/tests/literal_plus.rs` | 75 | — | 4 tests |
| `keymap/tests/pane_keys.rs` | 55 | — | 2 tests |
| `keymap/tests/platform_defaults.rs` | 136 | — | 4 tests |
| `keymap/tests/plugin_actions.rs` | 17 | — | 1 test |
| `keymap/tests/shift_enter.rs` | 55 | `KeyDownEvent` | 2 tests |
| **test** | **1,645** | | |

`docs/gpui-split-plan.md:291` calls the module "`keymap` (1,440 production lines)";
the 1,446 here is the same number one refactor later.

### 0.2 The six `gpui` sites, and the seven names they use

| site | names |
| --- | --- |
| `keymap.rs:29` | `KeybindingKeystroke`, `Keystroke`, `Modifiers` |
| `keymap/custom.rs:12` | `Keystroke`, `Modifiers` |
| `keymap/daemon.rs:15` | `Keystroke`, `Modifiers` |
| `keymap/host.rs:5` | `InvalidKeystrokeError`, `Keystroke`, `Modifiers`, `OperatingSystem` |
| `keymap/error.rs:21,27` | `InvalidKeystrokeError` |

What is actually *done* with them, exhaustively:

| use | where | gpui surface |
| --- | --- | --- |
| parse text → `Keystroke` | `host.rs:19` | `Keystroke::parse_with_os(text, OperatingSystem)` |
| print → text | `keymap.rs:184,225,243-244,326`, `custom.rs:213,218` | `Keystroke::unparse(OperatingSystem)` |
| match a typed key against a bound one | `keymap.rs:396-398` | `Keystroke::should_match(&KeybindingKeystroke::from_keystroke(..))` |
| read/modify modifiers | throughout | `Modifiers::{platform, control, alt, shift, function}`, `Modifiers::default/command/control` |
| pick the host OS | `host.rs:7` | `OperatingSystem::{Mac, Windows, Linux, FreeBsd}` |
| key identity | `keymap.rs:68-70` | `keystroke.modifiers`, `keystroke.key` as a `String` |
| build one from parts | `daemon.rs:435,487-495` | `Keystroke { modifiers, key, key_char }` |

That is a plain data struct, a plain bitfield, an enum, and two functions
(`parse_with_os`, `unparse`) plus one matching rule (`should_match`). No `App`, no
`Window`, no `Context`, no `Entity`, no `Element`. The module is already, in effect,
a library — it is just filed inside the app.

## 1. The inventory that decides the plan

### 1.1 Who consumes it, and what each one takes

`~185` occurrences of `crate::keymap::` in `~70` files, plus `~35` `self.keymap()`
call sites. The production consumers, exhaustively:

| file | what it uses | why it must keep calling |
| --- | --- | --- |
| `window.rs:214` | `keymap: Keymap` field | the window holds the resolved value |
| `window/construct.rs:353` | `Keymap` by value | constructor parameter |
| `config.rs:119-122` | `DaemonKeys::from_table`, `Keymap::with_overrides` | the one derivation step (`docs/notes-lower-shared-types-design.md` §7 Option A) |
| `window/server_keys.rs:11` | `Binding`, `DaemonKeys::from_profile`, `Keymap::with_overrides`, `PaneKeys` | a device's published profile, rebased when the selection changes |
| `window/prefix.rs` | `is_prefix`, `chord`, `holds_directly`, `triggers`, `custom_command` | prefix mode's dispatch |
| `window/input.rs:404,425,433` | `pane_key(&Keystroke)` | `[pane_keys]` interception |
| `chat/window.rs:380` | `pane_key` | the chat composer's `[pane_keys]` |
| `palette.rs:8,82,315,389,403-414,475,510` | `Reach`, `Keymap`, `available_commands`, `primary`, `custom_bindings` | row badges and the custom-command rows |
| `palette/keys.rs:66` | `navigates_workspace` | the workspace picker's up/down |
| `menu/settings.rs:14,169,329,349-350,737-747` | `Reach`, `Keymap`, `available_commands`, `shortcuts`, `custom_bindings` | the Keybinds page and its reach notes |
| `menu/shortcut_editor.rs:10,45-46,139-159` | `Binding`, `DaemonKeys::default`, `Keymap::with_overrides`, `bindings` | the shortcut recorder |
| `control/server/settings/keys.rs:10,39,45,55,61` | `Reach`, `available_commands`, `shortcuts`, `custom_bindings` | the `spaceport keys` CLI answer |
| `actions.rs:200-212,219-226` | `Keymap`, `parse_keystroke`, `EDIT_KEYS`, `SYSTEM_KEYS` | installing GPUI `KeyBinding`s |
| `menus/in_window/render.rs:167` | `HOST_OS` | menu labels |
| `menus/in_window/native.rs:17` | `parse_keystroke` | native menu key equivalents |
| `window/links.rs:267,294,406`, `window/find.rs:355-356`, `menu/devices/picker.rs:503` | `secondary(&Modifiers)` | "the platform's primary modifier is held" |
| `terminal.rs:14,502-513` | `PaneKey`, and `reaches_pane` lives here | turning a pane key into a `ClientPaneInputEvent` |
| `window/status_bar.rs:77` | `prefix_label()` | the prefix hint |
| `window/zen.rs:222` | `primary(Command::ToggleZenMode)` | the zen hint |
| `settings_window.rs:160`, `settings_window/source.rs:17,36`, `settings_window/persistence.rs:25` | `Keymap` by value | the Settings window's reload carries one |
| `app.rs:17`, `error.rs:103,186`, `i18n/errors/keymap.rs`, `i18n/errors/root.rs:126` | `Keymap`, `Error`, `daemon::Error`, `Localize` | the type, its two error variants and their words |

The other ~45 files are tests and the smoke harness: `window/links/tests/*` (~30
`secondary_key()` uses), `terminal/links/tests`, `window/file_links/tests`,
`endpoint/lifecycle_tests/{keyboard,function_keys,shifted_keys,dead_keys,...}`,
`settings_window/tests/*`, `chat/tests/zoom.rs`, `dialog_input/tests/*`,
`sidebar/layout_tests/device_multi_select.rs`, `smoke*/**.rs`. Every one of them
takes the same three things: `parse_keystroke`, `secondary_key`, `Keymap::*`.

### 1.2 The test inventory

55 `#[test]` functions, **zero** `#[gpui::test]`, zero `TestAppContext`, zero
`add_window_view`. The tests are pure functions over strings and `COMMANDS`. Four
files reach into the app, for five reasons:

| test site | app name | what it needs |
| --- | --- | --- |
| `tests.rs:485` | `crate::terminal::reaches_pane` | the keystroke→`ClientKeyCode` rule (§3, stage 2) |
| `tests/platform_defaults.rs:30` | `crate::actions::reserved_keystrokes` | `EDIT_KEYS` + `SYSTEM_KEYS` (stage 3) |
| `tests/platform_defaults.rs:37` | `crate::controls::CTRL_SHORTCUTS` | already a crate value |
| `tests/shift_enter.rs:5,17` | `gpui::KeyDownEvent`, `crate::terminal::pane_key_input` | one `assert_eq!` on the serialized key (stage 2) |
| `tests.rs:46-140`, `daemon/tests.rs:140-170`, `window/server_keys/tests.rs:134` | `crate::Error::Keymap(..)`, `crate::Error::ServerKeybindings(..)` | the app root's two nested variants (§4, stage 1) |

Everything else — 1,600-odd lines — moves byte for byte.

### 1.3 Words: measured, not assumed

`fl!` in the module: **0**. `Labels`, `locale`, `i18n`: **0**. The 13 keymap and 4
daemon error variants already carry English `#[error("…")]` text in `error.rs` and
are worded for people by `i18n/errors/keymap.rs` (17 message IDs in
`locales/en-US/error-keymap.ftl`), which is the shape `spaceport-palette` and
`spaceport-endpoint` already have: the crate names the variant, the app writes the
sentence. `menu/settings.rs:737-747`'s six `menu-settings-*` reach words are already
app-side and stay there. This module needs **no** i18n decision, which is unusual and
is most of why it is cheap.

### 1.4 The error shapes, and the one thing that has to change

Both enums already follow the convention: `thiserror`, `// keep-sorted start/end`,
one `Error` per module, variants not prefixed by the feature, `#[source]` preserved.
The app root carries them as two transparent variants:

```rust
// crates/spaceport-gpui/src/error.rs
#[error(transparent)] Keymap(#[from] crate::keymap::Error),                    // :103
#[error(transparent)] ServerKeybindings(#[from] crate::keymap::daemon::Error), // :186
```

The one shape that does not survive a move untouched is `DaemonKeys::from_profile`,
which today returns the **app's** `Result` so `window/server_keys.rs:38-39` can chain
it with `Keymap::with_overrides` through `.and_then`. Once both live in the crate
they must share the crate's `Result`, so the crate's root `Error` gains one variant:

```rust
#[error(transparent)]
Daemon(#[from] daemon::Error),
```

and the app's `ServerKeybindings` becomes redundant. §4 stage 1 settles whether to
keep it (and add an `Error::Daemon` arm to `i18n/errors/keymap.rs`) or retire it (and
update the six test assertions that name it). Recommended: retire it — one failure
gets one definition, which is the rule the app root already follows for
`spaceport_sound`, `spaceport_menu` and `spaceport_settings`.

## 2. The decision: a GPUI crate, or a gpui-free split

Both were counted. The question is not "does GPUI appear" but "what does a gpui-free
crate cost at the boundary", because every consumer already speaks
`gpui::Keystroke`.

### Option A — `spaceport-keymap` is a GPUI crate (recommended)

| | |
| --- | --- |
| moves | **1,446 production + 1,645 test lines**, whole |
| new dependencies in the graph | `spaceport-keymap → gpui` |
| app-side code rewritten | 6 lines (the `gpui::` imports), plus the crate's own `Error` root |
| call sites rewritten | 0 in stages 1–3 (a `crate::keymap` façade), ~185 if stage 4 flattens them |
| boundary conversions | **none** — `Keystroke` in, `Keystroke` out |
| new mirror types | **none** |
| what it buys | the whole resolution rule set is testable without the app (`cargo test -p spaceport-keymap`, ~1,645 lines, seconds); `spaceport-menu`, `spaceport-sidebar` and `spaceport-settings-window` can read a resolved keymap without naming the app |

### Option B — split: a gpui-free `spaceport-keystroke` leaf, resolution above it

To make the resolution gpui-free, something below it must own "a keystroke". That
something is a new type — call it `spaceport_keystroke::Keystroke` — with:

| must be written | cost |
| --- | --- |
| a parser with gpui's exact grammar: `-` separators, modifier aliases (`cmd`/`super`/`platform`, `ctrl`/`control`, `alt`/`opt`, `function`/`fn`), `->` shifted-input syntax (`shift-2->@`, used by `keymap/tests.rs:174`), named keys, `f1..f35`, a literal `-` and `+` | ~250–350 lines, and a drift risk against `gpui-pre 0.3.10` that no test in this repo would catch on an upgrade |
| an unparser with the same platform spellings | ~80 lines, and `HOST_OS`'s four variants |
| `should_match`, including "a shifted symbol matches however the keyboard reports it" | ~40 lines, and the rule is subtle enough that `keymap/tests.rs:169-177` exists to pin it |
| conversions at every boundary | `From`/`Into` in the window's input path, in `actions.rs`'s `KeyBinding::new`, in `palette/keys.rs`, in GPUI's own `KeyDownEvent.keystroke`, in ~20 test call sites |
| a mirror-type audit | AGENTS.md: "Define shared types once in the lowest appropriate layer and re-export them. Do not create mirror enums or convert between them through strings." This is a mirror type whose conversions are exactly string conversions |

Option B is the direction the split plan wants in principle (a gpui-free core), and
it is the wrong trade here: it charges ~500 lines of re-implemented parser plus a
permanent conversion seam to remove one dependency edge, and it makes the app's own
`Keystroke` the second-class citizen. `gpui-split-plan.md:293` already states the
pattern a feature needs — "a value type for what the crate reads, and a parameter for
what the app owns" — and here the value type **already exists and is gpui's**.

**Recommendation: Option A**, with one condition written into the crate's doc
comment: *if* a future keystroke value type lands below GPUI for another reason (the
terminal's input encoder is the only candidate, `terminal.rs:581-628`), the keymap
crate can then drop its `gpui` edge in one commit without changing any consumer,
because it only ever names `Keystroke` and never GPUI's runtime.

## 3. What moves, what stays, and why

### 3.1 Moves

**1,446 production lines** (the whole module) **+ 1,645 test lines**, plus two small
pieces of files outside it:

| item | from | lines |
| --- | --- | --- |
| `Keymap`, `Shortcut`, `PaneKey`, `with_overrides`, `layer`, `default_pane_keys`, `resolve_pane_keys`, `parse`, `typed_matches`, `has_modifier`, the constants, the queries | `keymap.rs` | 507 |
| `Reach`, `CustomBinding`, `custom_command`, `custom_bindings`, `holds_directly`, `direct_keystrokes`, `custom_label`, `unsupported_label`, `prefix_word`, `available_commands` | `keymap/custom.rs` | 282 |
| `ACTIONS`, `Target`/`Indexed`, `Trigger`, `DaemonKeys`, `entries`, `prefixes`, `triggers`, the translator | `keymap/daemon.rs` | 543 |
| 4 daemon variants | `keymap/daemon/error.rs` | 16 |
| 13 keymap variants | `keymap/error.rs` | 57 |
| `HOST_OS`, `parse_keystroke`, `secondary`, `secondary_key` | `keymap/host.rs` | 41 |
| all 55 tests and their fixtures, less §1.2's five app couplings | `keymap/tests*`, `keymap/daemon/tests*` | ~1,645 |
| `reaches_pane` + the `Keystroke`→`ClientKeyCode` table it needs | `terminal.rs:509-513,581-628` | ~54 |
| `EDIT_KEYS`, `SYSTEM_KEYS`, `reserved_keystrokes`, `free_system_keys` | `actions.rs:175-212,219-226` | ~50 |

The last two are the only lines that move out of a file outside `keymap/`:
**1,550 production lines leave the app, 1,446 of them the module.** Both are forced:
`resolve_pane_keys` refuses a `[pane_keys]` target the pane cannot receive
(`keymap.rs:471`), and `custom.rs:206` asks which keystrokes the app reserves. Both
are pure data/`&str` logic over `gpui::Keystroke` and `spaceport_client::protocol`, so
they belong below the window anyway.

### 3.2 Stays app-side, for good (and is not a defect)

| stays | lines | why |
| --- | --- | --- |
| `window/prefix.rs` dispatch (`intercept_prefix`, `prefix_keystroke`, `bound`, `resize_keystroke`, `disarm_prefix`) | ~250 prod / ~400 test | GPUI interceptors, `stop_propagation`, menu dismissal, `self.command(..)` — it *asks* the keymap and acts on the window |
| `window/prefix/shifted_keys.rs` (`Typed`, `KeyChars`, `restore`, `current_layout`) | ~180 | GPUI's own key naming policy: what a layout reports, what Shift restored means |
| `window/function_keys.rs` (`named`, `unnamed`) | ~90 | GPUI function-key naming, not binding |
| `window/server_keys.rs` (`ServerKeymap`, `ActiveServerKeymap`, `sync_server_keymap`, `publish_server_keymap`) | 140 prod / 140 test | holds a `gpui::Global`, republishes app-wide bindings, reads `self.live.snapshot` and `self.endpoints` |
| `actions.rs`'s `bind_keys`/`rebind_keys`, the `actions!` macro, `KeyBinding::new` | ~120 | installing GPUI bindings; the keymap tells it *what* to bind |
| `terminal.rs`'s `pane_key_input`, `keystroke_input`, `HeldKeys` | ~120 | the protocol encoder (`ClientPaneInputEvent`), not the binding |
| `menus/in_window/*` key-equivalent rendering, the `keycaps` splitter, `reach_note` | ~60 | words and elements |
| the `Keymap` field on the window and its 3 owners (`window.rs`, `settings_window/{source,persistence}.rs`) | ~10 | window state |
| `i18n/errors/keymap.rs`, `i18n/errors/root.rs:126` | 100 | the app's words (unchanged, one arm added) |

Total staying: roughly **1,200 production lines** of window glue, which is where
`prefix.rs` + `server_keys.rs` + `actions.rs` already live. That is the browser
program's terminal shape, not a defect.

### 3.3 The crate's shape

```
crates/spaceport-keymap/src/
  lib.rs        # the crate doc, re-exports, Error root (gains Daemon(#[from]))
  error.rs      # 13 variants, moved                                    (57)
  host.rs       # HOST_OS, parse_keystroke, secondary, secondary_key    (41)
  keymap.rs     # Keymap, Shortcut, PaneKey, with_overrides, queries    (507)
  custom.rs     # the daemon custom-command reach                       (282)
  daemon.rs     # the [keys] reader                                     (543)
  daemon/error.rs
  pane.rs       # reaches_pane + the key-code table, from terminal.rs   (~54)
  reserved.rs   # EDIT_KEYS, SYSTEM_KEYS, reserved_keystrokes, free_system_keys (~50)
  tests/        # the 55 tests, one topic file each as today
```

Dependencies: `gpui`, `spaceport-client`, `spaceport-commands`, `spaceport-settings`,
`thiserror`, `toml`. Feature: `test-support`, exporting `secondary_key` and the
`platform_default`-style helper, enabled from `spaceport-gpui`'s `[dev-dependencies]`
beside the other fourteen.

Every file lands under 600 lines, so `just check-file-size` is not a constraint here —
but the module split must be kept, not flattened, or `daemon.rs` + `keymap.rs` would
cross 1,000 on the first merge.

## 4. The staged plan

Each stage is a PR on its own. Each starts the same way:

> **Survey first**: `git log --oneline -1`; re-read the files the stage names;
> re-count with §0's commands; confirm the boundary being moved still has no runtime
> GPUI in it (`rg -n 'App|Window|Context|Entity|Element' <files>`). Do not begin from
> this note's line numbers.

### Stage 1 — the crate exists, and the daemon half moves (~600 production / ~510 test)

**What moves**: `keymap/daemon.rs` (543), `keymap/daemon/error.rs` (16),
`keymap/host.rs` (41), and `keymap/daemon/tests*` (393 + 34 + 83).

**What changes in the app**: `keymap.rs` keeps its contents and gains
`pub(crate) use spaceport_keymap::{daemon, …}` re-exports, so `crate::keymap::`
paths keep working in all ~70 files. `error.rs` keeps `Keymap(#[from]
crate::keymap::Error)` and either keeps or retires `ServerKeybindings` (§1.4).

**Survey first**:
1. Does the crate's root `Error` take a `Daemon(#[from] daemon::Error)` variant, and
   does `ServerKeybindings` go? Recommended: yes and yes; check
   `i18n/errors/root.rs:126`, `i18n/errors/keymap.rs` and the six assertions that
   match `ServerKeybindings` before committing.
2. `DaemonKeys::from_profile` returns the crate's `Result`; `window/server_keys.rs:38`
   becomes `Result<Keymap>` in the crate's terms or maps explicitly. Pick one and
   write why.
3. Do `daemon/tests.rs:140-170`'s four assertions move to
   `window/server_keys/tests.rs` (which already asserts one), or stay in the crate
   against the crate root? Recommended: move — they are about the app's diagnostic,
   not the reader.

**Exit criteria**: `cargo test -p spaceport-keymap` green (25 tests);
`cargo check -p spaceport-gpui --all-targets`; `just check-file-size`,
`check-sorted`, `check-i18n`; the daemon half has no `App`/`Window`/`Context`; the
`server keybindings` device menu still reports all three failures in the app
(`just test-gui <herdr>`, deferred to CI if no desktop).

### Stage 2 — the resolution moves (~620 production / ~1,020 test)

**What moves**: `keymap.rs`'s contents (507) and `keymap/error.rs` (57) into
`spaceport-keymap/src/{keymap.rs,error.rs}`; `reaches_pane` plus its table
(`terminal.rs:509-513,581-628`, ~54) into `pane.rs`; `keymap/tests.rs` (586) and
`keymap/tests/*.rs` except `platform_defaults.rs`'s reserved-key test and
`shift_enter.rs`'s serialization assertion.

**What changes in the app**: `terminal.rs` calls `spaceport_keymap::reaches_pane` and
`spaceport_keymap::key_code`, keeping its `pane_key_input`/`keystroke_input`/`HeldKeys`
where they are. `config.rs:119-122`'s derivation is unchanged in shape and now returns
`spaceport_keymap::Result<Keymap>`. `terminal/tests.rs:549` follows the move.

**Survey first**:
1. Is `key_code` (`terminal.rs:581-628`) used by anything but `keystroke_input` and
   `reaches_pane`? If a third caller appears, export it rather than copying it.
2. Does `Keymap` need to stay `Clone + PartialEq + Eq`? It is compared in
   `palette.rs:510` and `window/server_keys.rs:105`; keep the derives.
3. `PaneKey` is public today; decide `pub` vs `pub(crate)`-through-the-crate.
   `terminal.rs:14,502` and `config/tests/keybindings.rs:240-243` need it public.

**Exit criteria**: `cargo test -p spaceport-keymap` green (~48 tests);
`cargo check -p spaceport-gpui --all-targets`; the four `just check-*` recipes;
`rg -n 'gpui' crates/spaceport-keymap/src` shows only the six known sites; the
`spaceport keys --json` CLI output is unchanged (it reads `Reach` and `shortcuts`).

### Stage 3 — the custom-command reach and the reserved keys move (~330 production / ~305 test)

**Landed early in stage 2** (PR for `feat/wx78-keymap-2`): Rust forbids an
inherent impl on a foreign type, so `custom.rs`'s `impl Keymap` block moved with
the type, and the reserved-keys module moved with it (`custom.rs:203` calls
`reserved_keystrokes`). Stage 3 therefore shrinks to re-pointing
`platform_defaults.rs`'s reserved-key assertion into the crate and settling the
two `cfg`/`test-support` questions.

**What moves**: `keymap/custom.rs` (282) into `custom.rs`; `actions.rs`'s
`EDIT_KEYS`, `SYSTEM_KEYS`, `reserved_keystrokes`, `free_system_keys` (~50) into
`reserved.rs`; `keymap/tests/{custom_defaults,custom_reference,literal_plus,plugin_actions}.rs`
(52 + 159 + 75 + 17) and the reserved-key assertion in `platform_defaults.rs`.

**What changes in the app**: `actions.rs` imports `EDIT_KEYS`/`SYSTEM_KEYS` from the
crate (its `bind_keys` body is untouched); `palette.rs`, `menu/settings.rs` and
`control/server/settings/keys.rs` take `Reach`/`available_commands` from the crate
through the façade.

**Survey first**:
1. `reserved_keystrokes` is macOS-only for the system half (`cfg!(target_os =
   "macos")`); a `cfg` gate in a crate is proven only on that platform (§5, risk 2).
2. `free_system_keys` is `#[cfg(any(target_os = "macos", test))]`; in a crate that
   becomes `test-support` or a `cfg` on the crate — pick the one that keeps the
   `windows` CI leg green.
3. Does anything else use `EDIT_KEYS`? `actions.rs:139,150` does; that stays app-side
   and imports it.

**Exit criteria**: `cargo test -p spaceport-keymap` green (55 tests, the count today);
`cargo check -p spaceport-gpui --all-targets`; four `just check-*`; the Edit menu's
four shortcuts and macOS Hide/Minimize still bind (`just test-gui`, deferred to CI if
no desktop).

### Stage 4 — flatten the façade (~0 lines moved, ~185 renames)

**Landed as wx81 (PR #249).** It was not one string swap: the façade
re-exported three different sources under one path (`DaemonKeys` from the
crate's daemon module; `Binding`/`PaneKeys` from spaceport-settings; the rest
from the crate), and 194 direct `spaceport_keymap::` paths in 87 files replaced
the 163 `crate::keymap` lines and 15 in-brace imports. No load-bearing
re-export remains — the app has no lib target. This summary paragraph:

Optional, and deliberately last: `crate::keymap::X` → `spaceport_keymap::X` in ~70
files, deleting the re-export block. `spaceport-settings` and `spaceport-endpoint`
both kept their façade and flattened later as a readability commit, and the same is
right here: it is a diff that touches 70 files and cannot change behaviour, so it
should not ride on a stage that can.

**Exit criteria**: `rg -n 'crate::keymap' crates/spaceport-gpui/src` is empty apart
from nothing at all; `cargo check -p spaceport-gpui --all-targets`; the four
`just check-*` recipes.

### What it adds up to

| stage | production lines moved | test lines moved | app lines touched |
| --- | --- | --- | --- |
| 1 | 600 | 510 | ~40 (façade, root error, 6 assertions) |
| 2 | 620 | 1,020 | ~70 (`terminal.rs`, `config.rs`, 2 test files) |
| 3 | 330 | 305 | ~30 (`actions.rs` imports, 3 consumers) |
| 4 | 0 | 0 | ~185 renames in ~70 files |
| **total** | **1,550 out of the app (1,446 the module + 104 from `terminal.rs`/`actions.rs`)** | **1,645** | |
| **stays** | **~1,200 of window glue** (§3.2) | ~700 (`prefix.rs`, `server_keys.rs` tests) | |

## 5. Risks, ranked

1. **The crate is a GPUI crate, so the split plan's "no GPUI" goal is not advanced
   for this module.** This is the honest cost of Option A and should be stated in the
   commit body, not glossed. What *is* gained is that the rule set leaves the app: it
   is testable in seconds without GPUI's test harness, and `spaceport-menu`,
   `spaceport-sidebar` and `spaceport-settings-window` can each read a resolved
   keymap without naming `crate::keymap`. `sidebar`'s five shortcut-label sites
   (`notes-lower-shared-types-design.md` §5) are the concrete beneficiary.
2. **`cfg(target_os)` in a new crate is proven only on the platform it names.** Three
   platform branches move: `HOST_OS` (`host.rs:7`), the macOS-only
   `DEFAULT_PANE_KEYS` filter (`keymap.rs:412`), and macOS `SYSTEM_KEYS`. A macOS run
   cannot see the Windows branch and vice versa — rely on the `windows` and `checks`
   CI jobs, and run `just lint-linux` / `just lint-windows` locally before handoff if
   the change looks platform-sensitive.
3. **`should_match` semantics must not drift.** `keymap.rs:396-398` deliberately
   borrows GPUI's rule so a shifted symbol (`?`, `shift-/->?`) matches however the
   keyboard reports it, and `keymap/tests.rs:169-177` pins it. It moves as a one-line
   call; do not "simplify" it into a comparison of `(modifiers, key)` while moving.
4. **`reaches_pane` drags a protocol table out of `terminal.rs`.** That file is
   1,100+ lines of GPUI, so moving 54 lines out of it is safe but the survey must
   confirm `key_code` has exactly two callers. If it grows a third, export the
   function instead of copying it.
5. **The `Error` root change ripples into i18n and six test assertions.** §1.4. Small,
   but it is the only place a *behaviour* (which variant a caller can match) changes,
   and `i18n/errors/tests/keymap.rs` checks the wording of every variant — so a
   dropped `ServerKeybindings` arm will be caught, which is what we want.
6. **`test-support` gating.** `secondary_key` and `platform_default` are `cfg(test)`
   today and used from *other* modules' tests, so they must be `feature =
   "test-support"` exports (the `spaceport-sidebar` pattern), enabled from
   `spaceport-gpui`'s dev-dependencies. Forgetting it makes the app's tests fail to
   compile in the `default` pass and pass in the `all` pass — the same class of gap
   as project note `ci-clippy-all-features-gap`.
7. **One more `toml` reader in the dependency graph** — none new, but the crate adds
   `toml` as a direct dependency where it was previously only reached through
   `spaceport-settings`. It is already in `[workspace.dependencies]`; keep the version
   inherited.
8. **The 55 tests are a single-purpose suite with no window coverage.** They prove
   resolution, not dispatch. Prefix mode, the Keybinds page and the shortcut editor
   are proven only by `just test-gui <herdr>` and manual QA; state that plainly rather
   than implying the suite covers them.

## 6. The summary paragraph, in plain language

The keymap is the piece of the app that answers one question: when you press a key,
which command runs? Today that code sits inside the app, mixed in with the windows.
It does not need to be there. It reads two config files, it talks to one other
library for the idea of "a key", and it never draws anything or touches a window. So
we can move it into its own small package. The move is in four steps, one pull
request each. First the part that reads the daemon's key table. Then the part that
works out which command owns which key. Then the part that follows the daemon's own
custom shortcuts. Last, a tidy-up pass that shortens the paths other files use. After
that, the Settings window, the menus and the sidebar can ask for the keymap directly
instead of going through the app. The catch is honest and small: this package still
needs the graphics library for its "key" type, because inventing a second key type
would mean copying a parser, and a copied parser drifts. Everything else — the rules,
the bounds, the tests — leaves the app. About 1,450 lines of real code and 1,650 lines
of tests move. About 1,200 lines of window wiring stay where they are, and should.

## 7. What this does not do

No production code changed and no build run, so every number here is a file length in
this tree, not a moved diff. No decision is taken beyond Option A: §1.4's error-root
question, stage 1's three survey items and stage 3's two `cfg` questions are named for
the implementing PR, not answered by guess. It does not design a gpui-free keystroke
type (§2 Option B), and it does not claim the module gets faster or the binary smaller
— it moves a compile unit and makes 1,645 lines of rules testable without GPUI.
It does not touch `window/prefix.rs`, `window/server_keys.rs`, `actions.rs`'s binding
half, or the 17 message IDs, all of which stay. It does not promise that menus stage
5b (the Keybinds page and shortcut editor) is unblocked: those need the P2 text-field
crate as well; this removes one of their two blockers, not both.