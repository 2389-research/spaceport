# Native Git titlebar entry removal

Base: integration `71313810e`. The preceding cut removed the `git_menu`
catalog command. A source and test-reference search found the remaining
titlebar Git button, dirty counts, PR link and unopened-commit note in
`titlebar.rs`. Its click handlers were the only production callers of
`open_git_menu` and `open_git_menu_on_pull_request`; `git_unopened_commits`
served only this render path. This cut removes those controls, the dead menu
entry helpers, their window anchor state and unused locale string. It keeps
the account slot, sidebar toggle, status text and window controls in place.

The excluded button, unopened-note and repository-to-Git-menu tests were
removed. A new headless regression supplies a tracked checkout and checks
that no Git slot, button or PR link appears at 1200 and 360 pixels while the
sidebar toggle and account slot remain. The test first failed at the Git slot
assertion; after the cut it passed. The retained header test also checks
status text clears the leading controls and account slot across four widths.

The first RED compile stopped before the assertion: the shared Cargo target
marked this worktree's `sheprd-commands` as fresh while its reused metadata
still declared `GitMenu` from another worktree. `cargo metadata` resolved the
local package correctly. A scoped `cargo clean -p sheprd-commands` rebuilt
only that disposable package, then the test reached its expected failure.
Switching back from the parallel T03 build required the same scoped clean.
No source was changed to hide the build mismatch.

Focused checks on this branch passed: titlebar 33/33, Git menu component 7/7,
PR action 6/6 and group menu 26/26. The PR tests initially failed two key
cases because their adapted fixture no longer focused the menu; restoring
focus and notification in that fixture made them pass. A test-only
`seed_no_pr` helper became unused after the excluded note tests went away;
removing it cleared the new warning. `cargo +1.99.0 check -p sheprd-gpui
--all-targets --locked --offline` passed on macOS with no crate warning.
The linked tests emitted the existing macOS compact-unwind linker warning;
Cargo reported the existing `block` 0.1.6 future-compatibility note.
`just format-check`, `just check-file-size`, `just check-sorted` and
`git diff --check` passed. The package validator and all 21 package tool
tests passed; those checks cover package integrity, not application parity.
`just check-i18n` still fails at the two findings already recorded on the
preceding clean base: the `menu/github.rs:760` GitHub literal and unused
`browser-no-pages`. This cut adds no i18n finding.

This is a titlebar entry cut, not full Git negative reachability. PR action
transitions in `menu/pr_actions.rs` and commit submission in `menu/git.rs`
still set `Page::Git`; the menu chrome and rows, Git status and PR workers,
GitHub account control, sidebar state and worktree paths remain. The retained
menu tests now place their fixture on `Page::Git` directly, so their success
does not prove a user can still open that menu from the titlebar. T04 stays
blocked under G-LINK; A29 and A33 stay `not_run`. Native window, Linux and
Windows behavior were not exercised in this cut.
