# Confirmed Herdr TUI workspace closing

Date: 2026-11-17. Scope: `knqh` under `9s44`.

Doctor Biz clarified that Herdr 1 means Herdr TUI and its support takes priority.
He answered the explicit scope question about restoring confirmed workspace closing.
Decision D72 records that narrow exception to the earlier route removal.
Checkout deletion and other excluded Git actions remain outside this change.

## Runtime contract

The [v0.9.0 schema](https://github.com/herdrdev/herdr/blob/v0.9.0/src/api/schema/workspaces.rs)
and [v0.9.5 schema](https://github.com/herdrdev/herdr/blob/v0.9.5/src/api/schema/workspaces.rs)
define `workspace.close` with `workspace_id` and `close_group`.
The [v0.9.5 handler](https://github.com/herdrdev/herdr/blob/v0.9.5/src/app/api/workspaces.rs)
resolves the affected workspaces and shuts down detached terminal runtimes.
The [group selector](https://github.com/herdrdev/herdr/blob/v0.9.5/src/app/actions.rs)
closes linked workspaces with their sole primary workspace.
Another primary workspace keeps that close limited to its requested workspace.
A linked workspace closes independently.
The runtime removes workspace records and unattached terminals without deleting checkout files.

The [v0.9.0 key configuration](https://github.com/herdrdev/herdr/blob/v0.9.0/src/config/model.rs)
names `prefix+shift+d` as the default workspace-close chord.
Private Spaceport key configuration can override or disable that chord.
No Herdr source was edited.

## Application behavior

The workspace menu, palette, prefix chord, and `spaceport run close_workspace` share one confirmation.
The confirmation names every affected workspace and counts terminal panes.
Group termination has its own title and button.
Cancel remains selected initially.
Tab/pane confirmation preferences cannot bypass workspace consent.
Held Enter cannot confirm termination.

The captured target contains its boot, workspace IDs, labels, tab IDs, and pane membership.
Sending checks the current connection generation and selection.
It compares both displayed and newest published metadata, including metadata awaiting its surface frame.
It refuses inbox contention instead of blocking the UI thread.
The existing worker queues the exact request once and preserves its normal surface barrier.
Queue acceptance does not prove daemon acknowledgement or termination.

The CLI requires the caller's workspace on the selected host.
Named windows do not bypass daemon, workspace, or pane membership checks.
Core presentation groups are unsupported.
Exit zero means the confirmation opened; it does not mean processes ended.
The installed agent skill documents that distinction.

## Verification

The rendered-menu regression failed before the action existed.
The catalog regression failed because `close_workspace` was absent.
Independent review found a newer-inbox race.
Its regression failed when stale displayed metadata still allowed termination.
The fix checks `latest_snapshot()` in both projections.

The reviewed workspace scope passed 181 tests, with six existing opt-in tests ignored.
It includes exact requests, cancellation, held Enter, stale generations, and committed/pending metadata.
The isolated real-Herdr test closed only its test-created workspace.
It preserved checkout bytes, unrelated workspaces, the daemon, and its boot across detach/reattach.
Independent review approved target, consent, transport, and caller ownership.

Logs:

- `/tmp/spaceport-9s44-workspace-close-red.log`
- `/tmp/spaceport-9s44-workspace-command-red.log`
- `/tmp/spaceport-9s44-workspace-inbox-red.log`
- `/tmp/spaceport-9s44-workspace-reviewed-green.log`
- `/tmp/spaceport-9s44-workspace-close-live.log`

The final scoped check script passed with the validated pinned Sparkle SDK.

- Command catalog: 35 passed.
- Workspace scope: 183 passed; six existing opt-in tests ignored.
- Shared close-modal scope: 15 passed.
- Full endpoint lifecycle scope: 172 passed, including locked-inbox refusal.
- Control-server scope: 82 passed.
- Daemon key profiles: 23 passed.
- Compiled CLI discovery and app-required refusal: one passed.
- GPUI all-target/all-feature Clippy: passed with `-D warnings`.

The visible-refusal regression failed before disconnected shortcuts showed feedback.
It passes in the final workspace scope.
The generated default-shortcut fixture adds only three unbound workspace-command rows.
Its generator ran normally; no unrelated defaults changed.
Final log: `/tmp/spaceport-9s44-workspace-final.log`.
The reusable local driver is `/tmp/spaceport-9s44-workspace-check.sh`.
Full workspace and platform CI remain separate.

## Limits

Herdr's API accepts no expected revision or explicit member list.
It expands group membership when the request executes.
Local revalidation cannot eliminate a later change by another client.
This change does not claim atomic backend consent or add automatic retries.
Native workspace-dialog visuals, OS keyboard routing, and Windows runtime execution remain pending.
Doctor Biz requested that native keyboard and performance checks remain pending.
Full workspace CI remains deferred.

## Native Windows follow-up plan

Keep this follow-up inside the existing isolated runtime and UI Automation harness.
Keep the Mac foreground fixture pending.

- [x] Add real-renderer context regressions in `close_modal/tests/context.rs`.
  Text and inline-error regressions first fail with no role.
  The corrected close and departure scopes pass 105 cases, with one existing private fixture ignored directly.
- [x] Bound PowerShell commands in `tests/accessibility_windows.rs` with private capture files and exact-child cleanup.
  The timeout regression is added and compiled; actual Windows execution remains pending.
- [x] Add named-button invocation and exact foreground-window checks in `tests/support/uia_tree.ps1`.
  Refuse keyboard input when the private window does not own foreground focus.
- [x] Add a standalone scenario in `tests/accessibility_windows/workspace_close.rs`.
  Create a workspace through the real GUI and preserve a private checkout marker.
  Read its name and affected-pane count through UI Automation.
  Cancel the first dialog and confirm the second dialog with fresh keyboard input.
  Verify that the created workspace closes while the original workspace, marker, and daemon remain.
- [x] Run focused regressions, both lint configurations, and the Windows GNU checks.
  Keep native acceptance pending until the existing x64 CI job executes the scenario.
- [ ] Execute both new Windows tests in CI and inspect their actual results.
- [ ] Qualify native speech and inline-error announcements separately.

Use the repository's existing test and evidence locations instead of another scenario registry.
