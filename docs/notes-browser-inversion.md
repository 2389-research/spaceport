# The browser inversion: what shipped

Executes step one of
[`notes-browser-extraction-design.md`](notes-browser-extraction-design.md):
**commit 8 of its §5 plan**, the inversion of browser's `impl SheprdWindow`
methods onto the §3 borrow struct. The crate extraction (design commits 3–7
and 9) is a later wave; this one only makes it mechanical. Design commits 1
and 2 are *not* here — see "What is not in this wave" below.

Three commits, each of which compiles, passes its tests, and is reviewable
alone. Branch `wave10-browser`, head `d4eded2b0`.

| # | Subject | Contents |
| --- | --- | --- |
| 1 | `refactor: Read the browser's window through a borrow struct` | `browser/hosts.rs` (`BrowserHosts`, `browser_hosts`, `browser_parts`), `CoreTerminals::selected_group`, and the four `view.rs` methods inverted |
| 2 | `refactor: Invert the browser's group layout methods` | 22 group-query methods in `groups_view.rs`, plus the clock in `BrowserHosts` |
| 3 | `refactor: Invert the browser's strip methods` | 12 strip methods into the new `browser/strip.rs`, `browser_tab_labels`, plus `tab_min_width` in `BrowserHosts` |

Measured on this branch, the way design §0 measures (`impl` block extent,
docs included):

| | Before (`2e690b2da`) | After (`d4eded2b0`) |
| --- | --- | --- |
| lines in `impl SheprdWindow` blocks under `browser*` | **1,398** | 1,245 |
| lines in `impl Browser` blocks under `browser*` | 24 (`new` only) | **408** |
| methods inverted | — | **38** |
| `impl SheprdWindow` methods that take `Context<Self>` | 28 | 28 (**unchanged**) |

The 1,398 figure is exactly design §3.4's, and the 612-line half is exactly
its split: the 28 entity methods are 786 lines and the 73 methods' remaining
612 are the ones in scope. Only the design's *method count* is off — §3.4's
"92 methods, 64 without `Context<Self>`" is 73 and 45 in the tree this ran
on. That is a counting difference, not a scope one: the 612-line budget and
the 8 `impl` blocks both match, and every no-`Context<Self>` method that can
leave the window does (38 of 45; the other 7 are named below).

## 1. The borrow struct (§3.2, §3.3)

`crates/sheprd-gpui/src/browser/hosts.rs`, 93 lines:

```rust
pub(crate) struct BrowserHosts<'a> {
    core_enabled: bool,
    core_group: Option<&'a CoreGroupKey>,
    endpoints: &'a [Endpoint],
    now: Instant,
    selected_endpoint: usize,
    snapshot: Option<&'a ClientShellSnapshot>,
    tab_min_width: f32,
    tabs: Option<&'a Store>,
    workspace: Option<(Scope, String)>,
}
```

Two accessors on `SheprdWindow`, both `pub(super)`:

- `browser_hosts(&self, cx: Option<&App>)` — what one read-only call may see.
- `browser_parts(&mut self, cx: Option<&App>) -> (&mut Browser, BrowserHosts)` —
  §3.3's destructure, for the calls that change the browser. The window is
  taken apart by field (`let SheprdWindow { browser, core_terminals, endpoints,
  live, .. } = self`) so the two borrows are known to be of different fields.

`CoreTerminals::selected_group` (6 lines) names the shown herdr-core group
without going through the window, which `browser_parts` needs after it has
taken the window apart; `SheprdWindow::selected_core_group` now delegates to
it, so there is one definition.

Every window wrapper is one of two shapes, and nothing else:

```rust
pub(crate) fn group_slots(&self) -> Vec<Slot> {
    self.browser.group_slots(&self.browser_hosts(None))
}

pub(crate) fn ensure_layout(&mut self) -> Option<&mut Layout> {
    let (browser, hosts) = self.browser_parts(None);
    browser.ensure_layout(&hosts)
}
```

Consequence, and the reason this is reviewable: **not one call site outside
`browser*` changed.** `window/{commands,render,tab_strip,tab_tear,zen}.rs`,
`control/server/{tabs,hosts}/*`, `group_menu/*`, `group_terminals.rs`,
`chat/core_pane.rs`, `endpoint/lifecycle_tests/*`, `core_terminals/*` and the
window's own tests all still call `self.group_slots()`, `self.active_group()`,
`self.grow_tab(…)` and the rest, spelled as before.

## 2. The mapping, method by method

38 methods moved from `impl SheprdWindow` to `impl Browser`; each keeps an
`impl SheprdWindow` wrapper of the same name and signature, in the same file
as its body, unless the table says "private to `Browser`" (then nothing
outside `Browser` called it). "hosts" says the body reads a
`&BrowserHosts<'_>`; the others read only `Browser`'s own fields.

