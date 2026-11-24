# wx50: the source snapshot the Settings window reads (P4)

Prerequisite **P4** of `docs/notes-settings-extraction.md` §2.2, the
precondition stage 4 (and later stages) needs: the Settings window stops
reaching into the `SpaceportWindow` it was opened from for the values it shows,
and reads one value — `SourceWindow` — that the app keeps fresh through a
subscription on that window.

Everything below was surveyed on `79cbc1476` (the base of `feat/wx50-settings-source`),
by name, after the design's line numbers had drifted. No call site outside
`crates/spaceport-gpui/src/settings_window/` changed: `open()` keeps its
signature, so `menu/settings.rs` is untouched.

## 1. The survey: every `SpaceportWindow` reach in the module

The design (§1.3) counted ~15. There are **17 sites**, and six of them live in
five files beside `settings_window.rs`. They fall into three shapes, and only
the first is a *read*.

### A. Values the window owns, read by the Settings UI (11 sites)

| # | Before | Function | What it read |
| --- | --- | --- | --- |
| A1 | `settings_window.rs:236-244` | `SettingsWindow::new` | `config`, `keymap`, `theme`, `settings.shared` — four reads, cloned into the editable draft |
| A2 | `settings_window.rs:251-258` | `SettingsWindow::new` | `endpoints.first().connection.target`, cached as `new_window_target` |
| A3 | `settings_window.rs:383` | `retarget_source` | `integrations.busy`, to refuse a retarget mid-listing |
| A4 | `settings_window.rs:392-396` | `retarget_source` | `endpoints.first().connection.target` for the new source |
| A5 | `settings_window.rs:448-455` | `publish_appearance` | over **every** main window: `menu.page == Some(Page::Themes)`, `theme_save_in_flight()` |
| A6 | `settings_window.rs:575-582` | `additional_window_target` | `endpoints.first().connection.target`, live |
| A7 | `sidebar.rs:216-222` | `host_rows` | `endpoints`: `label`, `live.status.is_connected()`, `connection.target` |
| A8 | `remote_history.rs:174-187` | `connected_ssh_hosts` | `endpoints`: `label`, `live.status.is_connected()`, `connection.target` |
| A9 | `settings_window.rs:162` + `:248-250` | field + `new` | `source: WeakEntity<SpaceportWindow>` and the `cx.observe` that made it the freshness seam |
| A10 | `settings_window.rs:379,388-391` | `retarget_source` | `source.entity_id()`, `cx.observe`, `self.source = source` |
| A11 | `pages/notifications.rs:33` | `render` | `source.upgrade().is_some()` — whether the source is alive, gating a button |

### B. Calls that change the source window (6 sites; not reads)

These need a `&mut SpaceportWindow`, so a value cannot stand in for them. They
belong to **P5** (the appearance/integrations service) and are unchanged.

| # | Before | Function | What it calls |
| --- | --- | --- | --- |
| B1 | `settings_window.rs:398` | `retarget_source` | `source.load_integrations(cx)` |
| B2 | `settings_window.rs:613-616` | `select_section` | `source.load_integrations(cx)` |
| B3 | `persistence.rs:339-341` | `reload`'s completion | `source.sound.set_volume(volume)` |
| B4 | `controls/preferences.rs:84-86` | `render_integration_controls` | `source.render_filtered_integrations(&query, cx)` — a `Div` built from the window's own integration list, its scroll handle and its selection epoch |
| B5 | `remote_history.rs:244` → `:287` | `save_remote_history` | `queue_reload`, which asks every endpoint on that host for `reload_config` |
| B6 | `pages/notifications.rs:131-132` | sound test button | `source.sound.preview()` |

### C. Draft broadcasts: `cx.windows()` fanning out into every main window

`sidebar.rs:70-74`, `layouts.rs:31-35`, `themes.rs:405-413, 476-482, 697-701`
write a preview into *every* main window (and read that window's picker as a
fence). They are the other half of the draft seam, are stage 1's
(`spaceport_settings::Drafts`) on the read side and P5's on the write side, and
are outside this stage.

## 2. The value: `crates/spaceport-gpui/src/settings_window/source.rs`

```rust
pub(super) struct SourceWindow {
    config: Config,
    keymap: Keymap,
    theme: Theme,
    shared: Option<herdr_settings::Settings>,
    target: Option<ConnectTarget>,
    integrations_busy: bool,
    theme_page_open: bool,
    hosts: Vec<SourceHost>,   // label + target, connected endpoints only
}
```

Mirroring `browser/hosts.rs`, the app builds it and the consumer reads only the
value. One difference: `BrowserHosts` is a borrow for a single call, while
Settings keeps a snapshot for the life of its window, so it has three entry
points instead of a constructor:

