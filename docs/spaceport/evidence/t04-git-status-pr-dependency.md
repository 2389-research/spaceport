# T04/RM-GIT: status worker and PR route dependency

Read-only source audit at `43031ead`. No app code, checkout, credential,
Core state or user configuration changed. T04 remains blocked by G-LINK,
RM-GIT remains in progress, and A29/A30 remain not run.

## Live routes

- Each window tick calls `update_git` in `crates/sheprd-gpui/src/window.rs`.
  `crates/sheprd-gpui/src/menu/git.rs::update_git` tracks the focused local
  checkout, probes listed local checkouts, reads ahead commits when GitHub is
  connected, polls the VCS worker, then polls PR actions. The endpoint check
  restricts this work to a connected owned local daemon. The worker and its
  scheduled status, dirty and ahead probes live in
  `crates/sheprd-vcs/src/git.rs`; its field is constructed in
  `crates/sheprd-gpui/src/window/construct.rs`.
- `crates/sheprd-gpui/src/titlebar.rs::render_git_button` shows the focused
  Git state and a cached PR, and opens the Git menu. `Command::GitMenu` also
  opens it through `crates/sheprd-commands/src/lib.rs` and
  `crates/sheprd-gpui/src/window/commands.rs`. The menu in
  `crates/sheprd-gpui/src/menu/git.rs` still offers commit, push, PR creation,
  review, comment and merge. `menu/pr_actions.rs` owns the menu's PR writes;
  `orchestrator/view/pull.rs` uses the same `pr_actions` type for another live
  PR view.
- `crates/sheprd-gpui/src/sidebar/workspaces.rs::workspace_badge` reads the
  same Git worker for dirty, local line-count and ahead marks, then combines
  them with cached PR counts. `sidebar/render.rs` passes the worker only for
  the local endpoint. `[sidebar] change_counts` reaches config, Settings and
  tests; the configurable `git_status` row token is a separate rendered
  status path. Removing only the titlebar would leave its worker and sidebar
  polling active. Removing only local counts would still leave the focused
  titlebar, menu and dirty/ahead probes active.
- `crates/sheprd-gpui/src/menu/worktree_source.rs` uses
  `repo_items::list_branches` and `repo_items::Lookup` for its branch, PR and
  Issues picker. The same file fetches a chosen PR branch and writes its
  checkout context note. `crates/sheprd-gpui/src/repo_items.rs` and its
  children carry GitHub/GitLab account and repository identity through those
  jobs. This is still reachable even if the titlebar menu goes away. The
  Activity PR/Issues and saved Orchestrator view dependencies are recorded in
  `t04-github-account-pr-dependency.md`.

## Cut boundary

No complete small Git UI/worker route is independent at this commit. A
titlebar or shortcut deletion alone leaves Git probes and writes reachable;
deleting `Git` or PR actions now would strand sidebar status, the PR menu or
the retained-at-this-stage Orchestrator PR view. A bulk `repo_items` deletion
would also cut branch selection and checkout note writing before T03's
workspace owner is settled. None of these partial cuts proves SP-015's
negative reachability.

Settle the PR/Issues/Orchestrator view and worktree picker ownership first.
Then remove the Git menu and PR writes with their commands, titlebar status,
sidebar Git marks and settings, followed by the tick call, VCS worker and
exclusive dependencies. Preserve ordinary workspace/file interaction and
plain Git or diff text in terminals and chat. Regression checks should reject
the removed menu and stale Git settings, prove no local Git probe starts on a
window tick, and retain exact Core workspace/terminal targeting and file
opening. No product test ran for this source-only audit; package checks do
not prove that runtime behavior.
