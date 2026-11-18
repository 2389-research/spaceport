# Input ownership and connection recovery

Date: 2026-11-17. Scope: `jwa1`, `2qr3`, and `yftv` under `9s44`.

## Acceptance and changes

Recovery card actions must be reachable through the existing chrome keyboard region.
They use its tab order, focus ring, and standard Enter/Space click behavior.
The notice region remains available while a recovery card is visible.

A missing local Herdr installation must require an explicit retry.
The install card now says to install Herdr, then choose Try again.
Ordinary connection failures still retain their existing automatic retry behavior.

Passive pointer motion must not claim Herdr's shared focus or window size.
Focus follows mouse now defaults off in both parsing and generated defaults.
Users can explicitly enable the existing behavior.
Documentation explains that enabling it changes other clients' focused pane.
Dragging, clicking, scrolling, pasting, and dropping still count as input.

## Regression evidence

The corrected keyboard baseline failed both activation tests without the focus-region changes.
These tests dispatch both key down and key up, matching GPUI button activation.
Both tests pass with the changes.
The initial key-down-only attempt was insufficient and is not baseline evidence.

The missing-installation regression waits until the reader has drained before checking retry eligibility.
It failed before the missing-installation guard, then passed with it.
The earlier handle-only wait raced the reader and is not baseline evidence.
Nine missing-installation and card tests passed.

The default-hover and passive-pointer regressions failed before their changes.
Five focus-following tests and eleven input-ownership tests passed afterward.
Coverage retains explicit opt-in hover focus through an isolated framed transport.
It also checks dragging still counts as input.

Local logs use `/tmp/spaceport-9s44-` with keyboard, missing-herdr, hover, and pointer names.
Cargo used Rust 1.99.0, `--locked`, and the shared Cargo lock and target.
The linker compact-unwind warning and `block` future-incompatibility notice predate these changes.

Native focus appearance and final platform CI remain pending.
Headless key events do not establish OS keyboard routing or screen-reader behavior.

## Missing-Herdr closure audit, November 18

This audit separates `2qr3` onboarding from `jwa1` keyboard qualification.
Kata comment 2900 records an actual native confirmation of the missing-installation card.
Its title and Install Herdr/Try again buttons remain unchanged.
The current shorter note explicitly asks the person to install Herdr, then choose Try again.

The original issue asks to stop automatic retries.
An older comment's thirty-second automatic recovery expectation conflicts with that requested behavior.
The current contract therefore requires an explicit retry after installation.
The `may_dial` guard prevents the automatic poll path from retrying even after backoff expires.
It preserves the ordinary connection-failure backoff path.

The actual card click test dispatches a click through the production element.
The platform test harness records exactly `https://herdr.dev/tui/` as the opened URL.
Review confirms that callback only invokes `open_url`; it downloads or executes nothing.
This is production-callback verification, not a fresh native browser launch.

The previous recovery test injected its successful connection state.
A new test now binds a private socket and accepts the real connection worker.
It withholds the welcome and verifies that socket acceptance alone keeps the missing-installation advice.
It sends real framed welcome, snapshot, and complete surface messages.
The old picture stays stale until the coherent replacement frame arrives.
The fresh picture then clears the missing state and outage.
The starting missing-installation state remains fixture-injected.
The existing native confirmation covers that producer scenario.

The new recovery test passes in `/tmp/spaceport-9s44-missing-real-recovery.log`.
All ten missing/card/retry consumers pass in `/tmp/spaceport-9s44-missing-card-and-retry.log`.
Both keyboard consumers pass in `/tmp/spaceport-9s44-missing-keyboard-consumers.log`.
Those headless events do not close `jwa1`.
The typed missing-executable and private-path lookup checks each pass.
Their logs are `/tmp/spaceport-9s44-missing-{classification,path-lookup}.log`.

Independent lifecycle review found no material defect in the guard, explicit retry, or picture transition.
It confirmed that the original onboarding scope can close after the current checks pass.
Canonical host lint passes both all-feature and no-default-feature workspace/all-target legs.
Its log is `/tmp/spaceport-9s44-missing-recovery-host-lint.log`.
Formatting, file size, sorting, and diff checks pass.
Canonical Windows GNU lint passes both feature configurations too.
The log is `/tmp/spaceport-9s44-missing-recovery-windows-lint.log`.
This is cross-compilation, not native Windows execution.

Kata `2qr3` closed after these checks and the scoped review.
Close event `9647` has UID `01M7R9E97RXRY1W47V9HZSR0ET`.
Fresh issue and parent reads confirm the closure, thirty-one closed children, and twenty-two open children.
The umbrella remains open.
The closure ledger retains the issue, event, and evidence identities.
