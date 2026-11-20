<!-- ABOUTME: Records pane popup display ownership and exact process-target preservation. -->
<!-- ABOUTME: Separates headless departure checks from pending native qualification. -->

# Pane popup departure

Date: 2026-11-18. Scope: Pane, RenamePane, MovePane, PaneProcesses, and KillProcesses under `ydr4`.
Doctor Biz authorized the fix and its validation.
No commit, publication, or issue-tracker change occurred in this scope.

## Behavior and ownership

Active and departing panels share their templates.
Departure captures action labels, draft paint, destination labels, displayed process rows, and numeric offsets.
Process rows retain formatted pid text, names, commands, usage text, and selection marks.
The process worker already bounds listings to 200 rows and command text to 512 characters.
The capture retains no process identity, target, Watch, task, field entity, focus handle, or scroll handle.

Active templates resolve callbacks from the active controller.
Passive templates omit callbacks, hover handlers, and pointer cursors.
The active process list keeps its original direct row children for keyboard scrolling.
The passive list clips the same rows at the captured offset.

MenuState still drops PaneMenu during dismissal.
That immediately releases its rename field and process Watch.
The Watch still closes its worker channels and sets its existing cancellation flag.
The accepted move, rename correlation, process selection, and end-process confirmation methods remain unchanged.
The original held-Enter confirmation test remains in the consumer scope.

## Regression evidence

The actual five-page baseline lost every departing body after active reset.
Log: `/tmp/spaceport-qnz9/ydr4-pane-matrix-red.log`.
After the shared capture and templates, all five preserve identical body bounds.

The initial complete pane filter passed 28 tests and found one stale error-text expectation.
The failing expectation checked the old raw UnsupportedMethod message.
The fixture now checks the typed UnsupportedMethod result and the existing localized presentation.
Log: `/tmp/spaceport-qnz9/ydr4-pane-family-runtime.log`.
The refreshed complete `pane_menu` filter passes all 30 tests without failures or ignored cases.
Log: `/tmp/spaceport-qnz9/ydr4-pane-final-runtime.log`.

The passing initial scope includes passive controls without the parent input shield.
Old action, Rename, Cancel, Submit, and destination controls preserve replacement controller state.
The old rename field releases while its paint remains.
The destination fixture changes the boot before clicking to prevent accidental host CLI execution.
It also checks the error remains absent, so any attempted callback still fails the test.

Process cases click old row, End, Cancel, and Confirm controls against a different replacement selection.
They preserve selection, confirmation page, error, ending state, and an empty worker command channel.
Dismissal disconnects the old Watch channels before another frame renders.
Eighty displayed process rows preserve identical row and viewport bounds after active reset.
These UI tests use the existing isolated worker-channel fixture.
They do not start a real process monitor or signal those synthetic pids.

The existing worker test separately starts two exact owned child processes.
It signals only the selected child and refuses stale identity and root-process requests.
The retained child remains alive until the test explicitly kills and reaps it.
That test passed with the pane changes.
Log: `/tmp/spaceport-qnz9/ydr4-pane-owned-child.log`.

The immutable final executable is `/tmp/spaceport-9s44-gpui.v0RcOu/spaceport-tests`.
It also passes the existing live keyboard destination reveal test.
Sixty-four destination rows retain identical scrolled row and panel bounds after snapshot and target reset.

## Remaining checks

Owned formatting and diff checks pass.
Source-size, sorted-region, and localization checks pass with 2174 messages in 114 files.
Log: `/tmp/spaceport-qnz9/ydr4-pane-structure.log`.
The build reports only the existing block 0.1.6 future-compatibility notice.
Root independently reviewed the final capture, callbacks, indexed children, offsets, and cleanup.
The review approved this family without a material finding.
The review also checked the final 30-test and exact-owned-child logs.
The final aggregate lint remains pending.
Native animation, glyphs, keyboard/IME delivery, accessibility, and performance remain pending at Doctor Biz's request.
Headless geometry does not qualify native drawing or OS input.
This family does not establish completion of the whole `ydr4` issue.
