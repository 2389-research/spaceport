# T04 integrated route cuts and control contract

The integration branch combines the PR reader, folder-open, page annotation,
legacy Close/Delete, and GitHub account cuts (`9cc969af1`, `a5f6b0f81`,
`c5bd22ac8`, `9b175e099`, `1e0d2fde2`). The only source conflict was the menu
state import: both the deleted removal response parser and deleted GitHub
state were omitted. Acceptance evidence lists kept both branches' entries.
No Core repository, real checkout, personal daemon, credential, registry or
installed application was changed.

The folder-open and Archive script wire variants changed the private control
request shape. The source contract in `sheprd-control-protocol/src/protocol.rs`
requires a version increase for that change. A regression first failed at
version 29; version 30 passes. A focused malformed-request test also rejects
the old `work.script.archive` value. The earlier version-29 test now checks
that its own activity-panel contract reached at least 29. `docs/cli.md` states the
current version. This does not make old clients understand the new command set;
the existing status-version check detects a mismatch.

After a scoped clean of `sheprd-commands`, `sheprd-control-protocol`, and
`sheprd-gpui` build artifacts, default and all-feature macOS workspace
`cargo +1.99.0 check --workspace --all-targets --locked --offline` passed.
The control-protocol suite passed 60/60, the command catalog 29/29, and the
real CLI integration suite 19/19. Focused headless GPUI tests passed for the
old page `kind=send` post (1), removed legacy Close/Delete menu (1), stale
close setting (1), script line without deletion (1), neutral Profile entry
and four local routes (2), stale GitHub settings (1), Core document target and
version guards (3), layout-only group close (1), and exact Core Hub binding
(1). The new page-post test had been unrun on its source branch; it passed
here after integration. The earlier Close/Delete source branch could not get
a trustworthy build from a shared Cargo target; this single-tree build and
focused tests close that verification gap for these paths.

`just format-check`, `just check-file-size`, `just check-sorted`,
`just check-i18n`, `just check-commits`, and `git diff --check` passed.
The i18n checker found 3,352 used messages in 210 files. Six focused Python
installer tests passed with two Windows-only skips. Package-root
`tools/spec.py check --write-report` passed schema/link integrity, and
`tools/test_spec.py` passed 21/21 tool tests. These package checks do not
establish application parity. The inherited macOS compact-unwind linker
warning and `block` 0.1.6 future-compatibility notice remain; no new source
warning was observed.

The embedded page/webview and `browser.feedback`/mockup routes, local Git
scanner and specialized worktree creation, compiled GitHub PR library, and
Herdr 1 window constructor still require removal or migration. G-LINK and
the other recorded backend/release gaps remain. No native macOS acceptance
run occurred while `loginwindow` owned focus; Windows and Linux native runs
were not performed. T04 stays blocked; A28 and A29 stay `not_run`.
