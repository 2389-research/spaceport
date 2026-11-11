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
The six test/build logs are retained under this worktree's ignored
`target/report-issue/` directory. Integration and delivered-form verification
are tracked separately in Kata `kkfs`.

## Community issue-form follow-up

Kata `zcw6`, branch `docs/spaceport-issue-forms`, starts from merged main
`89466f193c61991bc1d7a27d6299ddb22ca879c7`. The feature-request and
documentation forms now name Spaceport. Feature duplicate searches and the
Documentation contact point to Spaceport. The documentation path example uses
the existing `docs/cli.md`. Field IDs, requirements, the Herdr daemon route and
the private security-report contact are preserved.

This is a documentation-only change. Ruby's standard YAML parser accepted all
four issue-template files, with unique field IDs; both example paths exist.
Diff and link review found no unrelated changes. The earlier bug form was
verified on GitHub main by matching its blob SHA to the merged local source.
Browser automation reported its access host unavailable, so no native browser
or report submission is claimed. Full application/platform CI is separate.
The GitHub repository check found `has_issues=false`: the report destination
could not work despite correct source URLs. Issues were enabled through the
documented repository setting; an independent GET confirmed `has_issues=true`
with the repository still public. No test issue was posted. The web reader
could not fetch the new-issue form, so rendered form/prefill delivery remains
unproved. Fresh-eyes review found no blocker in the five documentation files.
The inherited private security-report destination still needs an owner-approved
Spaceport route before general production shipping.