| Method | Now in | hosts | Wrapper |
| --- | --- | --- | --- |
| `active_group` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `browser_key` | `view.rs` | `&BrowserHosts` | `view.rs` |
| `browser_tab_ids` | `view.rs` | `&BrowserHosts` | `view.rs` (`#[cfg(test)]`) |
| `browser_tab_labels` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `drag_divider` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `drag_strip_thumb` | `strip.rs` | — | `strip.rs` |
| `ensure_layout` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `expect_new_tab_in` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `expected_new_tab_group` | `groups_view.rs` | — | `groups_view.rs` |
| `focused_herdr_tab` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `folding_groups` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `forget_browser_tabs` | `view.rs` | — | `view.rs` |
| `forget_gone_strips` | `strip.rs` | — | `strip.rs` |
| `grab_strip_thumb` | `strip.rs` | — | `strip.rs` |
| `group_lists` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `group_opened` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `group_pick` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `group_share` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `group_slots` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `group_tabs` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `groups_moving` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `grow_tab` | `strip.rs` | `&BrowserHosts` | `strip.rs` |
| `hides_tab` | `groups_view.rs` | `&BrowserHosts` | private to `Browser` |
| `hold_group_motion` | `strip.rs` | — | `strip.rs` (`#[cfg(test)]`) |
| `is_split` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `layout` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `layout_for` | `groups_view.rs` | — | `groups_view.rs` |
| `layout_key` | `view.rs` | `&BrowserHosts` | private to `Browser` |
| `leaving_tabs` | `strip.rs` | — | `strip.rs` |
| `observe_strip` | `strip.rs` | — | `strip.rs` |
| `open_workspace_tabs` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |
| `reveal_tab` | `strip.rs` | `&BrowserHosts` | `strip.rs` |
| `settle_tabs` | `strip.rs` | — | `strip.rs` (`#[cfg(test)]`) |
| `shown_pick` | `groups_view.rs` | `&BrowserHosts` | private to `Browser` |
| `strip_scroll` | `strip.rs` | — | `strip.rs` |
| `strip_thumb` | `strip.rs` | — | `strip.rs` |
| `tabs_growing` | `strip.rs` | — | `strip.rs` |
| `workspace_tabs` | `groups_view.rs` | `&BrowserHosts` | `groups_view.rs` |

The substitutions inside the moved bodies are the mechanical ones §3.4 names,
plus two the census missed:

| Was | Now |
| --- | --- |
| `self.browser.…` | `self.…` |
| `self.live.snapshot.as_ref()` | `hosts.snapshot` |
| `self.endpoints`, `self.selected_endpoint` | `hosts.endpoints`, `hosts.selected_endpoint` |
| `self.selected_core_group()` | `hosts.core_group` |
| `self.core_enabled()` | `hosts.core_enabled` |
| `self.tab_layout().min_width()` | `hosts.tab_min_width` |
| `crate::motion::now()` | `hosts.now` |
| `store(cx)`, `self.active_browser_key(cx)` | `hosts.tabs`, `hosts.workspace` |

## 3. What stays on the window, and why

- **The 28 `Context<Self>` methods** (786 lines) — untouched, byte for byte;
  a scripted compare of all 28 between `HEAD~3` and `HEAD` shows no
  difference. They are the entity glue §3.4 keeps: `cx.notify`, `cx.listener`,
  `window.focus`, and the connections they steer.
- **`group_shown` and `group_shows_page`** (no `Context<Self>`, but not
  invertible yet). `group_shown` reads `group_focused_tab`, `primary_group`
  and `parked_focuses`, which are `SheprdWindow` methods because
  `GroupTerminals` is still `Browser.terminals`. §3.3 asserts "the one browser
  site that calls them is inside an entity method anyway"; it is not — the
  site is `group_shown`, which takes `&App`. §4.3's field move (design commit
  2) makes them the window's own, and then these two invert with nothing else
  to change. Inverted anyway they would have needed `primary_group`,
  `parked_focuses` and a per-group focused-tab closure in the borrow struct,
  which would have dragged `group_terminals` into it — exactly what §4.3
  avoids.
- **`view/requests.rs` and `view/agent_tab.rs`** (5 methods, 96 lines) — §3.4
  says these are not browser's at all: control-socket targeting and window
  navigation, design commit 1's move. `control_target` and `endpoint_showing`
  also read every endpoint's `LiveState` (`self.hosts()`), which the borrow
  struct deliberately does not carry: it has the shown endpoint's snapshot
  only. They stay as they are until commit 1 moves them.
- **The strip's four getters** (§2.2) and `browser_tab_entries`
  (`view/render.rs`) — elements built with `cx.listener`.
- **`view_tests`** — 1,346 lines driving `Entity<SheprdWindow>`, unchanged.

## 4. Deviations from the design, and why