- `of(&SpaceportWindow)` — everything, when Settings opens.
- `follow(&SpaceportWindow)` — the facts that keep changing: `target`,
  `integrations_busy`, `theme_page_open`, `hosts`. Called from
  `SettingsWindow::source_changed`, the `cx.observe` callback the window already
  had.
- `release()` — the source window closed: the connection facts go
  (`hosts` cleared, both flags false) and the appearance and target stay, which
  is what `additional_window_target` falls back to.

`from_appearance(&InitialAppearance)` covers the existing case of a Settings
window opened when no main window is left, which used to read the appearance
global inline.

The appearance fields (`config`, `keymap`, `theme`, `shared`) are deliberately
**not** followed: what Settings edits from them is its own draft, and
re-reading them on every source notification would undo the draft under the
user's hands. That is the same contract `loaded_config` already encoded, now
stated in the type.

### Freshness

The source window notifies on every endpoint, integration and menu change, and
`App::notify` flushes its observers at the end of the update in which it
happened (`gpui-pre`'s `flush_effects`), so a snapshot is at most one turn of
the event loop behind — the same guarantee every `cx.observe` user in this app
relies on, and the reason the two tests below mutate the source in one
`cx.update` and assert in the next.

`follow` clones up to one label and one target per connected endpoint on each
notification. The window notifies often (terminal frames), so this is a few
small allocations per frame while Settings is open, and nothing at all while it
is not.

## 3. What moved

| Reach | Now reads |
| --- | --- |
| A1, A2 | `SourceWindow::of` in `SettingsWindow::new` |
| A3, A4 | `self.source_window.integrations_busy` / `.follow(owner.read(cx))` |
| A5 | `self.source_window.theme_page_open` |
| A6 (fallback) | `self.source_window.target` — replaces the `new_window_target` field |
| A7 | `self.source_window.hosts` — `host_rows()` no longer takes `cx` |
| A8 | `self.source_window.hosts` — `connected_ssh_hosts()` no longer takes `cx` |
| A9, A10 | unchanged `source` handle plus `_source` (observe) and the new `_source_released` (release) |
| — | `host_rows`, `connected_ssh_hosts` and the page's `render_sidebar` stopped taking an `App`; three tests dropped it too |

A6's *primary* read stays live on purpose: "New window" must go where the
source is now, and the existing test
`new_window_target_survives_source_close_and_tracks_live_source` pins that
(live endpoint while the window lives; the target captured at open/retarget once
it is gone). The snapshot supplies the fallback that used to be a bare field.

## 4. Behaviour deltas

One, and it is deliberate:

- **`theme_page_open` is one window's picker, not every window's.** Before, a
  theme picker open — or saving — in *any* main window stopped Settings from
  pushing its published appearance to the log window. Now only the window
  Settings was opened from fences it. With two main windows, a hover preview in
  the other one no longer holds the log window back from the appearance
  Settings published, which is the appearance Settings itself committed rather
  than that window's preview. `theme_picker` previews never reach the log
  window by another path (`menu/settings.rs:258` is the picker's own publish),
  so nothing else depends on the old breadth. If the wider fence is wanted back,
  it belongs in the app's P5 service, which is where every other
  `cx.windows()` reach in this module goes.

Everything else is behaviour-preserving, including:

- After the source closes, host rows, remote-history rows and the sound test
  button all go away as before — that is what `release()` restores, and it
  matters beyond pixels: `sync_remote_history` reads SSH hosts, and listing
  them after the source closed would have read them for a window that no longer
  exists.
- `integrations_busy` still fences retargeting: `load_integrations` and
  `poll_integrations` are the only writers of `integrations.busy` and both end
  with `cx.notify()`.

## 5. What is still app-side, by design

The six B calls. A value cannot replace them: they mutate the source window
(`load_integrations`, `queue_reload`), play through its sound worker
(`preview`, `set_volume`), or build a `Div` from its integration list, scroll
handle and selection epoch (`render_filtered_integrations`). They are P5's and,
for the integration list, part of stage 6's "keep app-side: the page passes
painted elements in".

`pages/notifications.rs:33` still upgrades the handle: it is a liveness gate on
a button whose click is itself a B call (B6), not a value read.

## 6. Validation

```sh
cargo fmt -p spaceport-gpui
cargo check -p spaceport-gpui --all-targets
cargo clippy -p spaceport-gpui --all-targets --features integration-test
cargo test -p spaceport-gpui --bin spaceport settings
```

`cargo test … settings` runs 298 tests (296 before, plus the two new ones in
`tests/source_snapshot.rs`), all passing.
