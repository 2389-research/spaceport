# Native Git mutation removal

Base: integration `92e73dc3d`. The inherited Git popup still offered commit,
push, and create/open pull request rows after the titlebar and CLI routes were
removed. Its `sheprd-vcs::git::Git` worker still accepted `Job::Run` and could
stage, commit, push, and submit a GraphQL pull request mutation. This cut removes
the popup pages and state, write action API, worker job/completion, write
functions, action-only errors, and their messages. No write job remains behind
the closed UI.

The worker still has live read-only status, dirty, line-count, and ahead probes
for sidebar display and PR prefetch. `close_status` stays for the workspace
close consent check; it reads dirty and unpushed state without contacting a
remote. The GitHub account page and read-only PR model/cache stay for later T04
cuts. The shared `git`/`git_bytes` process helpers stay because read probes
call them. Native Git mutation tests were removed; retained real-repository
close and probe tests still run. Read-only PR/cache tests live in the GitHub
and workspace PR modules. This is a bounded removal, not full Git/GitHub
negative reachability.

TDD: `test_git_mutation_absence.py` first failed 12 write-route subtests on the
base while its retained-route test passed. After the cut it and the prior
PR-action absence test pass (4/4). A source search found no `Page::Git`,
`GitCommit`, `git::Action`, `Job::Run`, or `Completion::Action` production
references in GPUI/VCS. The test guards source reachability; it does not prove
native interaction.

On macOS, `cargo +1.99.0 check --workspace --all-targets --locked --offline`
passed with no new source warning. `cargo +1.99.0 test -p sheprd-vcs --locked
--offline` passed 20 unit tests and one doctest, including a temporary
repository with staged, unstaged, untracked, and unpushed close-check states.
Focused GPUI test filters passed: `menu::git::tests` 1/1,
`menu::workspace_close::tests` 3/3, `sidebar::tokens::tests` 8/8,
`menu::appear::tests` 13/13, and `menu::page_title::tests` 2/2, all under
`cargo +1.99.0 test -p sheprd-gpui --locked --offline <filter>`.
`just format-check`, `just check-file-size`, `just check-sorted`, and
`git diff --check` passed. The macOS compact-unwind linker warning and
`block` 0.1.6 future-compatibility notice are inherited.
`just check-i18n` still reports only the two clean-base findings:
`menu/github.rs:760` GitHub literal and unused `browser-no-pages`.
Scoped clippy for VCS and GPUI all targets with `-D warnings` stopped in the
unchanged `sheprd-client/src/discovery.rs:120` at
`clippy::match_like_matches_macro`; it did not reach this cut's crates. The
same failure was reproduced with `cargo +1.99.0 clippy -p sheprd-client
--all-targets --locked --offline -- -D warnings` in the clean integration
worktree at the exact base commit `92e73dc3d` (exit 101); no integration
source was changed.
The package `tools/spec.py check --write-report` passed with schema validation,
and `tools/test_spec.py` passed 21/21. These package results show integrity,
not desktop behavior.

T04 stays blocked under G-LINK; A29 stays `not_run` because specialized
Git/IDE and other excluded routes remain elsewhere. This cut did not run a
native window, Linux, or Windows interaction.