1. **`BrowserHosts` carries `workspace: Option<(Scope, String)>` and
   `core_group` (selected, not "available"), not §3.2's eight fields.** §3.2
   assumed every inverted method's wrapper could build a hosts with a window
   context, because `tabs` needs `cx`. It cannot: `group_pick`, `ensure_layout`,
   `browser_key`, `group_slots`, `is_split`, `active_group`, `group_lists`,
   `focused_herdr_tab`, `expect_new_tab_in` and `drag_divider` are called
   without an `App` (`view.group_pick(group)` in the strip, `self.ensure_layout()`
   in `group_terminals`) and their signatures may not change (§3.3). So the two
   `cx`-derived fields are `Option`s that degrade to "nothing is shown". The
   safety guarantee is not that the degraded answer equals the real one (with
   a live `App` these methods would return real tabs); it is that no wrapper
   ever passes a `None`-carrying hosts into a method whose body reads
   `hosts.tabs` or `hosts.workspace`: the tab store comes from
   `store(cx)`, itself an `Option`, and the workspace from
   `active_browser_key(cx)`, which returns `None` with no workspace or no
   available core group. `active_browser_key` stays an `impl SheprdWindow`
   method (8 lines) because it reads `core_group_available`, which needs the
   observation global; the struct carries its result instead.
2. **`now` and `tab_min_width` arrived one commit late.** Commit 1's hosts has
   neither, because nothing in `view.rs` reads them; each is added by the
   commit whose methods read it. A committed dead field is a warning, and this
   repo fixes warnings rather than allowing them.
3. **`browser/strip.rs` is a new file**, and `groups_view.rs` went from 876 to
   866 lines. Inverting in place would have left it at the 975 lines commit 2
   measured before the strip methods moved — inside the 1,000-line limit but
   one parallel PR away from it. The strip is its own responsibility already
   (`tab_appear`, `tab_scroll`, `tab_body` are its three modules), so the 12
   methods that draw a group's strip, and their wrappers, live beside them.
4. **`SheprdWindow::browser_tab_ids` is `#[cfg(test)]`.** After the inversion
   its only callers are tests (`group_menu/tests/*`,
   `core_terminals/workspace_ui/tests.rs`); `Browser::open_workspace_tabs`
   calls the `Browser` one. Before, `open_workspace_tabs` was on the window, so
   the method had a production caller.
5. **`SheprdWindow::layout_key` was deleted**, not kept as a wrapper: nothing
   outside `Browser` called it, and `Browser::layout_key` covers the rest.

## 5. Gates, per commit

Same list for all three; nothing was deferred to CI except the cross-platform
lint and native UI checks, which no local run can do.

| Gate | 1 (`724e04a16`) | 2 (`0eeb4acf8`) | 3 (`d4eded2b0`) |
| --- | --- | --- | --- |
| `cargo check -p sheprd-gpui --all-targets` | passed, no warnings | passed, no warnings | passed, no warnings |
| `cargo fmt --all -- --check` | passed | passed | passed |
| `just check-file-size` | passed | passed | passed |
| `just check-sorted` | passed | passed | passed |
| `cargo test -p sheprd-gpui --bin spaceport -- browser` | 110 passed | 110 passed | 110 passed |
| `cargo test … -- group` | — | 156 passed | — |
| `cargo test … -- tab` | — | 336 passed | 336 passed |
| `cargo test … -- zen` | — | — | 30 passed |

On the branch head, beyond the per-commit list:

- `cargo test -p sheprd-gpui --bin spaceport` — **3,709 passed, 0 failed, 9
  ignored** (the ignored ones need an external daemon or a desktop).
- `cargo clippy -p sheprd-gpui --all-targets` — clean, no findings.
- `just check-commits` — passed.
- `cargo metadata` — not run and not needed: no `Cargo.toml` or `Cargo.lock`
  change, so the lockfile did not move.

Not run, and not claimed: `just ci`, `just lint-windows`/`lint-linux` (a
`cfg` gate cannot be proven on this machine), and any native window check —
this change moves code and adds no pixel.

## 6. What is not in this wave

The design's commit plan in order, with what this branch did:

| Design commit | State |
| --- | --- |
| 1 `Move app navigation out of the browser module` | **not done** — see §3 above; `view/requests.rs` and `view/agent_tab.rs` are untouched |
| 2 `Give a window's terminal groups their own field` | **not done** — it is what keeps `group_shown`/`group_shows_page` on the window |
| 3 `sheprd-panes` | not done |
| 4 `sheprd-state-file` | not done |
| 5 `sheprd-motion` | not done |
| 6, 7 `sheprd-browser` part 1, part 2 | not done |
| **8 `Invert the browser's 64 window methods onto a borrow struct`** | **this branch**, in three commits |
| 9, 10 `Move the browser's groups into sheprd-browser`, docs | not done |

For commit 9, the crate can now take `impl Browser` as it stands except
`Browser::new(cx: &mut App)`: it reads `Layouts` from a gpui global, so it
stays in the app as construction glue (`window/construct.rs:110` builds it)
while the crate takes a saved-layouts map — the same shape `sheprd-sound`'s
`Settings::load(path)` took. Everything else in those 408 lines reads only
`self` and `BrowserHosts`, which is what the crate's own `Browser` and its
borrow struct become.
