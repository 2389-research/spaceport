# Automatic merged-worktree cleanup removal

Base: integration `16987909c`. A window tick still scanned the read-only pull
request cache once per second when `remove_merged_worktrees` was enabled. It
probed local Git and either sent `worktree.remove` without a dialog for an
unfocused linked checkout, or opened a delete dialog for the focused checkout.
The request had its own mailbox slot and retry ledger. This remained reachable
after native PR actions and Git write controls were removed; it could close
the workspace and terminals, so the path violated the excluded Git/worktree
scope and had real data-loss consequences.

This cut removes the timed scanner, probe, request and mailbox slot, focused
auto-dialog route, setting and preference, General row, locale strings, old
feature tests, and inherited guide/example text. It retains the explicit
Delete worktree dialog, exact endpoint/boot/workspace targeting, checkout
lookup, dirty-worktree refusal and force confirmation, archive-script check,
and workspace close preflight. A stale `remove_merged_worktrees = true` key
now appears in `Config::unknown_keys`; it cannot enable a background job.

TDD: the new headless General settings test first failed at the visible
`general-merged-worktrees-toggle` assertion (1 failed). After removal, it and
the stale-setting parser test passed (2/2). The new negative reachability test
passed (2/2) and checks the absent scanner/request/setting with the retained
explicit dialog route. Focused retained delete tests passed 10/10, including
dirty refusal, force consent, and target validation. These tests do not prove
native interaction or absence in a shipped binary.

On macOS, `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/harper/Public/src/2389/spaceport-t00/target cargo +1.99.0 check --workspace --all-targets --locked --offline`
passed. The focused GPUI tests used the same environment and `--bin spaceport`.
`just format-check`, `just check-file-size`, `just check-sorted`, and
`git diff --check` passed. `just check-i18n` still fails at the two clean-base
findings: the GitHub literal in `menu/github.rs:760` and unused
`browser-no-pages`. The linker compact-unwind warning and `block` 0.1.6
future-compatibility notice are inherited. No new source warning remains.

T04 remains blocked under G-LINK and A29 stays `not_run`: other specialized
Git/GitHub/worktree routes remain. No native window, Linux, or Windows run was
performed. Package validation is recorded separately; it does not prove app
parity.
