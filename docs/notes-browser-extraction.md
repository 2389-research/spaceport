# The browser extraction: what shipped

Executes the crate half of
[`notes-browser-extraction-design.md`](notes-browser-extraction-design.md):
**design commits 3–9 and 10** of its §5 plan, on a tree where design commits
1 (app navigation out of `browser`), 2 (`GroupTerminals` out of `Browser`) and
8 (the inversion onto `BrowserHosts`) had already merged — #136 and #135.
[`notes-browser-inversion.md`](notes-browser-inversion.md) is the record of the
last of those; there is no separate note for #136, so its two commits stand as
commit 1 and 2.

Six commits, each of which compiles, passes its tests, and is reviewable alone.
Branch `wave13-extract`.

| # | Subject | Contents |
| --- | --- | --- |
| 1 | `refactor: Give a herdr-core tab's panes a crate` | `sheprd-panes` (design P1) |
| 2 | `refactor: Give the state-file writer a crate` | `sheprd-state-file` (design P2) |
| 3 | `refactor: Give the animation clock a crate` | `sheprd-motion` (design P3) |
| 4 | `refactor: Give the browser's saved state a crate` | `sheprd-browser` part 1: the types |
| 5 | `refactor: Move the browser's groups into sheprd-browser` | part 2: `Browser`, `BrowserHosts`, the 39 inverted methods |
| 6 | `docs: Record the browser extraction` | this note and `gpui-split-plan.md` |

Design commit 9's own contents — the `Browser` struct, `PageOpened`, the
`Scope`/`store`/`tab_label` free functions and `groups_view.rs`'s helpers —
landed in commit 5 with commit 7's, because once `Browser` is a foreign type
the `impl Browser` blocks may not stay behind. What is left on the window is
listed in §4.

## 1. The four extractions, measured

| Crate | From | Production lines | Tests | GPUI |
| --- | --- | --- | --- | --- |
| `crates/sheprd-panes` | `src/core_terminals/panes.rs`, `panes/error.rs` | 463 | 179 → 9 tests | no |
| `crates/sheprd-state-file` | `src/state_file.rs` | 127 | 33 → 2 tests | no |
| `crates/sheprd-motion` | `src/motion/slow.rs`, `motion.rs`'s clock | 90 | 61 + 1 → 3 tests | no |
| `crates/sheprd-browser` | `src/browser*` | **3,398** | **1,904** | **yes** |

The app keeps 1,395 production lines of browser: the façade `browser.rs` (61),
`view.rs`'s window half (190), `groups_view.rs` (635), `closed.rs`'s window half
(164), `group_settle.rs` (79), `view/render.rs` (76), `strip.rs`'s wrappers
(75), `hosts.rs` (61) and `tab_move.rs` (54). Its `view_tests` (1,329 lines)
stay as the app's integration tests for the crate, as `sound` kept
`connection/tests/sounds.rs`; 30 more test lines cover the app's own label
function.

`sheprd-browser` is the first crate in this repo with a `gpui` dependency. It
buys `cargo test -p sheprd-browser` over 1,904 test lines without building the
app, and one crate whose surface can be read in an afternoon; it removes
nothing from the app's dependency set, exactly as design §6 said.

## 2. The crate's API surface

`crates/sheprd-browser/src/lib.rs` re-exports, and that is the whole public
surface:

| Name | What it is |
| --- | --- |
| `Browser`, `BrowserHosts` | one window's groups; what a call may read of its window |
| `Store`, `Tab`, `TabId`, `Scope`, `CoreGroupKey` | the tab store and its identities |
| `Location`, `WebUrl` | what a tab shows, and a bounded address |
| `Layout`, `Slot`, `Pick`, `Shown`, `GroupId`, `GroupIds`, `SavedLayout`, `Fold` | the group algebra |
| `Layouts` | every workspace's saved layout |
| `Hidden`, `TabAppear`, `Leaving`, `Listed`, `TabBodies`, `Cells`, `Frame`, `List`, `Span`, `TabScroll`, `Thumb`, `ThumbDrag`, `GroupMotion` | the strip's state and its drawing values |
| `ClosedHerdrTab`, `ClosedTab`, `CloseRun` | what was closed, and a run of closes |
| `Error`, `Result` | six file-shaped failures and their state-file source |

