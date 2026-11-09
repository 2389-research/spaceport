# T04 worktree script menu and launch removal

Source base: integration `23df7f8c0`. SP-015 excludes client automation
execution and Git/worktree controls. The earlier control cut removed
`spaceport scripts` and `work.script` but left the workspace menu and its
in-process job. This cut removes that remaining route.

## Source trace and boundary

`menu/workspace.rs` offered Run script for a repository checkout and Run setup
script for a linked checkout. `menu/workspace/scripts.rs` was the only
production caller of `start_worktree_script`. The job read
`.herdr/worktree.toml`, checked a local trust digest, requested `tab.create`,
then typed a shell line into the new pane. The old automatic setup-after-create
description had no production trigger left after the earlier creation cut;
the setup menu row was still reachable. The job, trust store, script parser,
launcher, menu page/action/labels, error messages, and current README section
are removed. No repository file, daemon, or user trust state was opened or
changed in this cut.

`gix` has no remaining source use after this deletion. Its member declaration,
root workspace pin, and lockfile stay for the integrating writer to remove
and verify together after cherry-pick; the isolated checks here remain
`--locked` against the existing dependency graph. Other script dependencies
still serve retained code.

`created_tab`, `type_script_line`, and the single reply-slot check still serve
retained chat starts and closed-agent reopen. They now live in
`window/tab_start.rs`. The parser keeps a daemon refusal as a typed error and
reports a malformed success to each caller's existing response error. The
headless workspace menu test checks main, linked, and plain checkouts and
requires retained Rename while rejecting script action variants.

## Reproduction and checks

The new negative menu test first failed 0/1: the main checkout offered
`Script(Run)` alongside Mute, Rename, Note, Open worktree, and Collapse.
Log: `/tmp/spaceport-worktree-script-red.log`. The first post-cut compile
stopped on two type-inference errors in chat and overview; explicit result
types fixed those errors. The focused menu rerun passed 1/1. The extracted
tab response tests passed 2/2, including malformed success and typed daemon
refusal. Retained chat start passed 11/11, including tab creation, fixed-line
input, and rejection after connection replacement. Retained overview passed
83/83 and tab-strip launcher passed 10/10. The GPUI all-target, all-feature
`cargo +1.99.0 check -p sheprd-gpui --all-targets --all-features --locked
--offline` passed. `just format-check`, `check-file-size`, `check-sorted`,
`check-i18n` (3,315 used messages in 206 files), `git diff --check`, and
`tools/spec.py check` passed. The inherited macOS compact-unwind linker
warning and `block v0.1.6` future-compatibility notice remain.

T04 stays blocked by G-LINK and other excluded routes. A29 and A30 remain
`not_run`; this cut does not prove full negative reachability. Native macOS,
Windows, and Linux interaction checks have not run.
