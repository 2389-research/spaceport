<!-- ABOUTME: Records device setup departure ownership and confirmation safety. -->
<!-- ABOUTME: Separates current headless evidence from pending native qualification. -->

# Device setup departure

Date: 2026-11-18. Scope: Add Device, Add WSL, and Remove WSL under `ydr4`.
Doctor Biz authorized this fix and its validation.
No commit, push, release, or Kata change was made by this owner.

## Ownership

The opaque `DeviceSetupPaint` contains prepared Core, WSL, or removal display data.
Active and outgoing copies share form, footer, header, rail, and confirmation templates.
Outgoing controls install no close, submit, distribution-choice, or source-switch callback.
Fields paint text without retaining their Entity or focus handle.

Each live setup form owns its body scroll handle.
Outgoing bodies retain only numeric offsets.
The WSL capture retains only visible distribution rows and numeric prefix/suffix spacers.
One measured list extent supplies its snapped uniform row metric.
The capture contains no task, subscription, receiver, backend, request, or geometry-cell owner.

The Core request, app-wide Gate, worker thread, and reply handling retain their existing ownership.
Closing the form still leaves accepted Core work and uncertain outcomes under the existing Gate.
The WSL discovery/check observer still drops with WslSetup during MenuState reset.
No WSL task or accepted-operation controller was detached or transferred for animation.

Remove WSL now clones its distro string before dismissal captures the confirmation.
The existing worker still starts before dismissal.
It still removes only that saved distribution from the client catalog.
Its detached result still forgets the host mute and reports storage failures.
The fresh-Enter confirmation guard remains intact.
No distribution or daemon is installed, stopped, or modified.

## Verification

All three actual body regressions failed before implementation.
Log: `/tmp/spaceport-qnz9/ydr4-device-setup-red.log`.
The complete `menu::devices::` scope passed 88 tests with no failure.
One existing isolated CLI producer fixture was ignored by that direct filter.
Log: `/tmp/spaceport-qnz9/ydr4-device-setup-consumers.log`.

Five additional control and cleanup tests passed, including in the final full consumer scope.
They cover passive Core/WSL submit and close buttons, passive distribution rows, native-field release, and pending observer cancellation.
Log: `/tmp/spaceport-qnz9/ydr4-device-setup-controls-green.log`.

Both narrow scroll cases passed from the immutable `5kxORR` artifact.
They preserve exact panel, body, and field/row bounds after reset.
The WSL case also proves offscreen rows are omitted from the capture.

The private removal subprocess fixture first stopped at distro-name validation, before any catalog seed write.
Its names were corrected to valid hyphenated names after inspecting `valid_distro`.
Its parent test then passed and ran its ignored GPUI child through an exact filter.
Log: `/tmp/spaceport-qnz9/ydr4-device-setup-private-green.log`.
The parent uses the existing `test_sandbox` and an exact child-test filter.
The child verifies private HOME and XDG state paths before using the real WSL catalog APIs.
It tests stale submit/cancel against a replacement saved distribution and real accepted removal of the original.
No production mock mode was added.

The complete setup appearance scope passed six tests with no failure.
The private child is ignored only by the direct filter; its parent executes it explicitly.
Log: `/tmp/spaceport-qnz9/ydr4-device-setup-green.log`.
These final scopes used immutable `/tmp/spaceport-9s44-gpui.5kxORR/spaceport-tests`.
The additional exact Remove WSL shell-scroll case passed from immutable `gVX0UV`.
It uses a long valid distro name and a 150-pixel window.
It preserves the exact panel and button bounds after scrolling and resetting the confirmation.
Log: `/tmp/spaceport-qnz9/ydr4-device-setup-remove-scroll-green.log`.

## Remaining checks

Root and the popup owner independently reviewed the concrete forms, passive controls, exact removal order, and successful private fixture.
Neither review found a material issue.
Root's final approval also covers the central capture/scroll hooks and the exact shell-scroll case.
Owned source formatting and source-size/sorted/localization checks passed.
The shared aggregate host/Windows GNU lint gates remain pending.
Native keyboard/IME delivery, the Windows source rail, drawing, accessibility, and performance remain pending.
Doctor Biz requested keeping native qualification pending.
Headless tests do not prove native behavior.
This family does not establish completion of the whole `ydr4` issue.
