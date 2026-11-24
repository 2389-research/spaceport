# wx55: `spaceport-menu`, the menus' stage 1

Branch `feat/wx55-menu-core`, cut from `1b02358f8` (main after #221, with the
design in `docs/notes-menus-extraction.md`). This is the design's stage 1: the
page set, the targets, their typed requests, and the rows a menu offers, moved
into a new GPUI-free crate with facades at the old paths. No behaviour change:
no `fl!` string and no `json!` payload differs, and no line of paint moved.

## 1. What moved

| From `crates/spaceport-gpui/src` | To `crates/spaceport-menu/src` |
| --- | --- |
| `menu/page.rs` `Page`, `WorkspaceAction`, `WorkspaceMenuAction`, `pointer_anchored`, `pane_menu`, `in_header` | `page.rs` |
| `menu/workspace.rs:16-157` `WorkspaceTarget` (`new`, `group_key`, `request`), `known_worktree`, `left_checkout`, `heads_group`, `suggested_tab_name`, `suggested_workspace_name`, `chosen_label` | `workspace.rs` |
| `worktree.rs` (`checkout_root`, `left_checkout`) | `worktree.rs`, private |
| `pane_menu.rs:19-206` `Target` (`capture`, `validate`, `rename_params`), `Action`, `request`, `ACTIONS`, the filtered rows | `pane.rs`, with `Target::actions(Offers)` |
| `tab_menu.rs:22-149` `Target` (`capture`, `validate`, `validate_move`, `rename_params`), `Action`, `ACTIONS`, the filtered rows | `tab.rs`, with `Target::actions(Offers)` |
| `pane_menu.rs`/`tab_menu.rs` `Action::label` | app-side `menu/labels.rs` (`fl!`) |
| `menu/page.rs` `WorkspaceMenuAction::icon` | app-side `menu/page.rs::icon` (asset paths) |
| `pane_menu/tests.rs`, `pane_menu/tests/{moves,excluded_carry,menu_width}.rs`'s pure parts | `tests/pane_payloads.rs` |
| `tab_menu/tests.rs`'s pure parts | `tests/tab_payloads.rs` |
| `menu/workspace_tests/{naming,targets,left_checkout}.rs` pure tests | `tests/{workspace_targets,left_checkout}.rs` |

599 production lines left the app's five files (and `worktree.rs`); the crate
holds 686 production lines, the difference being public-API docs and the two
`Offers` structs. `impl SpaceportWindow` blocks: 47 before, 47 after.
`fl!("` sites over `menu/`, `palette/` and `menus.rs`: 353 before, 353 after.
No `fl!` string, `debug_selector` or `json!` payload changed.

## 2. Decisions the inventory did not settle, and what they came to

Two shapes recur: **an inherent `impl` on the moved type cannot stay app-side**
(Rust forbids it), and **anything that returns a GPUI value or names an asset
cannot enter a GPUI-free crate**.

- **`Page::accessible_role`/`accessible_name`** (`menu/page_title.rs`) were
  inherent methods on `Page`. They return `gpui::Role` and `fl!` text, so they
  became the free functions `menu::page_title::role(page)` and
  `page_title::name(page)`, at the same module path and with the same words.
  The design's inventory did not name this file; it named `page.rs` only.
- **`impl Target { fn destinations }`** (`pane_menu/moves.rs`) was an inherent
  impl on the moved `Target`. It builds `pane_move::Choice`s from `fl!` text,
  so it became `PaneMenu::destinations`, which keeps those choices in the app:
  `pane_move::Choice` did not move (design §8 question 2), and the crate's
  targets hold only the ids — `Target::actions(Offers { destinations, .. })`
  takes the fact that the picker has somewhere to list.
- **`WorkspaceMenuAction::icon`** returns `spaceport_icons::MUTED`, and
  `spaceport-icons` depends on GPUI, so it stayed app-side as
  `menu::workspace_action_icon`, called by the popover and the dialog header.
  The words for the moved actions went to `menu/labels.rs`, as the design asks.
- **`muted` on `WorkspaceTarget`** is set after `new` from client-local mute
  state, so the field stayed public along with the other target fields, as
  `spaceport-browser`'s model structs are. Behaviour lives in functions; the
  data a row draws from is read directly, so the app diff stays `use` lines and
  moved bodies.
