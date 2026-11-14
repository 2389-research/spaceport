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
| 5 | SIFT-S09-01 command table | done, in review | #32 | One `catalog!` table generates `Command`, `COMMANDS` and the macOS and Linux defaults. A snapshot of all 107 entries matched before the old lists were removed. |
| 6 | SIFT-S16-02 live-state accessor | done, in review | #33 | `live_of` and `hosts` at 24 sites. Two loops in `overview.rs` left as they were. |
| 7 | SIFT-S16-01 endpoint phase | step 1 done, in review | #37 (on #33) | `Phase` enum; status words pinned over 288 combinations. The devices overview now agrees with the picker. Not done: fold `detached` and `outage` into one stored field; the sidebar's online tally still derives separately. |
| 8 | SIFT-S27-01 settings rows from the catalog | done, in review | #39 | Page text wins. Search now finds seven more rows. The test found three drifted rows beyond the audit's five. |
| 9 | SIFT-S33-01 spec validator gate | done, in review | #25 | `just check-spec` in `just ci` and in the always-run CI job. `validation.json` stays tracked as a dated record. |
| 10 | SIFT-S31-01 saved versus live tab schema | done, in review | #38 | Load behaviour pinned and unchanged. Retired payload validators kept so a malformed retired entry still rejects the file. |
| 11 | SIFT-S26-01 typed footer menu items | done, in review | #34 | 15 rows in one `Item` enum; selectors byte-identical. |
| 12 | SIFT-S10-01 CLI family table | done, in review | #35 | `Family { name, summary, usage, parse }`; a kept snapshot pins all help text. |
| 13 | SIFT-S11-01 port-scan shell errors | done, in review | #36 | Shell has its own error type; five provider variants removed. |
| 14 | SIFT-S22-02 chat menu and its search | done, in review | #41 | Both new tests failed on the old code: the undrawn search field kept the keyboard. |
| 16 | SIFT-S04-01 Backend trait | done, in review | #40 | `Backend` is the attach seam only. `just test-core` not run. |
| 17 | SIFT-S17-01 Core pane-tree owner | in progress | `fix/core-pane-tree-owner` (on #38) | Test first; no production change unless the two-window overwrite is proven. |
| 18 | SIFT-S14-01 build channel | done, in review | #42 | Includes the update-availability part of SIFT-S29-02. Three edits behind the `macos-updater` feature were not compiled locally; the `(macos-15, all)` CI leg is the check. |
| 19 | SIFT-S15-01 checked settings writer | in progress | `refactor/checked-settings-writer` | Round-trip test first; may stop at a smaller change. |
| 32 | SIFT-S32-01 upstream release route | done, in review | #29 (on #27) | About 4,700 lines removed, including `nix.yml` and the `sign-artifacts` action. Kept, unsure: the Windows signing helpers, `generate-changelog.sh`'s non-preview mode, `build-icon.py`'s `linux` mode. |

Ranks not listed are not started.

## Merge order

Two stacks and four independent pull requests:

- #23, then #27, then #29, then #30 (CI and release tooling).
- #28, then #31 (SSH removal).
- #33, then #37 (endpoint state).
- #38, then the Core pane-tree change if it lands.
- #24, #25, #26, #32, #34, #35, #36, #39, #40, #41 and #42 stand alone.

## Open questions for Harper

- #37: a host that is stopped but not detached now shows Connecting in the devices overview (it showed Offline). Keep, or map it to Offline?
- #42: should the signed preview offer or refresh the agent skill on first start? Should an installed preview stop reporting "Locally built app bundle" in bug reports?
- #38: should a malformed retired tab entry stop rejecting the whole saved file?
- #29: are the Windows signing helpers, `generate-changelog.sh`'s non-preview mode and `build-icon.py`'s `linux` mode wanted?
- #30: `CODEOWNERS` is now `* @harperreed`. A team instead?

## Known local noise

Five tests in `scripts/release/tests/test_sparkle_native_cache.py` and `test_sparkle_native_regressions.py` fail on the development Mac on an untouched `main` checkout ("Executable-path census returned an unexpected row"). They are not caused by this work. Nobody has checked them on a clean machine yet.

## Build note

Do not share one `CARGO_TARGET_DIR` between worktrees that build at the same time. Cargo keeps one compiled copy of each workspace crate there, not one per worktree, so a build can link another worktree's output. That produced false failures ("module `shell` is private") and could produce false passes. Give each concurrent change its own build directory, keep to two at once on this disk, and rerun key tests alone before committing.
