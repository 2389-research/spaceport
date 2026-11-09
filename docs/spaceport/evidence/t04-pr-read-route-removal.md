# T04: workspace PR read route removal

Base: integration `4e9355c9c`. This cut removes the app's reachable,
read-only GitHub pull-request presentation and polling path. It does not
complete T04 or A29.

The 16 ms window tick no longer calls `update_workspace_pr`; no window
constructs or shares `pull_request::Cache`. The workspace popover has no PR
card or PR action, the sidebar rows have no PR number/unopened mark, and
Overview has no PR/checks column. The `[sidebar] change_counts` selector and
PR-specific messages are gone; an old `change_counts` key now gives an
explicit config error. The retained account popup uses its existing profile
slot and GitHub quota observation without the PR cache's separate countdown.
Sign-out and removal of saved additional-account credentials remain reachable
until the whole account state is removed. No personal credentials were read
or changed.

Local dirty and line-count badges, note and mute marks, workspace text,
popover focus, Overview agents, and native workspace dialog targeting remain.
The temporary default is local counts: the previous default was PR counts.
`sheprd-vcs::Git` still admits at most 128 listed inputs, scans for admission
at most once per second, refreshes successful probes every 30 seconds, and
uses one background worker. The PR-only ahead-probe call was removed from the
app. The final Git-read cut must remove this scanner. Legacy Close/Delete
dialog guards and their explicit Git preflight are untouched; those entire
excluded routes must go in a later coherent cut. Core document guards were
not changed.

Negative reachability: a source search of `crates/sheprd-gpui/src` found no
`pr_cache`, `update_workspace_pr`, `endpoint_pr_view`, `seed_pr`,
`WorkspaceMenuAction::PullRequest`, or `PrBadge` reference. A signed-in
headless workspace menu shows its ordinary Rename action and no `workspace-pr`
card. The config test first failed because `change_counts = "pr"` parsed,
then passed after the removal. Retained local-count, layout, account quota,
Overview, dialog and Close preflight tests passed. The broad sidebar text
test exposed a pre-existing stale “New worktree” tile expectation; the source
and locale already rendered “Open worktree...” as a row. The updated test
passed. A config error localization test exposed a pre-existing Sheprd versus
Spaceport Display mismatch in the touched error file; the one-line correction
passed the localization test.

Verified on macOS with `CARGO_INCREMENTAL=0` and the shared warm target:
`cargo +1.99.0 check -p sheprd-gpui --bin spaceport --all-targets --locked`
and the same command with `--all-features` passed with no new source
warnings. Focused `cargo +1.99.0 test -p sheprd-gpui --bin spaceport
<filter> --locked` results: config negative 1/1, local-count unit 1/1,
local chip 1/1, sidebar layouts 9/9, worktree rows 2/2, signed-in no-PR
menu 1/1, full sidebar text 1/1, titlebar quota 1/1, GitHub account 24/24,
Overview 83/83, workspace dialogs 48/48, config localization 4/4, and
Close preflight 3/3. `just format-check`, `just check-file-size`,
`just check-sorted`, and `git diff --check` passed. The test linker still
reports its inherited macOS compact-unwind warning, and Cargo still reports
`block` 0.1.6 future incompatibility. `just check-i18n` still reports
only the two clean-base findings: the literal “GitHub” in `menu/github.rs`
and unused `browser-no-pages`.
From the package root, `tools/spec.py check` passed package integrity
(not product conformance), and `tools/test_spec.py` passed 21/21 tests.

Remaining compiled code: `sheprd-github::pull_request` still includes
`cache`, `lookup`, `batch`, `fetch`, `parse`, `remote`, and `store`.
The app no longer creates a worker, but this library code is in the build.
`sheprd-vcs::git` imports its `Input`, Git process helpers, and remote
helpers; legacy `menu/workspace_close.rs` uses its repository input and
typed error; `sheprd-github` auth/HTTP use its error and clean-text helpers.
The final Git/Close/Delete and account cuts must remove these dependencies and
the compiled PR reader. The account popup and the local Git worker remain
active, so A29 stays `not_run` and T04 stays blocked on G-LINK and unfinished
excluded routes. No native UI test ran: this Mac was at `loginwindow`;
Doctor Biz can unlock it for a later acceptance pass.