The modules are `browser` (the struct), `hosts`, `pick` (what each group lists
and shows), `strip` (the strip's state calls), `groups`, `store` (+ `saved`),
`layouts`, `location`, `web_url`, `hidden`, `tab_appear`, `tab_body`,
`tab_scroll`, `group_motion`, `closed`, `close_run`, `error`.

Three things are more public than a narrow API would rather, and each is the
window's own state rather than a decision the crate makes: `Browser`'s fields
(the app reads and writes `layouts`, `saved`, `hidden`, `group_bounds`,
`herdr_tabs`, `appear`, `group_motion` and the rest directly — 15 field names),
the three methods its wrappers call (`layout`, `layout_for`, `browser_key`),
and the test hooks behind the crate's `test-support` feature.

## 3. The `test-support` feature

Four things the app's own tests drive — `TabId::test`, `GroupIds::test`-style
fixtures, `GroupMotion::enable`/`freeze`, `TabAppear::hold`/`held_start`,
`Store::restored_for_test` — were `#[cfg(test)]` in the app and stopped being
visible once they became another crate's `cfg(test)`, which is false inside
this crate's consumers' test builds. The crate carries `test-support = []` for
them, the same reason `sheprd-reorder` has one, and `sheprd-gpui`'s
dev-dependencies enable it.

## 4. What stays on the window, and why

- **The 28 `Context<Self>` methods** (786 lines), untouched by the inversion.
- **`group_shown` and `group_shows_page`**, still on the window. The inversion
  note §3 expected the `GroupTerminals` field move to let them invert; measured
  here, it does not. `group_shown` reads `group_focused_tab(group)`,
  `primary_group()` and `parked_focuses(tab)`, which are the window's
  connection state; a borrow struct carrying them would need a per-group
  focused-tab closure and a parked-tab predicate, which is `group_terminals`
  inside `BrowserHosts` — exactly what design §3.3 refused. Inverting them
  wants its own decision about that seam, not this wave's.
- **`neighbour` and `stepped`** (49 lines), with the entity methods that use
  them. `stepped` matches on the app's `controls::Command`; `neighbour` is used
  only from `show_neighbour` and `step_tab`, both `Context<Self>` methods.
- **`browser_tab_labels`**, an app function over `Browser::browser_tabs`:
  the labels a strip shows are the app's words (§5).
- **The close driver** (`close_modal/sequence.rs`, 115 lines now that `CloseRun`
  left for the crate), the element building (`view/render.rs`), and
  `view_tests`.

## 5. Deviations from the design, and why

1. **`BrowserHosts` carries the shown endpoint's `Scope`, not
   `endpoints` + `selected_endpoint`.** Design §3.2's table listed nine fields;
   the tree's is eight. The browser read `endpoints[selected_endpoint]` in
   exactly one place (`browser_key`), and `Endpoint` is the app's type — a crate
   cannot name it. One owned `Scope` replaces both fields, and nothing widened.
2. **The app writes the messages; the crate returns data.** `Tab::label`,
   `Location::drawn_title` and `Store::open`'s "New Tab" were the three
   `fl!` sites in the moved code. `Location::default_title` already returned
   English as data (`// i18n: data`), so the crate keeps that and the app adds
   `browser::drawn_label(&Tab)`, which picks the message. The six
   file-shaped `crate::Error` variants (design §4.2) moved into the crate's
   `Error`, with `error-browser.ftl` and `i18n/errors/browser.rs` beside the
   rest, and the app's root gained one `#[error(transparent)] Browser(…)`.
3. **`Store::load` and `Layouts::load` take a path.** Design §7's Option A:
   `Store::load(crate::browser::tabs_path())`, computed by
   `browser::tabs_path()`/`layouts_path()` beside `tabs_path`'s siblings in the
   app. `sheprd_client` keeps sole ownership of `state_dir`.
4. **`Browser::new` stays in the app** as the free function `new_browser(cx)`,
   not a method: `window/construct.rs` builds the browser before the window
   exists, so there is no `&self` to hang it on. The crate's `Browser::new`
   takes the saved-layouts map.
5. **`browser/code/tests.rs` (56 lines) was deleted.** It was not declared by
   any `mod` — `browser.rs` has had no `mod code` since the retired `code` tab
   kind left — so it had not compiled or run for some time. Its `WebUrl` cases
   are covered by `web_url.rs`'s own tests.
6. **Five inline `#[cfg(test)] mod tests` blocks moved to their own files**
   while they moved crates (`layouts`, `hidden`, `tab_appear`, `group_motion`,
   and `store/saved`'s), as AGENTS.md asks; of the 360 inline lines the design
   flagged, most are now files under each module's `tests/` — three inline
   blocks remain, carried over verbatim (`web_url.rs`, `tab_scroll.rs`, and
   `tab_scroll`'s nested reveal test), which a follow-up can move.

## 6. Gates, per commit

Same list for each; nothing deferred to CI except the cross-platform lints and
native UI checks, which no local run can do. `cargo test -p sheprd-gpui --bin
spaceport` (the full bin suite) ran at the three crate-boundary commits.

| Gate | panes | state-file | motion | browser 1 | browser 2 |
| --- | --- | --- | --- | --- | --- |
| `cargo check -p <new crate> --all-targets` | passed | passed | passed | passed | passed |
| `cargo check -p sheprd-gpui --all-targets` | passed, no warnings | passed | passed | passed | passed |
| `cargo test -p <new crate>` | 9 passed | 2 passed | 3 passed | 68 passed | 68 passed |
| `cargo clippy -p <new crate> --all-targets` | clean | clean | clean | clean | clean |
| `cargo fmt --all -- --check` | passed | passed | passed | passed | passed |
| `just check-file-size` | passed | passed | passed | passed | passed |
| `just check-sorted` | passed | passed | passed | passed | passed |
| `just check-i18n` | passed | passed | passed | passed | passed |
| `just check-commits` | passed | passed | passed | passed | passed |
| `cargo metadata` (lock moved) | run | run | run | run | not needed |

The full bin suite: **3,703 passed, 0 failed, 9 ignored** at the panes commit;
3,630 + 68 in `sheprd-browser` at the last, the difference being the 68 tests
that moved into the crate and the 9 + 2 + 3 that moved into the three leaf
crates. No test was dropped: 102 test attributes under `crate::browser` before
became 68 in the crate, 33 in the app's `browser*`, and 1 in `browser/tests.rs`.

Not run, and not claimed: `just ci`, `just lint-windows`/`lint-linux` (no `cfg`
gate was touched), and any native window check — this wave moves code and adds
no pixel.

## 7. What remains

- **`sidebar`** now has `Panes`, `motion` and `state_file` below it (design
  §5.2), and `browser` is out of its blocker list; it still needs `menu`,
  `keymap`, `endpoint`, `icons`, `worktree_notes` and `device_tags`, and its
  own `(usize, &Endpoint, &LiveState)` borrow.
- **`chat`** gets its 14 `browser::` references as crate paths, but still needs
  `sheprd-live`.
- **`group_shown`/`group_shows_page`** need a decision about how much of
  `group_terminals` a borrow struct may carry (§4).
- The tab strip stays in the app (design §2.5), as does `core_terminals`.
