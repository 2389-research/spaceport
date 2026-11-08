# T04/RM-GIT and RM-DISPATCH: worktree source and landing cut

This bounded cut starts at `b14aeac2`. It removes the desktop New Worktree
command and shortcut, workspace menu row, branch/checkout/PR/issue source
picker, its repository listings and fetch jobs, and the automatic agent setup
and launch after a checkout is created. The new-worktree cross-host choice and
job are removed from the desktop dispatch path. The removed jobs used app
account state, so tests used fixtures and did not access accounts or agents.

The ordinary workspace menu, group and tab actions, file actions, terminal and
chat paths remain. `OpenWorktree` still uses the linked repository resolver and
the correlated dialog response. The resolver also serves fan-out, which still
creates worktrees and remains a separate excluded route. The pane carry menu
still exposes `Choice::NewWorktree` in `pane_menu/carry.rs`, and
`fan_out/job.rs` still handles `WorktreeCreated`. The Teleport backend still
has `dispatch_worktree` even though this desktop dialog no longer calls it.
These are explicit remaining RM-GIT/RM-DISPATCH edges; this cut does not prove
either feature family absent.

Verification: the focused command catalog regression failed before the command
removal and passed after it. The focused headless GPUI menu regression passed;
retained OpenWorktree (10), Core workspace UI (19, with three native cases
ignored), workspace menu (56), dialog input (6), and group menu (29) filters
passed using the linked test executable. `cargo +1.99.0 check --locked -p
sheprd-gpui`, `just format-check`, `just check-file-size`, `just check-sorted`,
`git diff --check`, and `tools/spec.py check` passed. `just check-i18n` still
fails on the pre-existing unused `browser-no-pages` key in `browser.ftl`;
there are no missing or unused keys from this cut. Native UI, real accounts,
real agents, all-feature and cross-platform checks were not run.

T04 stays blocked by G-LINK. RM-GIT and RM-DISPATCH remain in progress; A29
and A30 remain not run. No Core, personal checkout, user configuration, account,
agent, or release state was changed.
