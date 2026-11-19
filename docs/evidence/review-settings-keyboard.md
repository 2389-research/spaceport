# Settings keyboard navigation

Date: 2026-11-17. Scope: `d0n2` under `9s44`.
Doctor Biz authorized fixes and validation without commits or external writes.

## Current implementation

Enabled shared Settings switches, segments, choices, and buttons join the window's tab order.
They use the existing GPUI keyboard click path and their original callbacks.
Disabled controls offer no focus action and remain outside the tab order.
The shared focus style uses the existing theme contrast policy.
Coloured swatches use their own background when choosing the focus-ring colour.

Settings owns a local Tab and Shift+Tab handler.
Native Settings fields join that tab order through one constructor.
Composing fields retain focus and keep their marked text.
An invalid numeric or hex edit refuses navigation until it can commit.
The existing row bounds and GPUI context identify focused rows for scroll reveal.
Dropdowns expose named button roles and expanded state.
Their captured previous focus restores keyboard focus on dismissal or selection.
No global shortcut or command catalog change is needed.

## Regression evidence

The shared controls originally offered no Focus action.
Real focus traversal and keyboard events invoked none of their callbacks.
See `/tmp/spaceport-qnz9/d0n2-controls-red.log`.
The original Settings window could not Tab into its search field.
See `/tmp/spaceport-qnz9/d0n2-window-red.log`.
The dropdown builder exposed no button node.
See `/tmp/spaceport-qnz9/d0n2-dropdown-red.log`.
Host colour keys invoked no colour callbacks before focus support.
See `/tmp/spaceport-qnz9/d0n2-swatches-red.log`.
An enabled lower row received focus at y1426 while the viewport ended at y564.
Scroll bounds now include the painted bottom border.
See `/tmp/spaceport-qnz9/d0n2-scroll-red.log`.

GPUI's keystroke helper sends KeyDown without KeyUp and may simulate committed text.
Keyboard-click tests deliver matching native key events after drawing the focused frame.
Composition tests use raw key events without manufacturing an IME commit.
The checkpoint compiles and passes six actual Settings keyboard tests.
They cover search, section activation, IME ownership, invalid numeric text, scroll reveal, dropdown return focus, and picker boundaries.
The shared Settings UI scope passes 24 tests, including numeric and colour keyboard callbacks.
Logs: `/tmp/spaceport-qnz9/d0n2-checkpoint-window.log` and `d0n2-checkpoint-controls.log`.
A valid Fonts-page fixture fails when picker confinement is disabled.
See `/tmp/spaceport-qnz9/d0n2-picker-real-red.log`.
The exact baseline hunk is restored; production source contains no disabled confinement.

The remaining custom-control baseline ran the full Settings scope.
Five real keyboard tests failed while 261 other tests passed.
Search results, indicator cards, backdrop tiles, font buttons, and spacing resets lacked tab stops.
See `/tmp/spaceport-qnz9/d0n2-custom-complete-red3.log`.
That run also caught navigation probes entering the catalog-row map.
Navigation and custom choices now use a separate focus-bounds map through the same geometry probe.
The existing catalog completeness test remains unchanged.

The opacity ruler exposes its numeric range and handles unmodified arrows, Home, and End.
Its original save callback remains authoritative for pointer and keyboard input.
Disabling only its keyboard handler fails with no write instead of `Opacity(0.84)`.
See `/tmp/spaceport-qnz9/d0n2-ruler-keys-red.log`.
The throwaway baseline ran through `uv run --no-project` outside the repository.
It restored the exact owned source before releasing the shared Cargo lock.
No project or dependency files were added for that one-off check.

Font buttons now join the existing modal tab order and expose their names.
The picker handles family-navigation keys only while its search field owns focus.
Focused close, default, and family buttons keep their own activation callbacks.
Custom controls reuse the shared contrast-aware focus style.
The shared persistence fixtures capture typed writes without personal configuration changes.
These callback tests do not replace native keyboard or file-save qualification.

Checkpoint format, file-size, sorted-region, and diff checks pass.
The localization check passes after annotating the unchanged SettingsFontPicker context as non-message data.
The annotation stays inside the closure so rustfmt preserves its association with the literal.
The final localization result is 2157 valid, used messages in 114 files.
Root's earlier localization failure preceded this annotation.
See `/tmp/spaceport-qnz9/d0n2-checkpoint-i18n.log`.
The focused tests report only the existing debug-link unwind and block 0.1.6 future-compatibility notices.

