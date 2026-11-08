# T04/RM-GITHUB: native Security section removal

At base `f11c641c`, `SheprdWindow::tick` called `poll_security` every tick.
`security.rs` read GitHub Dependabot alerts and private reports, cached them
under `sheprd_orchestrator::cache::Kind::Security`, and drew a Security Activity
page with search, detail panes and actions. `ActivityPanel::Security` also
exposed that page through `ui panel security` and saved its visit time.

This cut removes the whole Security reader, timer, cache kind, page, controls,
error catalog, locale text and UI control destination. The control protocol
advances from v24 to v25. Old `activity_panel` and `activity_last_opened_at`
Security values are ignored by the existing tolerant preferences parser;
the cut does not rewrite personal files. A source audit found no remaining
`poll_security`, `Kind::Security` or `ActivityPanel::Security` reference in
the application graph. The shield icon remains because chat settings uses it.
Ordinary terminal/chat advisory text and clicked local file links remain.

The new protocol test failed first because `security` still parsed as a panel.
It now rejects the old panel name and wire request while round-tripping a
retained Overview request with the same explicit window target. The cache
test still proves repository entries survive replacement, another connection
and removal. Existing headless tests in `titlebar/tests.rs`,
`sidebar/activity/tests/{layout,sections}.rs` and
`control/server/app/tests/ui_panel.rs` cover the profile avatar and retained
PR/Issues Activity navigation; an absence assertion was added beside the
Activity layout check. A focused GUI test compile started when 12 GiB was
free, then was interrupted at 8.5 GiB before linking under the shared 8 GiB
disk floor. No GUI test ran in this worktree; those render assertions remain
for the merged check.

Checks run: control-protocol suite passed 58/58, including the focused
negative wire test; focused orchestrator
cache test passed 1/1; default `cargo check --locked -p sheprd-gpui` passed
without new code warnings; `just format-check`, `check-file-size`,
`check-sorted` and `check-i18n` passed. Cargo still reports the existing
`block v0.1.6` future-compatibility notice. Native GUI, Windows/Linux,
all-feature app and full workspace checks were not run here. No account,
network service, daemon or personal configuration was accessed.

The Spaceport package render/check passed schema and local link validation;
its 21 tool tests passed. A plain system Python check lacked the package's
JSON Schema dependency, so validation used the existing package virtual
environment. The package report covers spec integrity, not application QA.

On the integration branch, the default and all-feature app checks passed;
the control-protocol suite passed 58/58. Headless Activity layout passed 6/6,
and the retained titlebar account-icon, profile-slot and UI-panel tests passed
1/1, 1/1 and 2/2. The first retained Activity sections run found a stale
embedded-page expectation: a repository URL correctly opened the system
browser and left Issues visible. Commit `f92f39ce` corrected the test to
assert both the opened URL and unchanged section; the focused sections suite
then passed 4/4. `just format-check`, `check-file-size`, `check-sorted` and
`check-i18n` passed on the merged cut. The test linker emitted the existing
compact-unwind size warning, and Cargo retained the `block v0.1.6`
future-compatibility notice. No native window or external service was used.

GitHub profile sign-in/polling, PR and Issues Activity pages, repository
workers, provider account dependencies and their credentials remain reachable.
The generic credential primitives and updater release transport are retained.
T04 stays blocked on G-LINK; A29 and A30 remain `not_run`. This slice alone
does not prove the full GitHub exclusion or release readiness.
