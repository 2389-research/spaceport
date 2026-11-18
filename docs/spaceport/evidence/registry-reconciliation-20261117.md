# Registry reconciliation, 2026-11-17

This note maps the pull requests merged since the last registry update to the
acceptance scenarios they touch. The mapping itself changes no task, scenario
or gap status; decision D71, taken afterwards, is recorded under Findings.

## Inputs

- Source: `origin/main` at `98147d511` (PR #92).
- Registry: `plan/tasks.json`, `plan/acceptance.json` and `plan/gaps.json`,
  last changed on 2026-11-12.
- Pull requests: #1 to #92 in `2389-research/spaceport`, read with
  `gh pr list --state merged`.
- Kata: epic `6nht` and its 22 children.

## Method and limits

Each pull request was mapped from its title and description. Thirteen
descriptions were read in full; the rest were mapped from the title alone.
No scenario was run for this note. The mapped changes carry unit, headless or
private-fixture tests. An acceptance scenario needs its own end-to-end run
with exact binary and runtime identities, so none of them moves to `passed`.

## Registry state

| Registry | State |
| --- | --- |
| Tasks | T00 done; T01, T03, T05, T09, T10 in progress; T02, T04, T06, T07, T08, T12 blocked; T11 not started |
| Scenarios | A01, A10, A45, A47 passed; A04 blocked; 50 not run |
| Gaps | All eight open |

## Merged work by scenario

| Scenario | Pull requests |
| --- | --- |
| A03 Installed runtime startup | #22, #54, #73, #85 |
| A07 Concurrent workspace edits | #43 |
| A11 Keyboard and mouse | #45, #66, #69, #70 |
| A12 Attach flow control | #79, #83 |
| A14, A16, A17 Native chat, permissions, prompt uncertainty | #41, #47, #57, #60, #61, #64, #74, #77, #81, #88, #89 |
| A18 Unknown versus idle | #24, #80, #81 |
| A21 Desktop notifications | #53 |
| A24 Offline behavior | #33, #37, #82, #90 |
| A25 Attachments | #76, #86 |
| A28 External-only browsing | #36, #67, #71, #84 |
| A29 to A32 Excluded-feature removal | #19, #27, #28, #29, #31, #38, #40, #72 |
| A34 Customization and accessibility | #39, #50, #51, #52, #58 |
| A35, A39, A55 Command semantics and inspection | #32, #35, #48, #56, #62, #63, #70 |
| A36, A37 Stale target, timeout is not failure | #55, #59, #63, #83, #87, #88 |
| A41 Truthful diagnostics | #20, #63, #65, #77, #90 |
| A42 Identity and upstream isolation | #7, #8, #9, #18, #20 |
| A43, A44 Updates and release | #1 to #5, #10 to #15, #75 |
| A49 Native macOS acceptance | #17, #21 |
| A50 Native Windows acceptance | #6, #10 (builds paused, D66) |

No merged pull request supplies what a gap requires. #67, #71 and #84 change
the SSH tunnel route, not a Core forwarding route (G-PORT). #76 and #86
concern chat attachments, not general transfer (G-FILE).

## Findings

- Previews `v20261113.1` and `v20261117.1` are published, but this package
  has no evidence record for either. The last record is
  `t12-macos-preview-20261112.1-signed.md`.
- Decision D69 puts Herdr Core behind a disabled-by-default setting. The
  gaps G-CHAT, G-MACHINE, G-PORT and G-FILE describe Core routes. After this
  pass, decision D71 removed them from the completion gates of T02, T06, T07
  and T11. They stay open and now gate enabling Core by default (Kata
  `rnbn`). T02, T06 and T07 moved from `blocked` to `in_progress` for that
  reason alone; no scenario result changed.
- Kata `rs4c` (build, lint and test gates) was closed on 2026-11-10 without a
  comment or an evidence pointer.
- Kata `r69x` names the nebula rocket artwork, which D68 replaced.
- Kata `kkfs` is merged (`89466f193`). Its report URL names
  `2389-research/spaceport`, where Issues are enabled and `bug_report.yml`
  exists. A signed-in check of the rendered form is the only open step.
- Katas `9s44` (review findings, 25 of 53 children open) and `qnz9` (state
  machines, closed) hold the work behind #69 to #90. Neither is linked to
  `6nht`.

## Next executable steps

1. Record `v20261113.1` and `v20261117.1` in evidence files.
2. Run the scenarios that need no Core contract and no unlocked desktop,
   starting with A29 to A32 on current main.
3. Run the Herdr 1 legs of the scenarios that D71 keeps in the fork's scope.
