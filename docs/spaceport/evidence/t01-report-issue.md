# T01: Spaceport bug-report destination

Kata `kkfs`, a bounded child of `rxkr`, addresses SP-022. Source base:
`fa5cfcab57049a5b5e3bf577bf8b44755e1a1335`, branch
`fix/spaceport-report-issue`, worktree `.worktrees/report-issue`.

The status-bar Report issue action still targeted Sheprd's issue tracker after
the fork. Its existing bug-report form also named Sheprd and linked duplicate
searches upstream. The action now derives its destination from the same
Spaceport repository constant as About and the worktree banner. The existing
form names Spaceport and searches Spaceport issues. Query field names, encoding,
bounded error text and the state captured at the click are unchanged.

## Verification

All Cargo commands used `RUSTUP_TOOLCHAIN=1.99.0`,
`HERDR_BUILD_PR_NUMBER=` and the existing dependency cache at
`.worktrees/sparkle-updates/target` as `CARGO_TARGET_DIR`.

- Baseline: `cargo test --locked -p sheprd-gpui --bin spaceport
  window::tests::report_issue` passed 13 tests.
- TDD: updated URL/form/window assertions failed in four report tests; the
  existing `sidebar_allocates_text_width` test separately failed after its real
  status-bar click opened the upstream URL. Logs:
  `/tmp/spaceport-report-issue-red.log` and
  `/tmp/spaceport-report-issue-click-red.log`.
- After the fix, `cargo test --locked -p sheprd-gpui --bin spaceport
  report_issue` passed 14 tests. The window tests exercise captured errors and
  runtime versions; URL tests retain encoding, optional-field and log bounds
  coverage. The form check reads the actual checked-in template.
- `cargo test --locked -p sheprd-gpui --bin spaceport
  sidebar_allocates_text_width` passed, including report clicks at 480px and
  800px widths. This uses headless GPUI and records the opened URL; it does not
  launch a browser or submit a report.
- `cargo clippy --locked -p sheprd-gpui --bin spaceport --tests -- -D warnings`
  passed, including type checking of the changed app and tests.
- `just format-check`, `just check-file-size check-sorted`, specification
  integrity/schema validation and `git diff --check` passed.
- Ruby's standard YAML parser accepted the actual issue form; all five
  prefilled field IDs remain present.
- Fresh-eyes review of the seven changed files found no blocker. The repository
  destination is a constant; user fields still use the existing URL encoder
  and error limit. No confirmation, credential or runtime behavior changed.

The unchanged baseline emits a macOS linker compact-unwind warning and a
future-compatibility warning for `block 0.1.6`. Neither was suppressed.

Native browser/form delivery, full workspace/platform CI, installation and
broader identity acceptance remain unproved. No issue was submitted, personal
app/runtime changed, or release published. T01/A42 and parent `rxkr` remain open.
Other inherited issue forms/contact links need their own review.
The six test/build logs are retained under this worktree's ignored
`target/report-issue/` directory. Integration and delivered-form verification
are tracked separately in Kata `kkfs`.
