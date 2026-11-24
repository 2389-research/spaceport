# wx60: `spaceport-palette`, the menus' stage 3

Branch `feat/wx60-palette-core`, cut from `0a2456281` (main after #225, with the
design in `docs/notes-menus-extraction.md`). This is the design's stage 3, its
seam S1: the palette's ranking model moved into a new GPUI-free crate, with
facades at the old paths. No behaviour change: no `fl!` string, no `json!`
payload and no paint line differs.

## 1. The home decision

`crates/spaceport-palette`, a sibling of `spaceport-menu`, as the design names
it. The dependency audit decides it: the moved types name no menu type — no
`Page`, no `WorkspaceAction`, no `spaceport_menu::` path anywhere in the set —
and `spaceport-menu` does not name the palette. What they do name is
`spaceport_commands::Command`, `spaceport_client::protocol::{ClientShellSnapshot,
ClientShellCommandAction, AgentStatus}`, `serde_json`, `nucleo-matcher` and
`unicode-segmentation`, all of which are already below the app. Independent, so
a separate crate is cleaner than folding the ranking into the menu model; the
design's §8 question 1 is answered the same way the split plan already answered
it for `spaceport-sidebar`.

Charter, as the AGENTS.md row now states it: **what a row is, which rows a
filter lists, how a query scores and highlights them, Go To's status filter and
heading order, the destination context a request is fenced by, the key a
frecency is remembered under, and the errors a chosen row answers with.** It
never holds an entity, a worker, a history, a scan, a word or a paint.

## 2. What moved

| From `crates/spaceport-gpui/src` | To `crates/spaceport-palette/src` |
| --- | --- |
| `palette.rs` `Filter` (`ALL`, `name`, `accepts`) | `filter.rs` |
| `palette.rs` `Action`, `Identity`, `Action::identity` | `action.rs` |
| `palette.rs` `Entry` (`new`, `with_keywords`) | `entry.rs` |
| `palette.rs` `Target` (`capture`, `validate_boot`, `workspace_exists`, `invocation`) | `target.rs` |
| `palette.rs` `rank_rows` | `rank.rs` |
| `palette/search.rs` (`Fields`, `Query`, `Highlights`, `Hit`, `matcher`, `rank`, `INLINE_CANDIDATES` and the four bounds) | `search.rs` |
| `palette/agent_filter.rs`'s `AgentFilter` (`ALL`, `name`, `key`, `accepts`, `rank`) and `headed` | `agent_filter.rs` |
| `palette/error.rs` (all 21 variants) | `error.rs`, plus `Result<T>` |
| `palette/frecency.rs`'s `Key` and its `valid`, with `MAX_FIELD_BYTES` | `frecency.rs` |
| `palette/projects.rs`'s `Project` | `project.rs` |
| `palette/project_create.rs`'s `NewProject` | `project.rs` |
| `navigation.rs` (`NavigationTarget<T>`, `OwnedNavigationTarget`, `as_deref`, `From<&NavigationTarget<T>>`) | `navigation.rs` |

Facades keep every old path: `crate::navigation::{NavigationTarget,
OwnedNavigationTarget}`, `crate::palette::{Action, Entry, Filter, Identity,
Target, rank_rows, search, Error}`, `palette::{frecency::Key,
projects::Project, project_create::NewProject, agent_filter::AgentFilter}`. No
importer changed outside the palette.

App production lines: the palette's own files went 3,853 → 3,171 (−682) and
`navigation.rs` 36 → 6; the crate holds 861 production lines across twelve
modules. `impl SpaceportWindow` blocks in the palette: 7 before, 7 after.
`fl!("` sites over it: 43 before, 43 after. `cargo fmt` reflowed nothing
structural; `git diff` shows no changed `fl!` id and no changed `json!` value.

## 3. Decisions the brief and the design left open

The brief's list named `palette.rs`'s pure types, `search.rs` and `keys.rs`'s
`PaletteKey`. Four things it did not name had to travel with them, because
leaving them behind would have meant mirror types, which AGENTS.md forbids.
Every one is the wx55 "value half moves, the reader stays" split:

- **`frecency::Key`** (and `MAX_FIELD_BYTES`, which sizes it): `Entry.frecency`
  is `Option<Key>` and `rank_rows` reads `Entry.boost`. The history stayed
  app-side — `Visit`, `Saved`, `parse`, `Frecency` (a gpui `Global`), `now`, the
  decay constants — and imports the key. `Boosts` stayed with it: nothing below
  the app asks for a points table, only for the `u32` a row carries.
- **`projects::Project` and `project_create::NewProject`**: the `Action::Project`
  and `Action::NewProject` payloads. Discovery (`collect`, `validate`,
  `Collection`, `Cancellation`, `expand_root`, `launch_root`) and the create
  path (`parse_name`, `plan`, `materialise`, `rebuild_pending`) stayed.
- **`AgentFilter`'s value and ranking half**: `rank_rows` calls `status.rank`,
  which is `search::rank` plus `headed`. `from_keystroke` stayed app-side as the
  free function `palette::agent_filter::from_keystroke`, because it takes GPUI's
  `Keystroke`, and `label()` became `palette::labels::agent_filter`, because a
  crate below the app says no sentence.
