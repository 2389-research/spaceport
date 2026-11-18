<!-- ABOUTME: Records resumed native qualification after Doctor Biz unlocked the desktop. -->
<!-- ABOUTME: Separates completed native fixtures from manual input, platform, and profile gaps. -->

# Resumed native qualification

Doctor Biz unlocked the desktop on November 18 and authorized resuming the pending native checks.
This does not authorize commits, publication, OS preference changes, or personal-session test fixtures.

## Completed native fixtures

The integration-test build completed from the review task worktree.
Build logs: `/tmp/spaceport-9s44-unlocked-native-build.{json,log}`.
The live test executable is `target/debug/deps/live_gui-2759daaef30b3fc6` in the shared target.
Its compiled app uses the same task source and integration-test feature.

- `native_local_controls --ignored --exact --nocapture --test-threads=1`: passed.
  It runs both themes, wide and narrow layouts, held Enter, cancellation, and focus isolation.
  Log: `/tmp/spaceport-9s44-unlocked-local-controls.log`.
  Captures: `/tmp/spaceport-9s44-unlocked-controls.fnKDHa/`.
- `native_sidebar --ignored --exact --nocapture --test-threads=1`: passed after the opener fix below.
  It checks actual AppKit clicks, native glyphs, Settings layouts, Unicode fields, and focus isolation.
  Log: `/tmp/spaceport-9s44-unlocked-sidebar-qualified.log`.
- `native_gui_live --ignored --exact --nocapture --test-threads=1`: passed.
  Its explicitly selected runtime is `/Users/harper/.local/bin/herdr`.
  It owns an isolated daemon, shells, sockets, clipboard fixtures, and native windows.
  It checks tabs, splits, workspace updates, reconnect input, and native clipboard/selection routes.
  Log: `/tmp/spaceport-9s44-unlocked-tui-live.log`.

These fixtures deliver events to exact native windows.
They do not establish every global keyboard shortcut, assistive announcement, or Windows behavior.

## Popup opener regression

The first native sidebar run failed when reopening the footer menu after closing Shortcuts.
The outgoing Shortcuts shield covered the actual live opener.
Its bounds were `(161,151,478,558)`; the opener bounds were `(234,405,29,16)`.
Logs: `/tmp/spaceport-9s44-unlocked-sidebar-{diagnostic,bounds}.log`.

The deterministic headless regression proves the same overlapping-bounds precondition.
It fails because the click leaves the menu closed.
Red executable: `/tmp/spaceport-9s44-gpui.K6ZadL/spaceport-tests`.
Red log: `/tmp/spaceport-9s44-native-opener-red.log`.

The fix removes the outgoing parent input shield.
Captured templates remain inert; the active overlay retains its input guards.
Independent review found no stale action callback in the passive templates.
The regression and 66 appearance checks pass; 47 departure checks also pass.
The scopes overlap and must not be summed.
Green logs: `/tmp/spaceport-9s44-native-opener-{green,detached-green}.log`.
The native sidebar rerun also passes.

## Generic shell scrolling

Review then found implicit wheel listeners in closing footer, static, and confirmation shells.
Six genuine runtime regressions prove a lost 12-pixel offset and movement after dismissal.
Red log: `/tmp/spaceport-9s44-generic-shell-corrected-red.log`.
The earlier origin assertion failures were invalid baseline evidence.

The fix owns the active generic scroll handle and captures only its numeric offset.
Reset clears the active offset; every outgoing shell disables scrolling.
The original specialized-family scroll policies remain in place.
All six cases pass from `/tmp/spaceport-9s44-gpui.iS6Ll0/spaceport-tests`.
The appearance scope passes 72 cases, with one parent-invoked private subprocess ignored directly.
The departure scope passes 50 cases; these scopes overlap.
Logs: `/tmp/spaceport-9s44-generic-shell-{green,appear-green,departure-green}.log`.
Independent review approved scroll ownership, capture/reset order, clipping, and input isolation.
Fresh native and lint qualification remain pending this latest scroll fix.

## Manual fixture limits

The UI tool observed accessible chat, recovery, and Settings labels in private native windows.
It also observed visible focus in two chrome regions.
Window activation and capture remained unreliable when those windows were not foreground.
No complete manual keyboard or assistive-technology qualification is claimed.

An early temporary preview expired and the UI tool relaunched its unguarded bundle with normal app state.
Root stopped interacting with that session and closed only the temporary app copy.
No personal terminal content is retained in repository evidence or used as a fixture.
That expired bundle now refuses launch with exit 78 and retains its original binary separately.

Later bundles clear their environment and refuse a second launch through a one-shot private claim.
The guard was exercised and returned exit 78.
Use `HERDR_TEST_TMPDIR=/tmp` to keep development control sockets below the macOS path limit.
The default long temporary root made the first manual control socket unreachable.
The short-root session's private CLI read its own PID and quit it cleanly.
All manual app copies were verified closed through fresh app inventory.

## Remaining work

The native Settings keyboard matrix is implemented; foreground qualification remains pending.
The dense-terminal release budget is supplemental evidence, not the requested whole-window profile.
The profile must identify its synthetic status and output workloads explicitly.
Actual provider execution, streaming ACP, Core performance, and Windows execution remain separate gates.
No additional Kata closure, commit, push, PR, release, or OS preference change occurred at this checkpoint.

