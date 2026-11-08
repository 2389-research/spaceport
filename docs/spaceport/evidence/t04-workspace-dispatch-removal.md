# T04/RM-DISPATCH: legacy New Workspace remote dispatch cut

This bounded cut starts at `f35ef24dd`. The legacy New workspace dialog no
longer offers a remote host picker or submits a client-owned Teleport job. Its
job worker, follow/setup queue, picker UI and exclusive translations are gone.
The now-unreachable `sheprd-remote::teleport::dispatch_workspace` route and its
helper are removed. The old `dispatch-history.json` file remains untouched.

The legacy dialog still submits a local Herdr 1 `workspace.create` request
against the selected workspace. That is migration work under T03, not a claim
that the local Herdr 1 route is approved Spaceport parity. The separate Core
presentation workspace create page still opens and its document edit path is
unchanged. `dispatch::Candidate`, ranking, history and host resolvers remain
because the still-live Orchestrator dispatch and pane carry read them. These
are excluded routes for later T04 cuts; the shared module cannot yet be
deleted by name. The Core machine catalog is not replaced by the old picker.

The focused headless GPUI test failed first on the old remote picker, then
passed after removal, including the Core create-page assertion. A first green
compile found an obsolete picker-only `sampled` assertion in a rank test; the
retained ranking assertion remains. A second green attempt exposed a test
fixture error: drawing while the window entity was already being updated.
The test now draws after that entity update, as the existing layout tests do.

Verification on the isolated worktree: the focused GPUI test passed 1/1;
retained Core workspace UI passed 19 with three live cases ignored, legacy
workspace menu 55/55, shared dispatch ranking/history 11/11, pane carry 6/6
and Orchestrator dispatch 2/2. With `CARGO_INCREMENTAL=0` and the shared
disposable target, `cargo +1.99.0 check --locked --offline -p sheprd-gpui
--all-targets` and the same command with `--all-features` passed after the
backend helper cut; `sheprd-remote --all-targets` passed too. The package
validator and its 21 tool tests passed, but they prove package integrity
only. Format, file-size, sorted and `git diff --check` passed. `just
check-i18n` still reports only the pre-existing unused `browser-no-pages`
key in `browser.ftl`. The existing macOS compact-unwind linker warning and
`block v0.1.6` future-compat note appeared; this cut added no warning.

No real Core, SSH host, account, agent, checkout or personal state was touched.
Native Mac/Windows/Linux UI, remote Core workspace parity and full app
acceptance were not run. T04 remains blocked by G-LINK; RM-DISPATCH remains in
progress and A29/A30 stay `not_run`.