- **`palette::Error` whole, not five variants.** `Target::invocation` raises
  five of the 21, but splitting the enum would have forced either mirror
  variants or a second nesting level inside `crate::Error`. The app keeps
  `impl Localize` in `i18n/errors/palette.rs` and the `crate::palette::Error`
  facade, so every message and `Error::Palette(#[from] spaceport_palette::Error)`
  are unchanged. One variant was **added**, not moved: `UnsupportedCommand`.
  `Target::invocation` used to refuse a plugin or unknown command action with
  the app's own `crate::Error::UnsupportedCommand`, which a GPUI-free crate
  cannot name; the crate now has its own variant with the same `#[error]` text,
  and the app's `impl Localize` maps it to the same `error-unsupported-command`
  message. The app's root variant stays, because `pane_menu.rs` still raises it.
- **`navigation::NavigationTarget`** moved here — `Action::Go` and
  `Identity::Go` carry one. Chosen over a 36-line `spaceport-navigation` crate,
  over putting it in `spaceport-menu` (which would have contradicted the home
  decision above), and over leaving `Action` app-side. **Interim home:** the
  endpoint design already expects it to travel with `state/activation.rs`, and
  `navigation.rs` at the crate root says so, so a later reader does not mistake
  the ranking crate for navigation's charter.
- **`SharedString` became `String`** in `Entry`'s label, detail and badge, as
  the design's stage 3 asked: they are words the app built. Two paint lines
  changed shape and nothing else: `StyledText::new(text.clone())` on a
  `&SharedString` became `StyledText::new(text.to_owned())` on a `&str`, and
  `SharedString::from(fl!(..))` in `go_to.rs` became `fl!(..)`.
- **Visibility is what the crate needs, not everything.** `search` is the one
  public module, because the app names `search::rank`, `search::Query` and
  friends from six files; the rest are private modules with root re-exports.
  Inside it, `MAX_QUERY_CHARS`, `MAX_TERMS`, `MAX_HAYSTACK_GRAPHEMES`,
  `Query`'s pattern and `Fields::name` are `pub(crate)` only because the moved
  search tests assert on the bounds they name. `Entry::fields` is `pub(crate)`
  for the same reason; every field the app reads is `pub`.

## 4. Tests

Moved to the crate (379 test lines, `cargo test -p spaceport-palette` → 15
passing):

| From | To |
| --- | --- |
| `palette/search/tests.rs` (8 tests) | `src/tests/search.rs` |
| `palette/tests.rs`'s `invocation_uses_captured_ids_not_current_focus`, `invocation_rejects_stale_boot_command_action_and_membership`, `workspace_selection_rejects_removed_id_and_restarted_daemon` | `src/tests/target.rs` |
| `palette/agent_filter/tests.rs`'s `a_status_filter_keeps_only_agents_reporting_that_status`, `headings_return_above_their_agents_in_the_order_of_each_workspaces_best_agent` | `src/tests/agent_filter.rs` |
| `palette/tests/frecency_ranking.rs`'s `a_frecent_row_rises_among_matches_but_never_makes_a_match`, `only_the_flat_project_list_sorts_by_frecency_without_a_query` | `src/tests/rank.rs` |

Stayed app-side, unchanged: the 94 window-driven tests that drive a whole
`SpaceportWindow` (`cargo test -p spaceport-gpui --bin spaceport palette` → 94
passing), and with them `alt_and_a_navigator_letter_choose_a_filter` (it parses
GPUI keystrokes), `go_to_rows_are_remembered_by_durable_or_boot_scoped_keys`
(it calls `go_to_entries`), and `go_to_destinations_are_revalidated_against_the_current_snapshot`
(it calls `go_to::destination_exists`).

## 5. What stays, and why

`palette.rs`'s `Palette` entity (an `Entity<SearchInput>`, two `Task`s, the
nucleo `Matcher`, the scroll handle, the subscription, the `Rc<RefCell<TerminalPainter>>`),
`Selection`, `Source`, `LocalTarget`, `ProjectOperation` and every
`impl SpaceportWindow`; `render.rs`; every `fl!` site; `keys.rs`'s `PaletteKey`
and its `Direction`; the frecency history and its file; the project scan and
creation; `activate`, `project_open`, `preview`, `last_seen`, `resume`,
`go_to`.

**Deferred: `keys.rs`'s `PaletteKey::of`.** It takes `gpui::Keystroke`, which is
GPUI's own type (`gpui-pre`'s `platform/keystroke.rs`), so a GPUI-free crate
cannot name it; passing the four modifier booleans instead would be a mirror of
the keystroke the app already has. Deferred with that reason, as wx55 deferred
`menu/colors.rs` for returning `Rgba`. `Direction` is only the key's own
vocabulary and stays with it.

Also still to come from the design's stage 3 (out of this brief): `frecency`'s
store over `spaceport-state-file`, `last_seen`, `preview`'s facts, `go_to`,
`projects`/`project_create`/`canonical`'s machinery, and the design's
`tests/{agent_filter,background_ranking,frecency_ranking,go_to_search,notes,preview_layout}.rs`
rewritten as pure tests. Those are window-driven today because they build rows
through `go_to_entries` and `prepare_palette_entries`, which are the app's.

## 6. What ran

```sh
cargo check  -p spaceport-palette
cargo clippy -p spaceport-palette --all-targets
cargo test   -p spaceport-palette                       # 15 passed
cargo fmt
cargo check  -p spaceport-gpui --all-targets --features integration-test
cargo clippy -p spaceport-gpui --all-targets --features integration-test
cargo test   -p spaceport-gpui --bin spaceport palette  # 94 passed
cargo test   -p spaceport-gpui --bin spaceport -- navigation notification toast menu   # 429 passed
just check-file-size && just check-sorted && just check-i18n
```

`CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0` throughout. Not run locally:
the full workspace suite, `just ci`, the Windows/Linux lint legs, and native
palette QA — deferred to CI.
