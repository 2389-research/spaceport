# Error feedback checkpoint

Date: 2026-11-17. Finding: `kjkw`. The issue is closed after native qualification; see the receipt below.

## Implemented so far

Warning flashes now last ten seconds; success flashes retain two seconds.
Hover pauses their deadline and resumes the remaining reading time.
Hidden feedback resumes paused timers when a menu replaces its controls.
Local and selected connection notices can render an alert when the status bar is hidden.
The same alert remains available in Zen mode.
The status bar, flash, and live toast nodes now request accessible announcements.
Departing toast paint copies remain silent.

Each notice owner retains 32 entries with shared text and unique IDs.
Text stays within 4,096 UTF-8 bytes, including its truncation mark.
The window merges cloned projections by ID and shows at most 64 entries.
History remains in memory and clears when its owners disappear.
It never writes private warnings into persistent diagnostics.
The shared `messages` command opens the Recent warnings popup through the palette or CLI.
Keyboard arrows, Page Up/Down, Home, and End scroll its history.

## Verified boundaries

The original duration and hidden-status-bar regressions failed at runtime.
The hover and history regressions also failed before implementation.
The actual toast constructor lacked its Status role before the change.
The keyboard regression placed the oldest row at y1669 before scroll handling.

Four feedback tests passed in `/tmp/spaceport-9s44-feedback-final.log`.
The same run passed the actual toast-node test.
The notice filter passed 17 tests, including nine notice tests and eight icon-font tests.
Log: `/tmp/spaceport-9s44-notices-checkpoint.log`.
The notice tests cover retention, shared text, UTF-8 limits, pause/resume, and connection priority.
These checks are headless; native accessibility announcements remain unqualified.

The first all-feature lint attempt found test-only unwraps in the new history topic.
Their allowance now stays inside that test module.
The final shared lint result belongs in the task handoff after the retry.

## Selected notice display ownership

Selected worker updates preserve the display deadline only for the same notice ID.
Explicit context resets still replace the projection without inheriting its display state.
Held-input restoration retains a pause only while its notice revision remains current.
The selected endpoint's copy does not expire while its displayed notice pauses.
Hidden cards and replaced owners resume their deadlines.

Two selected-inbox tests pass, including newer-notice and explicit-reset refusal cases.
The original runtime failure is `/tmp/spaceport-9s44-notice-pause-red.log`.
Its green result is `/tmp/spaceport-9s44-notice-pause-green.log`.
The held-input regression failed before the pause field survived temporary removal.
Its green result is `/tmp/spaceport-9s44-held-notice-pause-green.log`.
All eight status-bar tests pass in `/tmp/spaceport-9s44-warning-statusbar-green.log`.
That scope includes actual card hover and opening history for both notice owners.

## Shared content feedback

A connected `w0` fixture opens and verifies a real Devices tab before raising a warning.
The warning was absent while its builder belonged to the unused Herdr TUI terminal subtree.
Runtime red: `/tmp/spaceport-9s44-all-view-flash-red.log`.
The shared content layer now mounts that builder once for every active view.
The same Devices regression passes in `/tmp/spaceport-9s44-all-view-flash-green.log`.
Departing flash copies omit live announcements and hover callbacks.
A bottom-tab regression reproduced overlap before the shared overlay reserved that strip.
Runtime red: `/tmp/spaceport-9s44-feedback-bottom-red.log`.
Both Devices tests now pass, including actual content bounds and strip clearance.
An endpoint-free Core tab awaiting its terminal also paints its warning.
That test verifies the Core placeholder before checking the warning.
It does not qualify native Core terminal glyphs or OS input.

An earlier Devices fixture had no focused workspace and never opened that view.
It was removed and contributes no verification evidence.

## Required before completion

Independent review found a visible-status-bar gap while the runtime is disconnected.
The real local configuration reload callback can raise a failure in that state.
Runtime red: `/tmp/spaceport-9s44-disconnected-warning-runtime-red.log`.
The secondary card now shows that failure while the status keeps its connection reason.
The corrected runtime test passes, and independent review approved the final predicate and strip insets.

Core-only popup capture and `messages` opening now pass scoped checks and independent review.
See [Core popup ownership](review-core-popup-ownership.md) for exact runtime and authorization limits.

The scoped script `/tmp/spaceport-9s44-feedback-scope.sh` completed successfully.
Its log is `/tmp/spaceport-9s44-feedback-scope.log`.
It passed nine status-bar, two Devices, one Core-view, and one named-window control test.
It also passed five flash, eighteen notice-filter, two toast, and twenty-one Zen tests.
All 38 command tests pass, including the regenerated shortcut fixture.
The new command adds only three unbound `messages` rows to that fixture's existing task changes.
The named-window control test opens only the requested window's history.
File-size, sorting, localization, and diff checks pass.
Quick log: `/tmp/spaceport-9s44-feedback-quick.log`.
Known output remains the existing debug-link unwind and block 0.1.6 future-compatibility notices.

The additional explicit Zen warning-card and gesture-feedback test also passes.
Log: `/tmp/spaceport-9s44-zen-warnings-green.log`.
Whole GPUI host and Windows GNU all-target, all-feature Clippy now pass with warnings denied.
Logs: `/tmp/spaceport-qnz9/2njz-host-clippy-source-stable.log` and `2njz-windows-clippy-source-stable.log`.
That checkpoint also passes aggregate formatting, size, sorting, localization, and diff checks.
Later popup-family changes still need their own final aggregate qualification.
Keep native keyboard, accessibility, visual, and release-performance checks pending.

## Original finding audit and qualification

The 2026-11-19 audit finds no remaining implementation defect in the scoped feedback paths.
The original finding required native announcement-delivery evidence; the receipt below now supplies it.
The existing UIA chrome and workspace checks do not trigger warning or toast announcements.
Headless metadata checks do not execute GPUI's native synthetic accessibility children.

The owned Windows UIA scenario triggers a benign warning and a live toast.
It verifies their displayed names with the status bar hidden and checks a distinct Zen warning.
It observes their native `LiveRegionChanged` events.
Pinned AccessKit Windows raises those events for named live nodes.
Microsoft documents the [native live-region event](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-event-ids).
This check does not require audible speech, a Mac foreground fixture, Settings, or release profiling.
Keep those broader qualifications separate from `kjkw`.

The [native announcement qualifier](review-native-error-announcements.md) now implements that focused check.
Its parser, type, static, and independent review evidence passes locally.
Its actual Windows x64 event and removal receipts now pass on source `88566686c`.
All five native UIA cases pass without failures or ignored cases.
The real hidden-status warning, Custom toast, and distinct Zen warning deliver named live-region events.
The earlier rendered tests retain the timing, pause, history, and view-ownership proof.
The combined evidence qualifies the original visible-error and announcement finding.