## Chrome entry and foreground keyboard checks

Native interaction exposed a gap while keyboard focus entered window chrome.
Tab reached the terminal before the deferred control focus, and Escape did not cancel that focus.
Both regression cases failed against the original code.
The log is `/tmp/spaceport-9s44-chrome-entry-red.log`.

Chrome entry now immediately focuses a zero-area sentinel inside the shortcut scope.
Deferred focus requires the same region and sentinel ownership.
Escape and later focus changes cancel the handoff.
Nine chrome focus tests pass, including ordinary mouse focus and later focus changes.
The log is `/tmp/spaceport-9s44-chrome-entry-final.log`.
Independent static review approved the ownership and GPUI event path.

The new `native_settings_keyboard` fixture keeps exact AppKit key-window checks.
It waits for the operator to select **Spaceport keyboard check**.
The November 18 run timed out after 60 seconds with that window visible but inactive.
No keyboard matrix pass is claimed.
The exact child exited and was reaped; the window is closed.
The log is `/tmp/spaceport-9s44-native-settings-foreground.log`.
After Doctor Biz confirmed readiness, a second run also timed out with the exact window inactive and non-key.
That run is `/tmp/spaceport-9s44-native-settings-ready.log`; it reached no keyboard matrix assertions.
Both child processes exited and were reaped.
Ordinary `native_sidebar` retains its existing checks without the additional foreground matrix.
See [the keyboard fixture instructions](../native-settings-keyboard.md).

The latest ordinary sidebar rerun failed at its later decoy-window activation deadline.
The native window resized, but remained non-key while the app stayed inactive.
Before that failure, Settings, fonts, Core preferences, confirmations, shutdown, and local controls passed.
Those partial results do not make the complete driver pass.
The log is `/tmp/spaceport-9s44-latest-sidebar.log`.

Profile readiness tests also found four invalid captured states accepted immediately before sampling.
They cover missing groups, stale frames, lost Working, and unexpected Working during Idle.
The valid case passed while all four invalid cases failed before the correction.
After the correction, all eleven ordinary profile harness tests pass.
The completed release native profile remains pending.
Logs are `/tmp/spaceport-9s44-profile-before-red.log` and `/tmp/spaceport-9s44-profile-harness-green.log`.

## Release measurement attempts

The optimized integration build succeeded on November 18.
The dense native rendering fixture passed its cached/reference glyph and changed-cell assertions.
Warm terminal p95 was 1.465 ms, warm scroll p95 was 3.751 ms, and first content took 19.679 ms.
These machine-specific fixture timings do not replace a working-session CPU profile.
The log is `/tmp/spaceport-9s44-dense-native-performance.log`.

The first working-session run failed while waiting for generated terminal output in its seed peer.
The private command history proves the generated printf command ran.
The run opened no GUI and sampled no process CPU interval.
Its private directory is `/tmp/h46be-d5eda0-0`; the exact daemon child was stopped and reaped.
The log is `/tmp/spaceport-9s44-working-native-profile-first.log`.
This failed seed attempt supplies no performance conclusion.

The diagnostic rerun proved the marker arrived beside Herdr's scrollbar cell.
Whole-row equality rejected that actual output.
One shared row marker helper now serves the seed peer and native observer.
The captured decorated-row regression failed before the fix; three controls already passed.
All fifteen ordinary profile harness tests now pass.
They retain command-echo rejection, row boundaries, CPU semantics, and exact pre-sample readiness checks.
Logs are `/tmp/spaceport-9s44-profile-marker-red.log` and `/tmp/spaceport-9s44-profile-latest-harness-green.log`.

The corrected native rerun passed seeding but timed out before initial frame presentation.
Its exact GUI was connected to workspace w2 but inactive, non-key, and GPUI-occluded.
The observer reported zero group bounds and no presented frame.
No CPU interval was sampled, and no working-session performance result is claimed.
Private artifacts remain at `/tmp/hed91-4415b00-0` after exact child cleanup.
The log is `/tmp/spaceport-9s44-working-native-profile-corrected.log`.

## Kata closure

Core experiment guidance (`jemh`) closed after its copy-only acceptance checks and independent review.
Its closure event is `9502` (`01M7QSRXM67SSS8D7DFGZ0PTXR`).
A fresh parent read confirms 29 closed children and 24 open children.
The umbrella remains open.

Fresh-press confirmation (`fe28`) then closed after its three regressions, native confirmation checks, and independent review.
Canonical host `just lint` passes workspace/all-target checks with all features and without default features.
Its verified close event is `9503` (`01M7QTDYHPCMCJ5NVG5BY0TMY4`).
A fresh parent read now confirms 30 closed children, 23 open children, and the open umbrella.
Core native Cmd-W qualification remains open under `1qx4`.

Final ordinary release profile harness rerun: fifteen passed, none failed, one native opt-in ignored.
The log is `/tmp/spaceport-9s44-profile-stop-tests.log`.
Canonical host and Windows GNU lint each pass all-features and no-default-features workspace/all-target checks.
Windows runtime execution, native foreground input, and the working-session CPU profile remain unqualified.
