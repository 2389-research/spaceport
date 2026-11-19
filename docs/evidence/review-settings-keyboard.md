# Settings keyboard navigation

Date: 2026-11-17. Scope: `d0n2` under `9s44`.
The original implementation authorization excluded commits and external writes.
Doctor Biz later authorized the aggregate draft checkpoint and its qualification updates.

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

## Windows qualifier implementation

The private fixture now opens Settings through the normal main-window button.
It retains the unique PID-owned Settings HWND, rather than `Process.MainWindowHandle`.
Every key requires that exact foreground HWND and focused-element ancestry within Settings.
It refuses already-held modifiers before sending a key.

The first stage anchors the actual search field, then uses Tab and Shift+Tab.
Space opens General from its navigation tab.
Native Tab reaches Smooth scrolling, leaves it, and returns to the same control.
Switches expose UIA Button and Toggle state in the pinned AccessKit version.
Layout segments expose RadioButton and SelectionItem state.

The capture observer starts only for the Windows integration fixture.
It observes the actual singleton handle without changing focus or settings.
It captures the switch row with keyboard focus, then captures it after focus leaves.
The comparison requires identical geometry, viewport, ring color, and switch state.
It counts actual ring-colored pixels near the switch and requires more when focused.
It retains both row PNGs for inspection.
Image cropping, counting, and encoding run on the background executor.
Full readbacks are bounded to four million pixels; encoded crops are bounded to 128 KiB each.

Later native stages toggle the switch with Space and restore it with Enter.
They select Bubbles and restore Timeline with the actual segment controls.
The parent parses the real private TOML after each action and checks persisted values.
The language dropdown opens, receives Down/Up, dismisses, reopens, and selects its current English choice.
Dismissal and selection must restore the exact trigger and collapse its expanded state.
The existing headless tests separately check the intermediate arrow-selection indices.

Nine focused Settings keyboard tests pass, with none ignored.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- settings_window::tests::keyboard settings_window::ui::tests::keyboard --nocapture`.
Log: `/tmp/spaceport-9s44-settings-native-consumers.log`.
Both host and Windows GNU lint configurations and the static checks pass.
Independent review finds no material ownership, keyboard, measurement, or original-scope gap.
Actual Windows keyboard execution, persisted-value receipts, and both native PNG inspections remain pending.
Kata `d0n2` remains open.

## First native result on `7607bde87`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44537828031/job/135608225364) passes all eight GUI cases.
Its UIA scope passes twelve cases and fails the Settings observer's log read.
The failure occurs after `SETTINGS_RING_READY focused` at `2026-11-19T14:05:02.4599080Z`.
Windows rejects `File.ReadAllText` because its default sharing mode excludes the GUI's open writer.
This is a harness sharing failure, before the unfocused comparison and later control checks.

The observer now uses `FileStream` with `FileShare.ReadWrite`.
It handles short reads and reads at most one MiB plus one sentinel byte.
It rejects overflow and disposes the stream in `finally`.
It preserves keyboard guards, measurement rules, and deadlines.
Independent review finds no material issue in that correction.
Focused Windows harness lint and the diff check pass; corrected native execution remains pending.

The focused PNG was decoded, framing/hash-checked, and inspected.
It shows the authored Smooth scrolling row and its native switch.
It has 7748 bytes and SHA-256 `bd0ecd512c828f5abca3c1c1a6868cade4b5ea219bf54275a820621a16877e5d`.
The partial image and manifest remain in `/tmp/spaceport-9s44-settings-native-760/`.
One focused image does not prove the comparison; `d0n2` stays open.

## Native focus acknowledgment on `7711184cc`

The next [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44542123431/job/135623241590) again passes all eight GUI cases and native chat.
Its Settings case fails earlier, immediately after requesting search focus.
It does not reach the corrected log read or emit another ring image.
The UIA scope has twelve passing cases and one Settings failure, with none ignored.

Pinned `accesskit_windows 0.34.0` implements `SetFocus` by dispatching `Action::Focus`.
Pinned `gpui-pre 0.3.10` sends that action through a channel to its foreground executor.
The provider returns before that executor applies `Window::focus`.
An immediate UIA focus assertion therefore races the queued action.

The fixture still requests focus once.
It now waits up to three seconds for exact search focus within its existing 35-second parent budget.
Each poll retains the exact Settings foreground HWND and ancestry guard.
The timeout logs a fixed diagnostic, without reading another window's control name.
Independent review confirms the async contract and retained guards.
Actual native execution of the acknowledgment correction remains pending.

## Completed native qualification on `940749d29`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44545681976/job/135635803161) passes all eight GUI and fifteen UIA cases, with none ignored.
The complete CI run has eighteen successful jobs and one unconfigured target skipped.
Supply-chain checks and the ARM native job also pass.
ARM does not run UIA under the existing runner policy.

Actual Settings keys pass at `2026-11-19T14:49:48.5050717Z`.
Tab and Shift+Tab traverse the owned Settings window.
Space changes Smooth scrolling to false; Enter restores true.
Native segment activation saves Bubbles, then restores Timeline.
The real private TOML confirms each value and the unchanged English language.
Dropdown dismissal and selection return focus to the exact trigger.

The native focused and unfocused captures retain identical row geometry and switch state.
The focused capture contains 120 ring-colored pixels; the unfocused capture contains zero.
Both PNGs were decoded, framing/hash-checked, and visually inspected.
Both show the authored Smooth scrolling row and its switch, with the changed focus border.
The focused PNG has 7748 bytes and SHA-256 `bd0ecd512c828f5abca3c1c1a6868cade4b5ea219bf54275a820621a16877e5d`.
The unfocused PNG has 7561 bytes and SHA-256 `756d1578ac9d9386b2c168ef06567d09d6a5a8ec85ae9d26af96b63dcf7d50e0`.
The pair and manifest remain in `/tmp/spaceport-9s44-settings-native-940/`.

The fresh original-scope audit confirms switches, segments, buttons, keyboard reachability, activation, and visible focus.
The 267 headless Settings cases cover the remaining control callbacks, traversal, IME, and boundaries.
The native case qualifies OS keyboard delivery, persisted values, dropdown return focus, and the painted ring.
The original issue does not require release profiling or a separate Mac-only matrix.
Those broader checks remain tracked by their own open findings.
This evidence satisfies the original scope of Kata `d0n2`.

## Native toggle observation on `fe4b3d7e4`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44565690663/job/135707084161) passes eight GUI cases and eighteen UIA cases.
Settings passes its ring comparison and saves Smooth scrolling as false after Space.
Its next Enter check observes the unchanged toggle after the fixed 100 ms wait.
That fails at `2026-11-19T16:35:40.3540622Z`.

[Microsoft's SendWait contract](https://learn.microsoft.com/en-us/dotnet/api/system.windows.forms.sendkeys.sendwait) does not await another process on its SendInput path.
The fixture now polls the actual toggle state for at most three seconds after that single keypress.
It retains the exact Settings HWND, focus ancestry, and switch focus guards while waiting.
It sends no additional key and still verifies the actual private saved preference afterward.
A missing activation still fails at the deadline.
Native execution must confirm whether asynchronous observation explains this failure.
The previous completed qualification remains recorded above; this regression run remains failed.

Root reviewed the four-file diff, actual failing receipt, parent deadline, and owned-focus guards.
The review found no further issue in this bounded observation change.
`just format-check check-file-size check-sorted check-just-operations check-i18n check-commits` passes.
`git diff --check` passes.
No PowerShell runtime is installed on this Mac; actual script execution remains assigned to Windows CI.
