# T04/RM-GIT: checkpoint CLI and control route

At base `2a0e3c22` (control protocol 23), `sheprd checkpoint` parsed four
verbs. `list`, `create`, and `diff` ran Git-backed checkpoint code in the CLI
process; `restore` sent `WorkOp::Restore` to the running app to open its dialog.
This cut removes that command catalog, parser, help, local executor and work
wire/server route. Protocol version 24 records the removed wire operation.
The managed skill, CLI guide, and the dead example on the inherited
`website/site` page no longer teach the command.

`scripts list` still uses the local checkout-root lookup and its error
boundary. Notes, scripts, fan-out and the work socket keep their caller and
window target. The native checkpoint menu and VCS checkpoint engine still
have live UI callers and remain for a separate coherent T04 cut. The
inherited `website/site` is not a qualified Spaceport release surface; its
remaining feature table and checkpoint marketing need a later content audit.
The archival `design/sheprd-website` concepts were not changed. No user
config, checkout, daemon, Core service or website was mutated or published.

The new CLI test failed first because `checkpoint` parsed; the new wire test
failed because `restore` decoded. Both pass after the cut. The control
protocol suite passed 57/57. Focused GUI filters passed: removed CLI 1,
retained work parser 1, local scripts/notes 4, work server 8, and native
checkpoint menu 7, elapsed-time helper 3, and control-client output 7. Default and all-feature production app checks passed with
no code warnings. Format, file-size, sorted and i18n checks passed (5,066
messages in 266 files). The inherited-site test suite passed 24/24. The
existing macOS compact-unwind linker notice and Cargo `block v0.1.6`
future-compatibility notice remain. Native window, Windows/Linux builds,
full workspace tests and release packaging were not run.
The Spaceport package render/check passed and its 21 tooling tests passed;
that package check does not prove application conformance.

T04 and RM-GIT remain `in_progress`; A29/A30 remain `not_run`. This route
removal does not prove absence of native checkpoint UI, worktree/Git controls
or client VCS internals.

## Merged branch verification

Cherry-pick `365d2884` resolved the task-evidence list against the concurrent
Daytona cut and regenerated the package report. On the combined branch, the
control protocol passed 57/57; the removed CLI route 1/1, retained work client
4/4, work server 8/8, CLI parser 9/9 and native checkpoint menu 7/7 passed.
The inherited site suite passed 24/24. Default and all-feature production app
checks, format, file-size, sorted and i18n checks passed (5,066 messages in
266 files), as did all 21 specification-tool tests. No native app window or
other platform was exercised; A29 remains `not_run`.
