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
  Step 3a removed the separate failed-run projection from `overview.rs`.
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
3. Remove the Overview failed-run rows and their direct actions in a bounded
   cut; then remove Activity PR/Issues/Tasks, menus/commands/CLI/wire, the tick
   worker, its state and events, GitLab account UI/config, and exclusive
   assets, locales, docs, tests and backend modules as one coherent route.
   Preserve generic credentials, Core groups/tabs, terminal/chat, manual file
   actions and system URL/file opening. Audit Git status and separate PR
   actions after this cut; they are still reachable and excluded.

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

## Step 3a: Overview failed-run route

Source: isolated worktree at integration commit `ecca86a33`, with the T04
context packet from `tools/spec.py context T04`. Direct references, typed
variants, strings, saved settings, tests and crate dependencies were traced
before editing. No Core or real account endpoint was used.

The retained Overview page previously appended every failed Orchestrator run
to its host-agent rows, with a `Run` open target that jumped to Issues. That
made an excluded dispatch result visible and navigable outside the Activity
PR/Issues/Tasks sections. `overview_rows` now reads host agents only. `Open`
has only the pane target; the run icon, issue jump and unused fallback item
label are gone. The English Overview empty text no longer promises failed
runs. This does not change the retained device Overview's two-hour chart or
history in `devices_overview/chart.rs` and `history.rs`; its Core source is
still G-OBS blocked, so this cut makes no Core activity claim.

Old `overview-marks.json` files can contain pinned run marks beside agent
marks. The Run key remains a decode-only shape so the whole file still parses.
Loading filters run marks in memory, keeps agent marks, and does not rewrite
the file. The focused test failed before the filter because the old run pin
loaded. The route's typed `Open::Run` variant is deleted, so no Overview row
can navigate to Issues through that path. The active Activity sections,
`poll_orchestrators`, GitLab account flow and `sheprd-orchestrator` worker remain
reachable. T04 is still blocked by G-LINK; A29 and A30 remain `not_run`.

Review found a second saved-state edge: validation ran before the retired Run
filter. A run mark with an empty ID discarded the whole file, including valid
agent marks. A second test failed on that case. The parser now checks the
original total mark bound, filters decoded retired run marks, then validates
the retained marks. A malformed active agent mark still rejects the file,
and the tests check that loading never changes its exact bytes.

`sheprd-orchestrator` still supplies the live Activity views, GitLab account
types and error mapping; deleting the crate here would break those routes.
Its `cache` and `private_file` modules also require a separate ownership audit
before a later crate removal. The app's avatar cache is a separate module.

Verification for this cut: the new saved-mark regression failed before the
filter and passed after it. `cargo +1.99.0 test -p sheprd-gpui overview::tests
--locked --offline --quiet` passed 83 tests after the parser fix, including
retained device Overview history/chart behavior. The Core saved-group identity filter passed
3 tests; `usage::tests` passed 102 tests. No new Rust warnings appeared; the
macOS compact-unwind linker and `block` future-compatibility warnings remain.
The exact-byte saved-file assertion passed before the parser fix, and the
`overview::tests::marks` filter then passed all 9 tests, including both
malformed-key contracts. `just format-check`, `just check-file-size`,
`just check-sorted`, `git diff --check`, package `tools/spec.py check`, and
all 21 package tool tests passed. The first package command with system
Python lacked `jsonschema`; rerunning through the existing package virtual
environment passed. No native macOS window check ran: the desktop was at
the login window in this session. A29/A30 still need full negative reachability and
native-platform evidence.

## Verification for the original audit

Read-only source search, package T04 context, `tools/spec.py check`, and
`git diff --check` passed. No Cargo tests or native UI checks ran, and the
shared Cargo target was not used.
