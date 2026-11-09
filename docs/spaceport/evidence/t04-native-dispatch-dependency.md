# T04 native dispatch dependency and saved state prerequisite

The remaining Orchestrator removal is not a single page deletion. No product
code changed in the original audit. T04 remains blocked by G-LINK; A29 and A30
remain `not_run`.

## Remaining routes after step 1

- `browser/view.rs` calls `poll_orchestrators` each window tick. The worker and
  event owner live in `orchestrator/host.rs`, `orchestrator/view.rs`, `window.rs`,
  `window/construct.rs`, and `window/monitors.rs`.
- `orchestrator/section.rs` creates a view before the first Activity click to
  count issues and pull requests. `sidebar/activity.rs` and
  `sidebar/activity/counts.rs` show and count Pull requests, Issues, and Tasks.
  `overview.rs` also reads failed runs from the same views.
- `group_menu.rs`, `menu/workspace.rs`, `menu/settings.rs`,
  `window/commands.rs`, and `sheprd-commands/src/lib.rs` still open the native
  section. `ui panel pull-requests|issues|tasks` remains in
  `cli/control/app.rs`, `control/client/help.rs`, `docs/cli.md`, and the
  `ActivityPanel` wire type in `sheprd-control-protocol/src/protocol/app.rs`.
- Old `Location::Orchestrator` tabs already disappear in
  `browser/store.rs::with_saved`; it reserves the largest saved ID first.
  `browser/store/tests/orchestrator_tabs.rs` proves only that an old tab drops
  while a page survives. A mixed Core/page/Orchestrator fixture now proves
  ID reservation and exact list/focus/close targeting. The retired parse shape
  remains in `browser/location.rs` so old files still load. Old Activity
  choices and visit stamps now fall back to Workspaces in memory.
  `control/server/tabs.rs` and `sheprd-control-protocol/src/protocol/tabs.rs`
  still report an Orchestrator tab kind, so its wire removal needs a protocol
  check.
- Step 1 removed `menu/worktree_source.rs`, `worktree_landing.rs`, and their
  dispatch/reader routes. The old source and landing evidence below records
  why they had to leave before the native Activity cut.
- `menu/gitlab.rs` and `titlebar.rs` still offer GitLab sign-in and refresh;
  `config.rs` owns its saved config. `sheprd-orchestrator` cannot be deleted by
  name while the native Activity routes and other callers remain.

## Smallest coherent cut order

1. Removed the excluded Git worktree source/landing workflow, including its
   issue/PR fetch and automatic agent launch. Extract only proven shared Core
   workspace/document primitives first; SP-015 excludes Git worktree and
   branch controls. Do not execute a real agent or GitHub/GitLab request in
   tests.
2. Verify mixed saved tab and chrome fixtures. Keep retired tab IDs reserved,
   preserve Core/page tabs and exact target behavior, and map retired Activity
   choices to a retained section without rewriting a user's saved file.
3. In one code cut, remove Activity PR/Issues/Tasks and Overview run rows,
   menus/commands/CLI/wire, the tick worker, its state and events, GitLab
   account UI/config, and exclusive assets, locales, docs, tests and backend
   modules. Preserve generic credentials, Core groups/tabs, terminal/chat,
   manual file actions and system URL/file opening. Audit Git status and
   separate PR actions after this cut; they are still reachable and excluded.

Removing just `poll_orchestrators` would leave a visible section with no data.
Before step 1, removing just the native section would have left the excluded
worktree picker and landing dispatch live. Step 2 proves saved-state behavior
before the next removal cut.

## Step 2: saved tabs and chrome

`control/server/tabs/tests/saved_tabs.rs` serializes one Core group, one web
page, and one retired Orchestrator tab with `next: 0`. It restores through the
real saved-tab parser. The CLI list shows exactly Core and page IDs, page-only
list shows only the page, and focus/close refuse the retired ID and its next
unallocated ID. Focusing Core selects its exact document/group; focusing and
closing the page target the page; closing Core leaves no survivor. A new tab
gets an ID above the retired maximum. This clarifies the existing `Store`
contract; no production tab logic changed.

`preferences.rs` maps saved `overview`, `pull-requests`, `issues`, `tasks`,
and `automations` choices and visit stamps to Workspaces. It keeps `workspaces`
and `usage` as themselves. When several old stamps map to Workspaces, the
latest timestamp wins. The tests read actual preference files and assert
the bytes stay unchanged. No user state was opened or rewritten.

Focused Cargo tests passed: `restored_tabs_keep_exact_cli_targets_and_reserve_retired_ids`
and `preferences::tests` (18 tests). The first preference test ran red before
the in-memory mapping. The subsequent save-race test exposed its prior
expectation of an Issues stamp; the corrected test asserts the mapped
Workspaces stamp and the retained Usage stamp. Native macOS verification is
unavailable because this session has no loginwindow. These tests do not prove
the later Orchestrator worker/CLI/UI deletion or A29/A30 acceptance.
`just format-check`, `just check-file-size`, `just check-sorted`,
`git diff --check`, package `tools/spec.py check`, and all 21
`tools/test_spec.py` tests passed. `cargo +1.99.0 check --locked --offline -p
sheprd-gpui --all-targets` passed without new warnings. The Cargo test build
emitted the pre-existing macOS compact-unwind linker and `block`
future-compatibility warnings.

## Verification for the original audit

Read-only source search, package T04 context, `tools/spec.py check`, and
`git diff --check` passed. No Cargo tests or native UI checks ran, and the
shared Cargo target was not used.
