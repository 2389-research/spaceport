# SIFT audit: implementation progress

This file tracks work on the findings in [the audit report](sift-audit-2026-11-13.md).
Read the report for evidence, risks and first slices. Update this file when a finding changes state.

## Rules for this work

- One branch and one worktree per finding, under `.worktrees/sift-<topic>`, cut from `origin/main`.
- Test first. Prove each new test fails for the expected reason before the change.
- Commit on the task branch, push it and open one pull request per finding (decision 6).
- Take findings in rank order. A prerequisite ships before its dependant.

## Approvals recorded

- 2026-11-13, Harper: "i think we shoudl knock through all these". Read as approval to implement the findings, including the add-device wizard deletion (SIFT-S25-01), with local commits on task branches.

## Decisions (Harper, 2026-11-13)

1. **SIFT-S32-01:** delete the upstream release route now. Keep the macOS preview route and the Windows packaging scripts that `ci.yml` still calls. Restore from history when the release gate opens.
2. **SIFT-S03-01:** remove the application-built SSH bridge (`connect_command`, `Transport::Bridge`) with the other two dead entry points, and update the two tests that pin it.
3. **SIFT-S27-01:** the page description wins for the five drifted settings. Delete the shorter search-only messages from every locale.
4. **SIFT-S32-02 remainder:** delete `nix.yml`, `cancel-merged-pr.yml`, `close-low-confidence-prs.yml` and `star-history.yml`, with the PR-closure script and its test. `flake.nix` stays.
5. **Session delete and D58:** not a violation. D58 is about not stopping the Herdr daemon behind the user's back; deleting one session the user picked and confirmed is a user action. The feature stays.
6. **Landing:** push each finished branch and open one pull request per finding, with a QA checklist. This report and progress file go in their own docs pull request.

## State

| Rank | Finding | State | Branch | Notes |
|---:|---|---|---|---|
| 1 | SIFT-S32-02 CI guard | partly done | #23 `ci/supply-chain-fork-guard`; #27 `ci/remove-upstream-only-workflows` (stacked on #23) | #23: `supply-chain.yml` runs in the fork. On its first real run both cargo-deny jobs and zizmor passed. #27: deleted `cancel-merged-pr`, `close-low-confidence-prs`, `star-history` with the PR-closure script and tests. Remaining: reduce the nine `ci.yml` guards to the fork identity and fix `check-ci.py` defaults. `nix.yml` goes with rank 32. |
| 2 | SIFT-S07-01 ACP session phase | done, in review | #24 `fix/acp-session-teardown` | `fail` and `exited` share one teardown; five new tests failed before the change and confirmed the audit's inferred defects. Then the three lifecycle fields became one `Phase` enum with no behaviour change. 387 `sheprd-acp` tests pass; fmt and clippy clean. |
| 3 | SIFT-S25-01 add-device deletion | in progress | `refactor/remove-ssh-add-device` | Approved. |
| 4 | SIFT-S03-01 dead SSH entry points | waiting on rank 3 | | Bridge removal approved (decision 2). |
| 5 | SIFT-S09-01 command table | not started | | Its tests build GPUI. |
| 6 | SIFT-S16-02 live-state accessor | not started | | |
| 9 | SIFT-S33-01 spec validator gate | done, in review | #25 `ci/spec-validator-gate` | `just check-spec` in `just ci` and in the always-run CI job. The validator passes on `main`. Evidence references must now name a file in the package. `validation.json` stays tracked: acceptance and the T00 baseline cite it as a dated record. |
| 32 | SIFT-S32-01 upstream release route | in progress | `ci/remove-upstream-release-route` (stacked on #27) | Approved (decision 1). Includes `nix.yml`. |

Ranks not listed are not started.

## Known local noise

Five tests in `scripts/release/tests/test_sparkle_native_cache.py` and `test_sparkle_native_regressions.py` fail on the development Mac on an untouched `main` checkout ("Executable-path census returned an unexpected row"). They are not caused by this work. Nobody has checked them on a clean machine yet.

## Build note

GPUI findings share one build directory, `.worktrees/.sift-target`, so only one of them builds at a time.
