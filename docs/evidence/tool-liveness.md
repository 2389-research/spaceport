# Tool liveness verification

Kata ed3g keeps unfinished calls from looking live after their owning work stops.
`Transcript::tool_live` derives liveness from existing wire states.
It checks session termination, direct activity, parent activity, and current-turn membership.
It never changes serialized tool status.

GPUI cards, row markers, expanded child steps, and the working list share that rule.
The TUI cache also tracks the liveness projection.
A status change can therefore refresh a mark without replacing its tool entry.

## Native captures

These images show actual macOS windows in isolated preview fixtures.
No daemon, agent, personal transcript, or SSH host participated.
Before images use the original GPUI renderer code from `724957c05`.
After images use the shared renderer state and marker correction.
The later parent-activity correction preserves every captured case.

The steps fixture uses Idle status with unfinished ordinary calls.
The nested fixture changes one synthetic child to InProgress beneath an ended parent.
Unrelated running background work remains visible as Running.
The temporary fixture changes were restored after capture.

| Surface and size | Before | After |
| --- | --- | --- |
| Terminal, dark wide | [Image](tool-liveness/before/terminal/dark-wide.png) | [Image](tool-liveness/after/terminal/dark-wide.png) |
| Terminal, light narrow | [Image](tool-liveness/before/terminal/light-narrow.png) | [Image](tool-liveness/after/terminal/light-narrow.png) |
| Timeline, light wide | [Image](tool-liveness/before/timeline/light-wide.png) | [Image](tool-liveness/after/timeline/light-wide.png) |
| Timeline, dark narrow | [Image](tool-liveness/before/timeline/dark-narrow.png) | [Image](tool-liveness/after/timeline/dark-narrow.png) |
| Nested steps, dark wide | [Image](tool-liveness/before/nested/dark-wide.png) | [Image](tool-liveness/after/nested/dark-wide.png) |
| Nested steps, light narrow | [Image](tool-liveness/before/nested/light-narrow.png) | [Image](tool-liveness/after/nested/light-narrow.png) |

All three cases also include the other light/dark and wide/narrow combinations in the image directories.
The native harness passed all 24 preview windows.
Inspection confirmed muted stopped marks, preserved background marks, and bounded narrow labels.
The captures do not prove live-provider event delivery or OS input behavior.

## Automated evidence

Two TUI cache regressions failed before production changes.
The actual Timeline painted-quad test also failed without the marker correction.
A parent-activity regression failed before the nested-step correction.
The final clean-member checks passed 71 protocol tests, 406 ACP unit tests, and five process/branding tests.
The GPUI chat filter passed 482 tests.
Scoped all-target Clippy passed with denied warnings.
The existing block dependency future-compatibility notice remains.

Full workspace and platform gates belong to the pull request's CI.
