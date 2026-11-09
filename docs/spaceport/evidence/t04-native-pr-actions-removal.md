# Native pull request action removal

Base: integration `0e82b8fcb`. The Git popup had review, comment and merge
rows that opened three native pages. Their handlers used the
`sheprd_github::pr_actions` worker to read a PR conversation and submit a
comment or merge. This cut removes those rows, pages, menu state, polling,
worker, action-only GraphQL `mutation` helper, error variants and messages.
The read-only PR prefetch, shared PR metadata, GitHub accounts and remaining
Git popup routes stay for separate T04 cuts. In particular, the read-only
`graphql`, `graphql_aliases` and `rest` helpers remain; `MergeMethod` stays in
the PR model because GraphQL read responses still parse available methods.

The new negative reachability test first failed because the PR action worker
existed, while its retained-owner test passed. After the cut both tests
passed (2/2). A source search found no `pr_actions`, `PrReview`, `PrComment`,
`PrMerge`, removed row/locale name, or `mutation(` reference under `crates`.
This is a source-level absence check, not a native interaction test.

On macOS, `cargo +1.99.0 check -p sheprd-gpui --all-targets --locked --offline`
passed. `cargo +1.99.0 test -p sheprd-github --locked --offline` passed
215/215, including read-only PR/cache and account tests. Focused GPUI filters
passed: `menu::git::tests` 7/7, `menu::github::tests` 26/26, and
`menu::page_title::tests` 2/2. The Git tests retain commit-dialog and PR
metadata cases; the account tests retain profile, sign-in, and selection
cases. `just format-check`, `just check-file-size`, `just check-sorted`, and
`git diff --check` passed. The package check and tool tests passed after
this evidence was linked. The existing macOS compact-unwind linker warning
and `block` 0.1.6 future-compatibility notice appeared. `just check-i18n`
still fails at the two clean-base findings: the `menu/github.rs:760` GitHub
literal and unused `browser-no-pages`; this cut added no finding.

The Git popup still has commit, push, and create/open PR rows; its Git status
worker and `sheprd-vcs` actions remain. The GitHub account/profile page and
read-only PR prefetch remain. Earlier T04 titlebar and command cuts removed
some entry routes, but retained workers and internal menu routes still need
their own coherent deletion. T04 stays blocked under G-LINK; A29 and A33
stay `not_run`. No native window, Linux, or Windows interaction was tested.
