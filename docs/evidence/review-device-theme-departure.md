<!-- ABOUTME: Records captured device-theme content and prompt owner cleanup. -->
<!-- ABOUTME: Separates actual headless behavior from pending native qualification. -->

# Device theme departure

Date: 2026-11-17. Scope: Device Theme under `ydr4` and `9s44`.
The global Themes popup has its separate [evidence](review-theme-popup-departure.md).
Doctor Biz authorized this fix and its validation.
This owner made no commit, push, release, or Kata change.

## Behavior and ownership

The popup shell captures visible device-theme rows before MenuState drops HostMenu.
The capture stores the device label, field paint, row labels and flags, status, and plain list offset.
It contains no host model, Entity, focus handle, subscription, task, or action closure.
The outgoing list paints the captured visible range with its original row height and offset.

The active and outgoing header, rows, and status share one template.
Outgoing rows have no choice action, hover handler, or close-button handler.
An old copy cannot choose a theme or dismiss a replacement host's popup.
The live renderer retains its existing input and choice paths.

The existing reset already drops the complete host and device-theme picker.
It still releases search, subscription, discovery, preview, and read-before-save ownership immediately.
The fix does not retain those owners for animation.
No additional lifecycle state or cancellation path was added.
No target, profile, connection generation, preview, or persistence handling changed.
Accepted configuration writes keep their existing window owner outside HostMenu.

## Regression evidence

The actual scrolled departure regression failed before the fix.
HostMenu reset removed the departing choice.
Log: `/tmp/spaceport-qnz9/ydr4-device-theme-red.log`.

The complete `device_theme` filter then passed all 21 tests.
Log: `/tmp/spaceport-qnz9/ydr4-device-theme-green.log`.
No case failed or was ignored.
The scope covers these actual contracts:

- A scrolled choice keeps its position after HostMenu drops.
- Dismissal restores window focus and releases the native field after the redraw.
- Frozen rows and close controls cannot change a replacement host without a parent input shield.
- Three pending GPUI tasks and the subscription release after dismissal.
- Existing live search, preview, cancellation, and cross-device preview isolation still work.
- Existing configuration save/unset and accepted-write pin/reload cases still pass.

The new fixtures use built-in themes and private synthetic hosts.
They perform no catalog discovery or saved configuration writes.
The stale-row fixture selects Same as app, which already matches its saved value.
Even a broken active callback therefore cannot write personal configuration.
The task cleanup case uses actual pending GPUI futures and subscription disposal counters.
It does not substitute a fake application cleanup path.

## Checks and limits

Owned-file Rust 1.99.0 formatting and the complete diff check passed.
The source-size and sorted-region checks passed.
Localization passed with 2174 messages in 114 files.
Logs: `/tmp/spaceport-qnz9/ydr4-device-theme-format.log`, `ydr4-device-theme-diff.log`, and `ydr4-device-theme-structure.log`.
Root independently reviewed the pure capture, shared renderers, unchanged target/save paths, and actual cleanup tests.
That review found no material defects.
The source-stable aggregate host and Windows GNU Clippy gates passed with `-D warnings`.
Command: `cargo clippy -p sheprd-gpui --all-targets --all-features --locked -- -D warnings`.
The Windows run adds `--target x86_64-pc-windows-gnu`.
The host uses the pinned Sparkle SDK at `/private/tmp/spaceport-cli-sdk.7S7nty/sdk`.
Logs: `/tmp/spaceport-qnz9/2njz-host-clippy-source-stable.log` and `2njz-windows-clippy-source-stable.log`.
Windows cross-compilation does not qualify native Windows behavior.

Cargo and artifact runs serialize through `/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock`.
Cargo uses locked dependencies, Rust 1.99.0, and `CARGO_INCREMENTAL=0`.
The shared target is `.worktrees/upstream-fixes/target`.
The green build reports only the existing debug-linker unwind and `block 0.1.6` notices.

Native animation, keyboard/IME delivery, screen-reader behavior, and performance checks remain pending.
This family does not establish completion of the whole `ydr4` issue.