- **Errors.** `spaceport_menu::Error` carries the six fences and refusals the
  moved code answers with (`StaleWorkspace`, `StalePane`, `StaleTab`,
  `EmptyWorkspaceLabel`, `EmptyTabName`, `LocalWorktreeNote`), with the exact
  `#[error]` text the app had, so `to_string()` — which the menus show — is
  unchanged. The app gained `Error::Menu(#[from] spaceport_menu::Error)` and
  `i18n/errors/menu.rs`, mapping each to the existing `error.ftl` message, so
  no translation changed. The app dropped the three variants no app-side code
  constructs any more (`EmptyTabName`, `EmptyWorkspaceLabel`,
  `LocalWorktreeNote`); `StalePane`, `StaleTab` and `StaleWorkspace` stayed,
  because move refusals still raise them outside the menu model.

## 3. Deferred, and why

- **`palette/search.rs`, `palette/keys.rs`'s `PaletteKey`, `palette.rs`'s
  `Filter`/`Entry`/`Action`/`Identity`/`Target`/`rank_rows`** → the design's
  stage 3 (`spaceport-palette`), not stage 1. `PaletteKey::of` takes GPUI's
  `Keystroke`, and the palette's types carry `SharedString`, frecency keys and
  `projects`/`project_create` values; they belong with the ranking crate the
  design already stages separately, and moving them now would either widen this
  crate to GPUI or split the palette in two.
- **`menu/colors.rs`** → not stage 1. Its functions return `gpui::Rgba` and mix
  with `Rgba::blend`, so no GPUI-free crate can hold them unchanged; the design
  puts them in `spaceport-settings` (S6) and leaves their home open (§8
  question 3). Moving them needs a colour representation first, which is that
  stage's work, not a move.
- **`group_menu.rs`'s `Action`** → not stage 1. Its variants carry
  `spaceport_chat_protocol::Agent` and `listening_ports::scan::Link`, and
  `Link::Tunnel` holds the tunnel `Key`; its `icon` needs `spaceport_icons`. A
  move would drag the tunnel and port-scan types into a menu crate, or replace
  the row's identity with a parallel one. It fits stage 2's picker-line work
  (S8) or a later stage, once those payloads have a home.
- **`Step`** (design §7 item 7, S4) → not stage 1. All four copies of the wrap
  arithmetic live in app key handlers (`menu/chrome/keys.rs`, the pickers, and
  `palette/keys.rs`, which is stage 3), and none of them moved here. A `Step`
  with no call sites is scaffolding; it belongs with the change that rewires
  those handlers.

## 4. Tests

Moved to the crate as pure tests (17 in all, one fixture): the workspace
request payloads and both fences, the note refusal, the suggested names and
`chosen_label`, the group key and the left-checkout identity, the pane
capture/payload/stale-snapshot cases, the pane and tab row filters, the tab
capture/rename/move fences, and the page set's three predicates. The crate's
fixture is its own compact snapshot, so it does not lean on the app's sidebar
fixture.

Stayed app-side, because they drive a window: `menu_isolation_…`, the tab
menu's composition and mute flows, every `en_xa` check, the moves through POSIX
host scripts, and the close dialogs. The English words the moved actions used
to own kept their coverage: `pane_menu::tests::the_rows_say_what_the_panes_own_state_makes_them_do`
and `tab_menu::tests::mute`'s two assertions now read `menu::labels`.

## 5. Verification

From the worktree root, with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`:

```sh
cargo check -p spaceport-menu --all-targets
cargo clippy -p spaceport-menu --all-targets
cargo test -p spaceport-menu                     # 17 passed
cargo check -p spaceport-gpui --all-targets --features integration-test
cargo clippy -p spaceport-gpui --all-targets --features integration-test
cargo test -p spaceport-gpui --bin spaceport     # 3485 passed, 10 ignored
cargo test -p spaceport-gpui --bin spaceport menu # 288 passed
cargo fmt --all --check
just check-file-size
just check-sorted
just check-i18n
```

`cargo metadata` refreshed `Cargo.lock` with the new package and the app's
dependency on it. The Sparkle framework was not needed: `check`, `clippy` and
the app's tests all build the binary without linking the updater.

Not run, and deferred to CI: the native and live checks the design's stage 4
names (macOS menu smoke, the Linux `native.rs` check), the Windows-only paths,
and the release-mode performance budget. Nothing in this change touches a
`cfg` gate except the error mapping's test list, which compiles on every
platform.
