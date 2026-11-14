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
| 1 | SIFT-S32-02 CI guard | done, in review | #23, #27 (on #23), #30 (on #29) | #23: `supply-chain.yml` runs in this repository; cargo-deny and zizmor passed on their first real run. #27: deleted `cancel-merged-pr`, `close-low-confidence-prs`, `star-history` with the PR-closure script and tests. #30: every `ci.yml` guard names only this repository; the Windows pause is the repository variable `WINDOWS_ENABLED`; `check-ci.py` defaults to this repository; `CODEOWNERS` names `harperreed` (it named the upstream maintainer). |
| 2 | SIFT-S07-01 ACP session phase | done, in review | #24 | `fail` and `exited` share one teardown; five new tests failed before the change and confirmed the audit's inferred defects. Then the three lifecycle fields became one `Phase` enum with no behaviour change. |
| 3 | SIFT-S25-01 add-device deletion | done, in review | #28 | About 4,650 lines removed, including `setup_relay`. 3,791 GPUI tests pass. The remaining claim error is renamed `Busy`. Not done: `crates/sheprd-gpui/README.md` still describes the wizard. |
| 4 | SIFT-S03-01 dead SSH entry points | done, in review | #31 (on #28) | About 1,600 lines removed, including the bridge; `Transport` is gone. Kept as live: `set_login_config_home`, `local_network_access`, the WSL probe helpers. Kept, unsure: `HostProbe::SshFailed`, `login_env::probe_for_tests`. |
| 5 | SIFT-S09-01 command table | in progress | `refactor/command-table` | |
| 6 | SIFT-S16-02 live-state accessor | in progress | `refactor/endpoint-live-accessor` | |
| 9 | SIFT-S33-01 spec validator gate | done, in review | #25 | `just check-spec` in `just ci` and in the always-run CI job. `validation.json` stays tracked as a dated record. |
| 32 | SIFT-S32-01 upstream release route | done, in review | #29 (on #27) | About 4,700 lines removed, including `nix.yml` and the `sign-artifacts` action. Kept, unsure: the Windows signing helpers, `generate-changelog.sh`'s non-preview mode, `build-icon.py`'s `linux` mode. |

Ranks not listed are not started.

## Merge order

Two stacks and four independent pull requests:

- #23, then #27, then #29, then #30 (CI and release tooling).
- #28, then #31 (SSH removal).
- #24, #25 and #26 stand alone.

## Known local noise

Five tests in `scripts/release/tests/test_sparkle_native_cache.py` and `test_sparkle_native_regressions.py` fail on the development Mac on an untouched `main` checkout ("Executable-path census returned an unexpected row"). They are not caused by this work. Nobody has checked them on a clean machine yet.

## Build note

GPUI findings share one build directory, `.worktrees/.sift-target`, so only one of them builds at a time.
