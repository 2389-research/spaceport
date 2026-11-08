# T04/RM-GIT and RM-DISPATCH: fan-out removal

This cut starts at `f4b68357c`. It removes the desktop fan-out menu tile,
dialog, CLI command, private control-wire request, host-script worker, saved
journal loader/writer, translations and fan-out-only tests. The control
protocol version changes from 26 to 27; an old valid `fan_out` request now
fails strict decoding. No old `fan-outs.json` file is read, changed or deleted.

The shared host picker still serves the legacy New workspace dialog. Its
single host list and ranking tests remain; lane assignment and spread code
were removed. The `fan-out.svg` asset remains because retained subagent
activity and the overview use it as an icon. `WorkspaceTarget.base` remains
because the live PR menu reads it. Open worktree, ordinary workspaces, chats,
terminals and files were not converted to a fan-out replacement.

The control-wire rejection test failed first on the old valid request and
passed after removal. `sheprd-control-protocol` passed 59/59 tests. The
focused GPUI CLI rejection test passed; from the same linked executable,
workspace menu tests passed 56/56, dispatch tests 18/18, work-control tests
5/5, CLI command tests 10/10, page-title tests 2/2 and the Core workspace UI
tests 19 passed with three Core-live cases ignored. The focused tile tests
passed 4/4. `cargo +1.99.0 check --locked --offline -p sheprd-gpui --all-targets`
and the same check with `--all-features` passed. `just format-check`,
`just check-file-size`, `just check-sorted`, and
`git diff --check` passed. The package validator and 21 package-tool tests
passed; these check package integrity, not app parity. `just check-i18n` has
only the pre-existing unused `browser-no-pages` key (present and unreferenced
at the base revision); no cut-owned missing or unused key remains.

The still-live New workspace remote dispatch, pane carry worktree creation,
PR/issue orchestration and Teleport backend host scripts remain separate
excluded routes. The inherited site assets still describe fan-out; the site
publication and identity audit is T12. Native Mac, Windows and Linux UI,
real Core/agent/account behavior, no-default builds, all-feature tests and full
application acceptance were not run. T04 remains blocked by G-LINK; RM-GIT
and RM-DISPATCH remain in progress; A29 and A30 remain `not_run`.
