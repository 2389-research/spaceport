# T04 local Git reader and sidebar status removal

Source cut from integration `ece5aae07`. The work ran in a dedicated worktree.
No personal checkout, daemon, Core repository, installed app, or user state was
changed. This is source and headless evidence, not native acceptance.

## Reachability and ownership

Before the cut, `SheprdWindow::poll` called `update_git` on every tick.
`menu/git.rs` gave the focused local checkout and listed worktrees to
`sheprd-vcs::git::Git`, which scheduled status, dirty, count, and ahead jobs.
`sidebar/workspaces.rs` read that cache for dirty/count badges. Sidebar row
layouts also drew daemon-supplied `git_ahead_behind` as arrows or the
configurable `git_status` token. Legacy Close/Delete and merged cleanup had
already been removed on the base, so no remaining app action needed the VCS
crate's Git worker.

The tick call, window field, menu module, local status/count/dirty badges,
upstream arrow views, `git_status` parser and renderer, VCS error localization,
and `sheprd-vcs` workspace crate/dependency are gone. The `Cargo.lock` change
removes only the orphan package and the GPUI dependency edge. A stale
`git_status` row now fails parsing. The old `[sidebar] change_counts` key
continues to fail explicitly. The protocol snapshot field
`git_ahead_behind` remains in shared wire types for compatibility, but the
sidebar no longer reads it.

`sheprd-github` remains because `pull_request::run` still serves the explicit
project picker in `palette/project_create.rs`, `checkout` serves worktree
helpers, and `github_repo` serves repository avatars. These are separate T04
route cuts; deleting that crate here would strand live consumers. Plain branch
text, repository grouping, worktree notes, mute marks, terminal/chat drawing,
file interactions, and Core document views keep their existing paths.

## Reproduction and checks

- The new `removed_git_status_token_is_rejected_without_losing_workspace_labels`
  test failed on the base because the parser accepted `git_status`; the new
  `window_tick_has_no_git_status_poll` test failed because the tick called
  `update_git`. Both pass after the cut.
- `cargo +1.99.0 check --workspace --all-targets --all-features --locked
  --offline` passed after the source cut. A GPUI test compile initially picked
  up a stale `sheprd-control-protocol` artifact from a divergent worktree;
  scoped `cargo clean -p sheprd-control-protocol -p sheprd-gpui -p
  sheprd-commands` removed it. No source was changed to fit that artifact.
- Focused GPUI tests passed: sidebar config 5/5, token resolver 8/8, sidebar
  layouts 9/9, tick absence 1/1, workspace note/mute marks 1/1, and titlebar
  Git absence 1/1. The layout test supplies a literal
  `git/diff-looks-like-code` branch and nonzero daemon ahead/behind metadata;
  the branch renders as text without upstream, dirty, or count marks. Existing
  note and mute headless tests also passed within the broader sidebar run.
- The broader `sidebar::` filter ran 296 passed and 10 failed. Failures were
  device drag (2), device footer count (1), row drag (1), update panel (1),
  workspace menu (2), rail (2), and text-width menu (1). The text-width test
  still expects the already removed `Close group` menu action. The device
  footer and row drag failures persist when run alone. Their baseline status
  is unproved; they remain open for integration review and are not counted as
  passed.
- Python release guards for Git route and checkpoint absence passed 3/3.
  `cargo fmt --all -- --check`, `just check-file-size`, `just check-sorted`,
  `just check-i18n`, and `git diff --check` passed. `just check-i18n` reported
  3,349 used messages in 209 files. Package-root `tools/spec.py check` passed
  schema/link integrity using the integration worktree's Python venv; that
  validator does not establish application behavior. Package tool tests passed
  21/21.

The inherited macOS compact-unwind linker warning and `block` 0.1.6 future
compatibility notice remain. No native macOS check ran because `loginwindow`
owned the desktop; Windows and Linux were not run. T04 remains blocked by
G-LINK and other independent excluded routes. A29 remains `not_run` until the
full Git/IDE negative and retained native scenarios are exercised.
