# T04/RM-GIT: read probes and workspace close dependency

Source audit at integration `16987909c`. This note records remaining routes; it
does not prove native behavior. No application code, Core state, checkout, or
user configuration was changed. T04 remains blocked by G-LINK, and A29/A30
remain `not_run`.

## Remaining Git reads

- `window.rs:612-615` calls `update_merged_cleanup` and `update_git` on the
  window tick. `menu/git.rs:7-26` tracks the focused checkout and up to 128
  listed local checkouts, counts local changes when configured, and schedules
  ahead probes when GitHub is connected. It gates Git subprocesses to a
  connected owned local daemon (`menu/git.rs:32-42`).
- `sheprd-vcs/src/git.rs:58-72,174-236,255-415,472-479` still schedules
  status, dirty, count, and ahead subprocesses. `sidebar/workspaces.rs:103-148`
  reads those cached answers for dirty dots, counts, and PR badges. The
  configurable `git_status` token is another Git-specific sidebar path
  (`config/sidebar.rs:104-111,296-309`; `sidebar/tokens.rs:180-185`).
- `menu/workspace_close.rs:89-141` runs an independent fresh Git check when
  the legacy close dialog opens. `sheprd-vcs/src/git.rs:645-676` reads dirty
  files and commits not reachable from remote-tracking refs. It contacts no
  remote, but unknown results still alter consent; when confirmation is off,
  unknown can be skipped (`menu/workspace_close.rs:170-210`).
- `merged_cleanup.rs:232-245,269-291` is a second independent Git reader.
  When `remove_merged_worktrees` is enabled, a cached merged PR starts a local
  HEAD/dirty/archive probe (`merged_cleanup.rs:188-217`). This is not merely a
  badge: `merged_cleanup.rs:337-385` can send `worktree.remove` for an
  unfocused linked checkout **without a dialog**. The default setting is false
  (`config.rs:302`), but config parses it (`config.rs:699`) and Settings exposes
  it (`settings_window/pages/general.rs:242-249`). The pure decision requires
  a clean checkout, unchanged merged HEAD, no working/blocked agent, and no
  archive script for automatic removal (`merged_cleanup.rs:82-115`). A later
  daemon refusal is not forced; it is retried after backoff
  (`merged_cleanup.rs:392-429`).

## Three different close/delete effects

| Action | Request and observed guard | Effect claimed by current source |
| --- | --- | --- |
| Legacy Close / Close group | `WorkspaceTarget::request` checks boot, target, worktree and group membership, then constructs `workspace.close` with `close_group` (`menu/workspace.rs:208-246`). The dialog requires a completed Git check and, if dirty/unpushed/unknown, typed `close`, subject to `confirm_close_workspace` (`menu/workspace_close.rs:167-215`; `menu/workspace/requests.rs:289-306`). | The English dialog says it terminates running terminals and leaves checkout files/branches (`locales/en-US/workspace-dialog.ftl:27-31`). This is a legacy Herdr 1 request, not a Core presentation close. The daemon implementation was not audited here, so this is a source/UI contract, not observed daemon behavior. |
| Legacy Delete worktree | Dialog first requests `worktree.list`; it accepts exactly one matching linked, non-bare checkout and displays its path (`menu/workspace.rs:699-715`; `menu/state.rs:252-284`). Submission waits for archive lookup and uses `worktree.remove`, initially `force=false` (`menu/workspace/requests.rs:310-363`; `menu/workspace/scripts.rs:52-77`). A daemon `dirty_worktree_requires_force` refusal enables a later force confirmation (`menu/workspace/requests.rs:22-76`). An archive script, if present, runs in a new tab after trust review; removal follows only successful script execution (`worktree_scripts.rs:175-201`). | The dialog explicitly says the checkout folder is removed, force discards modified/untracked files including submodule contents, and the Herdr workspace closes; branch remains (`locales/en-US/workspace-dialog.ftl:35-40`). This is a file-deleting route. |
| Core presentation Archive / Delete | Archive toggles one document field (`core_terminals/workspace_ui.rs:644-683`; `workspace_document.rs:120-123`). Delete opens a dialog and checks target ID, connection epoch and document version before applying `Edit::Delete` by `doc.set` CAS (`workspace_ui.rs:522-546,686-783`; `workspace_document/edit.rs:57-65,79-110`). | `Document::delete` removes one grouping record from the document (`workspace_document.rs:158-168`); the UI says its terminals keep running (`locales/en-US/core-workspace.ftl:20-22`). No Git probe is involved. |

Removing only `close_status` from legacy Close would change consent for an
operation whose UI says it **terminates terminals**. Git preflight only detects
checkout state; it cannot establish that live terminal work is safe to stop.
It is therefore unsafe to replace that dialog with a no-op or an unconditional
submit. Delete worktree
and merged cleanup can remove checkout files. Keep their request/confirmation
behavior intact until each whole excluded route is removed; never retain a
background removal after dropping its preflight. Core document Archive/Delete
is the retained presentation path and must keep its exact target/version guard.

## Safe cut and tests

The next bounded cut is automatic merged-worktree cleanup: delete its tick
call, state, worker, setting/Settings control, cleanup candidate trigger, and
route-specific tests/docs in one change. First add a failing negative test
that proves a merged PR plus the legacy setting cannot queue
`worktree.remove`, and a config test that rejects the stale setting. Keep a
positive Core document test proving Archive/Delete changes only the selected
record, preserves other records, and sends no terminal kill. Inspect dependency and platform gates
before removing the crate. Later, remove legacy Close and Delete worktree
dialogs as complete routes with their Git preflight and tests; do not merely
weaken the preflight. Only then can the sidebar/PR read worker and
`sheprd-vcs` crate be cut after their remaining UI consumers are gone.

No pure primitive in `sheprd-vcs` is needed by the Core document path; its
public API is only `git` (`sheprd-vcs/src/lib.rs`). Preserve the separate
retained file interactions when removing Git: terminal file-link parsing and
click gating (`terminal/links.rs:1-74`), chat attachment selection/upload
(`chat/window.rs:729-855`), and Core `file_read` (`sheprd-core-client/src/connection.rs:314-318`).
`sheprd-github/src/checkout.rs` provides pure checkout-root tests used by
legacy worktree actions, not a generic file API. The `spaceport://open` path
still validates a `.git` checkout and may open a legacy workspace after an
explicit dialog (`link/follow.rs:232-258,281-309`;
`menu/open_checkout.rs:16-60`); it needs its own T04 route cut, not extraction
as a retained file opener.

Validation for this audit: source/path review, package-root
`tools/spec.py check` (passed, specification integrity only), and
`git diff --check`.
No Cargo, product test, native desktop, or actual daemon action was run.