## Qualification and review

The full Settings scope passed 267 tests with no failures or ignored cases.
Log: `/tmp/spaceport-qnz9/d0n2-full-green.log`.
Root independently reviewed the shared keyboard, IME, dropdown, and scroll behavior.
Root also reviewed the custom cards, opacity ruler, font callbacks, search results, and spacing reset tests.
That review found no material defects.

Settings remains a singleton through `settings_window.rs::open_with`.
An existing window is retargeted and activated before a new window can be created.
The focus-bounds map follows the existing catalog-bounds ownership model.

The README now describes Tab, Shift+Tab, button activation, dropdown dismissal, and opacity slider keys.
Native qualification remains pending.
No commit, external write, or Kata edit was made by the Settings owner.

The first final lint run stopped on sibling Core test visibility and unfinished popup paint wiring.
Log: `/tmp/spaceport-qnz9/d0n2-host-clippy.log`.
A resumed Settings run stopped on a sibling Devices regression's private Flash import.
Log: `/tmp/spaceport-qnz9/d0n2-resume-settings.log`, exit 101.
After the sibling import fix, the fresh Settings scope passed all 267 tests again.
It reported no failures or ignored cases.
Log: `/tmp/spaceport-qnz9/d0n2-resume-settings-final.log`.
The first resumed lint exposed six test-only unwrap diagnostics.
The Settings and clipboard owners added narrowly scoped test-module allowances.
Both fresh platform lints then passed with `-D warnings`:

- Host: `cargo clippy -p sheprd-gpui --locked --all-targets --all-features -- -D warnings`.
- Windows cross: the same command with `--target x86_64-pc-windows-gnu`.

Logs: `/tmp/spaceport-qnz9/d0n2-host-clippy-final.log` and `d0n2-windows-clippy-final.log`.
The host used the pinned Sparkle SDK below.
Windows cross-compilation does not qualify native Windows input.
The host reports the existing `block 0.1.6` future-compatibility notice.

The fresh file-size, sorted-region, and localization checks passed.
Localization reports 2162 messages in 114 files.
The full `git diff --check` passed.
Logs: `/tmp/spaceport-qnz9/d0n2-resume-structure.log` and `d0n2-resume-diff.log`.
The aggregate format check found only concurrent sibling edits outside Settings.
Log: `/tmp/spaceport-qnz9/d0n2-resume-format.log`.
Those owners received the exact paths; aggregate formatting still needs a passing retry.
Later static-dialog integration checks also passed size, sorting, localization, and the full diff check.
Settings ownership transferred after the verified keyboard implementation.
The chat accessibility owner may extract the shared focus-ring builder without changing Settings behavior.

All Cargo commands must use `/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock`.
Preserve `CARGO_INCREMENTAL=0` and the shared upstream-fixes target directory.
The SDK path is `/private/tmp/spaceport-cli-sdk.7S7nty/sdk`.
Do not reuse q72g's earlier Clippy result as validation for these later changes.

## Limits

Headless tests exercise actual GPUI controls and their real callbacks.
They do not qualify physical keyboard delivery, native glyphs, or screen-reader behavior.
The dedicated Mac keyboard fixture has not reached its matrix assertions.
Native Settings keyboard checks remain pending.
The full workspace CI gate passes on source `88566686c`.

## Private Windows qualification route, November 19

The original-scope audit finds no remaining Settings implementation defect.
Open Settings through the owned app's normal Settings button.
Find exactly one PID-owned top-level window titled `Settings` and retain its HWND.
Scope control lookup to that root.
Before each key, require that exact foreground HWND and focused-element ancestry within Settings.
The existing `Process.MainWindowHandle` target does not distinguish Settings from the terminal window.

Traverse search, navigation, and controls with Tab and Shift+Tab.
Toggle Smooth scrolling with Space and Enter, then verify the private saved preference.
Choose and restore a chat-layout segment.
Open and dismiss a dropdown, then verify focus returns to its trigger.
Compare native focused and unfocused switch images to qualify the painted focus ring.
UIA focus alone cannot prove that ring.

This route can run in the private Windows CI desktop without a Mac foreground fixture.
Speech, contrast auditing, release profiling, and exhaustive glyph checks belong to separate findings.
Keep `d0n2` open until its actual keyboard and image receipts pass.
