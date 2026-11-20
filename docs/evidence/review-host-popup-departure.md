<!-- ABOUTME: Records host popup paint ownership and exact target preservation. -->
<!-- ABOUTME: Separates headless departure checks from pending native qualification. -->

# Host popup departure

Date: 2026-11-18. Scope: Host, Rename Device, Forward Port, Device Tags, and Remove Device under `ydr4`.
The completed Device Theme family has [separate evidence](review-device-theme-departure.md).
Doctor Biz authorized this fix and its validation.
This owner made no commit, push, release, or Kata change.

## Behavior and ownership

The active and outgoing copies share their display templates.
The outgoing copy holds labels, checkbox flags, status text, input paint, and numeric scroll offsets.
It retains no input entity, focus handle, subscription, task, action closure, or tunnel handle.
Frozen rows, close controls, tag choices, forward buttons, and confirmation buttons install no handlers.
They cannot act on a replacement popup without a parent input shield.

Host actions and forwarded-port summaries remain bounded by their existing domain limits.
The forward manager still limits the window to 32 forwards.
The capture preserves the forwards width decision and wrapping of ended-forward reasons.
Device Tags captures only visible rows, with numeric spacers for the omitted rows.
A single live list measurement supplies its pixel-snapped uniform row extent.
The capture retains no geometry cell or per-row rectangle ledger.

MenuState still drops HostMenu during dismissal.
That releases rename, port, and tag fields immediately.
The live shell scroll handle and tag measurement cell also drop with their owner.
The existing Device Theme task and subscription cleanup remains separate and unchanged.

Removal copies only its effect arguments before dismissal captures the confirmation.
The worker still claims the exact saved target and session before removing the exact saved profile.
The worker starts before dismissal, as before.
The detached removal result still forgets the host mute and reports failures through the existing notice path.
Accepted rename work and accepted tag preference writes retain their original owners.
The forward manager retains exact target/port ownership and its existing process cleanup.
No runtime, tunnel, confirmation, queue, or generation policy changed.

## Regression evidence

The five actual departure tests failed before implementation.
Reset dropped every departing body.
Log: `/tmp/spaceport-qnz9/ydr4-host-red.log`.

The long-tag regression then found a genuine 1.5-pixel row jump.
Actual rows measured 29 pixels each.
Spacers used the unrounded 29.142857-pixel font metric instead.
The frozen list grew from 2320 to 2329.5 pixels.
The fix derives spacer extents from the measured uniform list.
The regression keeps exact row, body, and panel geometry assertions.
Log: `/tmp/spaceport-qnz9/ydr4-host-tag-geometry.log`.

The final `host_menu` scope passed all 46 tests, with no failure or ignored case.
It includes actual departure, confirmed removal, long-list scroll, passive controls, and native-field release.
Log: `/tmp/spaceport-qnz9/ydr4-host-green.log`.
The complete `device_theme` consumer scope also passed all 21 tests.
Log: `/tmp/spaceport-qnz9/ydr4-host-device-theme-consumers.log`.
Both use the immutable `/tmp/spaceport-qnz9/ydr4-workspace-scroll-tests` executable.
That executable includes the final measured-list fix and confirmed-removal regression.
Fixtures use synthetic hosts, in-memory tags, and handleless forward records.
Confirmed removal uses Saved::Local and cannot invoke a saved-profile CLI command.
Stale rename and port submissions contain invalid input and cannot start an external operation.

## Checks and limits

Owned-file Rust 1.99.0 formatting and the complete diff check passed.
The source-size and sorted-region checks passed.
Localization passed with 2174 messages in 114 files.
Logs: `/tmp/spaceport-qnz9/ydr4-host-format.log`, `ydr4-host-diff.log`, and `ydr4-host-structure.log`.
Root independently reviewed target preservation, accepted-worker ownership, passive controls, and final measured tag capture.
That review approved the focused boundary without a material finding.
The final owner fresh-eyes read also found no material issue.
The shared aggregate host and Windows GNU all-feature lint gates remain pending.
Cargo and artifact runs serialize through `/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock`.
Builds use locked dependencies, Rust 1.99.0, and `CARGO_INCREMENTAL=0`.
The shared target is `.worktrees/upstream-fixes/target`.

Doctor Biz requested keeping native checks pending.
Native keyboard/IME delivery, animation, clipping, accessibility, and performance remain unqualified.
Headless geometry tests do not qualify native drawing or OS input.
This family does not establish completion of the whole `ydr4` issue.
