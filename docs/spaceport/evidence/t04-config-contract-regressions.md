# T04: removed settings and config regressions

Base revision: `a3dcddd6d`. The app's config filter had 253 passed, 13 failed, one ignored. Five failures came from `confirm_close_workspace` still being listed in the shipped default example even though the field and workspace-close action were removed; the parser reported it as unknown in every default-backed load. A test also expected the two unknown keys in input order, although the parser sorts them. Removing the stale example entry and matching the sorted contract left 258 passed and eight failed.

The remaining eight fixtures expected startup to reject old `[cloud.PROVIDER] setup` tables. The current parser deliberately treats removed cloud tables as inert unknown keys, and the adjacent legacy-cloud test expects ordinary settings to remain usable. An Azure fixture confirmed the observed unknown key is `cloud`. Each provider fixture now checks that startup reports `cloud` as unknown and leaves the original file byte-for-byte unchanged. These tests no longer claim a load error that does not occur.

With pinned Rust 1.99.0, `--locked --offline` and an isolated worktree, the full `config::tests::` filter passed 266 tests with one ignored. The all-target/all-feature app check passed without new source warnings. `just format-check`, `check-file-size`, `check-sorted`, `check-i18n`, and `git diff --check` passed. The test link emitted the existing macOS compact-unwind warning; Cargo reported the existing `block 0.1.6` future-compatibility notice.

This proves config parsing and file preservation in fixtures, not absence of every hidden worker or network call. A30 remains `not_run`; T04 remains blocked under G-LINK. No native desktop, Windows/Linux, or live provider check ran.
