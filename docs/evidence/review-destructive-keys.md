# Destructive confirmation keys

Verified on macOS with Rust 1.99.0 on 2026-11-17.
This covers review findings `fe28` and `1qx4`.

## Cause and change

The process, SSH device, and WSL confirmation handlers accepted repeated Enter events.
They now consume held Enter without submitting.
Process-list arrow repeats still move the cursor.

Core split panes previously removed their layout and ended their terminal before asking.
They now use the shared close dialog and `confirm_close_pane` preference.
The captured target retains its workspace, tab, terminal, and Core identity.
Confirmation checks those values before removing the pane or queuing its kill.
Observer panes still detach without ending their terminal.

GPUI dispatches shortcut actions before raw key handlers.
Its action and keystroke interceptor APIs omit the repeat flag.
ClosePane therefore keeps an unreachable menu-label binding, like existing Edit actions.
The window capture handler matches the resolved keymap and rejects held events.
Menu actions and prefix chords retain their existing command route.

Canceling a Core close restores the captured pane's focus.
If that pane disappeared, cancellation returns to the window's root focus.
It never selects a replacement pane through that restoration path.

## Regression evidence

The three `fresh_confirmation` tests failed before the guards.
Held Enter submitted each destructive confirmation.
All three tests pass with the guards.

Core tests failed before routing changes because no confirmation opened.
Held shortcuts also removed the whole tab through neighboring panes.
A separate failing assertion proved that cancellation lost Core pane focus.
Six Core close tests now pass.
They cover consent, cancellation, missing panes, stale selection, repeats, and rebinding.
The positive cases observe a framed `terminal.kill` request for exactly `t2`.
The scripted peer returns a valid finished-terminal response.

The following default-feature checks passed:

- `cargo test -p sheprd-gpui --bin spaceport fresh_confirmation --locked`: 3 tests.
- `cargo test -p sheprd-gpui --bin spaceport close_confirmation --locked`: 6 tests.
- `cargo test -p sheprd-gpui --bin spaceport close_modal:: --locked`: 12 tests.
- Compiled test-binary filters for `pane_menu::`, host menus, WSL menus, Core pane ownership, actions, and window key actions: 73 tests.
- `git diff --check`, `just check-file-size`, and `just check-sorted`.

Cargo calls used the shared task lock and target directory.
They set `CARGO_INCREMENTAL=0`.
Logs remain under `/tmp/spaceport-qnz9/destructive-*.log`.
The linker reports its existing compact-unwind size warning.
Cargo also reports the existing future-incompatibility notice for `block`.
A concurrent connection-reader change briefly reported an unused handle.

## Native and runtime boundaries

Integration-only helpers prepare native confirmation surfaces without real external targets.
The process helper has no worker and cannot signal a process.
The host helper uses a local sentinel and cannot invoke the device CLI.
The Core helper refuses enabled or connected Core state.
It owns one synthetic Store tab and supplies exact cleanup.
The root smoke harness owns native execution and its separate results.

The native harness passed on macOS in dark and light themes at 800 and 360 pixels.
It checked held Enter and Escape cancellation on process, device, and Core confirmations.
Modal capture waits for the first rendered animation to settle.
Root inspected the narrow Core, wide Core, process, and device captures for clipping and visible controls.
Captures are under [confirmations](review-squad/confirmations/).
The device fixture uses an inert local sentinel, so its body does not represent a real SSH device's description.
The input goes through GPUI event dispatch, not an OS keyboard-equivalent event.
This proves native rendering and handlers, not AppKit Cmd-W routing or real runtime termination.
The final run is `/tmp/spaceport-9s44-native-confirmations-settled.log`.
Root independently reviewed the production diff, exact target checks, and regression evidence without material findings.

The process regression uses the existing worker channel fixture.
The host regression checks UI submission with a local sentinel.
The WSL regression uses a unique nonexistent distribution name.
The Core integration tests use a scripted peer, not a running daemon.
These tests do not establish real runtime termination or WSL behavior on Windows.
Native Windows verification and full platform gates remain CI work.

## Fresh-press finding completion on November 18

Independent review confirmed `fe28` covers repeat suppression on the existing three confirmations.
Actual process, device, and WSL destruction are not needed to verify that guard.
The isolated native confirmations exercised held Enter, Escape, layout, and visible controls.
All three `fresh_confirmation` regressions pass against the task's integration-feature unit executable.
The latest log is `/tmp/spaceport-9s44-fresh-confirmation-final.log`.
Canonical `just lint` passes workspace all-target checks with all features and without default features.
Its log is `/tmp/spaceport-9s44-latest-host-lint-green.log`.
Formatting, size, sorting, i18n, and diff checks pass.

The separate `1qx4` Core Cmd-W finding remains open.
Its remaining check must enter consent through a focused Core split's native Cmd-W path.
The existing native helper opens that dialog directly and does not prove this route.
