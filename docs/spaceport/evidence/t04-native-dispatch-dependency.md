# T04 native dispatch dependency at e4523bb6

The remaining Orchestrator removal is not a single page deletion. No product
code changed in this audit. T04 remains blocked by G-LINK; A29 and A30 remain
`not_run`.

## Current routes

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
  while a page survives. A new mixed Core/page/Orchestrator fixture must prove
  ID reservation and exact list/focus/close targeting. The retired parse shape
  belongs in `browser/location.rs` until that migration is verified. Old
  Activity panel choices and visit stamps in `preferences.rs` need a separate
  fallback to Workspaces or another retained panel. `control/server/tabs.rs`
  and `sheprd-control-protocol/src/protocol/tabs.rs` still report an
  Orchestrator tab kind, so its wire removal needs a protocol check.
- `menu/worktree_source.rs` still offers GitHub/GitLab issue and pull request
  rows in the excluded Git worktree source picker, using `repo_items/*` and
  account stores. `worktree_landing.rs::land` still calls
  `sheprd_orchestrator::actions::start_agent` after worktree creation.
  These are live dispatch/reader routes even if the Activity page is gone.
- `menu/gitlab.rs` and `titlebar.rs` still offer GitLab sign-in and refresh;
  `config.rs` owns its saved config. `repo_items/gitlab.rs` uses the same
  backend crate for issue lookup. `sheprd-orchestrator` cannot be deleted by
  name: these consumers need a joint removal or a proved retained owner.

## Smallest coherent cut order

1. Remove the excluded Git worktree source/landing workflow, including its
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
Removing just the native section would leave the excluded worktree picker and
landing dispatch live. This is why the current task stops at source evidence
instead of shipping a disabled or incomplete product path.

## Verification for this audit

Read-only source search, package T04 context, `tools/spec.py check`, and
`git diff --check` passed. No Cargo tests or native UI checks ran, and the
shared Cargo target was not used.
