<!-- ABOUTME: Records the source-backed diagnosis of Linux and Windows shortcut collisions. -->
<!-- ABOUTME: Separates verified dispatch behavior from pending native layout and desktop checks. -->

# Platform shortcuts

Inspected on 2026-11-17 for the default-shortcut report (`3wd4`).
Doctor Biz authorized the scoped default fixes and their validation.
The original implementation authorization excluded publication; the later aggregate draft authorization covers qualification updates.
Doctor Biz requested native keyboard checks remain pending.

## Shifted symbols

Pinned GPUI is `gpui-pre` 0.3.10.
Its Linux XKB conversion removes Shift from one-character nonletters.
Its Linux platform returns `DummyKeyboardMapper`.
That mapper leaves configured key names unchanged.
Spaceport's non-macOS `window::prefix::current_layout` also returns `None`.
Therefore a delivered `ctrl-!` cannot match the Linux default `ctrl-shift-1`.
The same gap affects shifted punctuation whose configured name differs from the delivered symbol.
Existing `keymap/tests/platform_defaults.rs` mostly supplies canonical configured names.
It proves configuration coverage, rather than native event normalization.
The font defaults already include Linux symbol aliases.

Windows has a real `WindowsKeyboardMapper`.
Its `map_key_equivalent` maps shifted digits and punctuation even with key equivalents disabled.
The raw shortcut routing temporarily bypassed that mapper.
The held-shortcut fix now delegates to it again.
See [shortcut repeat evidence](review-shortcut-repeats.md) for that regression's red and green results.
The broader report's Windows shifted-symbol claim therefore needs separate native verification.

## Character input

Pinned Windows event handling compares translated text with and without Ctrl/Alt/Win.
It marks character-producing combinations with `prefer_character_input`.
The raw shortcut owner yields direct catalog and custom shortcuts when that flag is set.
Real focused-field tests cover fresh direct input and captured direct repeats.
Explicit prefix and resize modes retain their chosen input ownership.
Those headless tests do not prove a Polish keyboard's native AltGr events.
Pinned Linux X11 and Wayland key events always set the preference flag to false.
No native Linux AltGr claim follows from the Windows fix.

## Terminal and desktop keys

The initial non-macOS catalog bound F6, Shift-F6, F11, and Ctrl-PageUp/Down directly.
Those catalog actions consumed their matching events before terminal input.
F6 and Shift-F6 intentionally moved between accessible window regions.
The correction must preserve a discoverable keyboard route for that behavior.
Herdr's prefix routes already cover tab navigation and terminal actions.
Window-only actions remain discoverable in the command palette and keybinding settings.

GNOME interception of Ctrl-Alt-Shift combinations remains unverified.
An app-side fixture cannot prove whether the desktop delivers those combinations.
Do not infer native layout support from headless matching tests.

## Scoped correction

Prefer a real platform layout capability over guessed US symbol aliases.
GPUI's public Linux mapper currently supplies no such conversion.
Its public keyboard-layout interface exposes only the layout name and identifier.
Linux now leaves the proven unmapped shifted-symbol defaults unbound.
The shared Ctrl catalog still supplies Windows' working mapped symbols.
The Linux wrapper preserves every other shared default, including font aliases.
Retained Herdr prefix routes cover pane cycling, tab digits, Settings, and Keyboard Shortcuts.
Back, Forward, Split Editor, and Cycle Windows remain available through the palette and explicit configuration.

Both Ctrl profiles now leave F6, Shift-F6, F11, and Ctrl-PageUp/Down to pane programs.
Herdr's existing prefix N/P still navigate tabs.
Spaceport-specific private-view entries retain prefix F6, prefix Shift-F6, and prefix F11.
They dispatch Focus Next Region, Focus Previous Region, and Full Screen through the resolved window keymap.
They are not presented as Herdr TUI defaults.
macOS catalog defaults remain unchanged.
The accessibility owner confirmed no dependency on the exact F6 spelling.
The Windows native accessibility fixture now types the prefix before its existing region commands.

## Regression evidence

The Linux profile regression failed on `next_pane` before the wrapper.
The unchanged Windows-symbol case passed before and after the wrapper.
Both portable profile cases now pass.
Logs: `/tmp/spaceport-qnz9/linux-default-symbols-red.log` and `linux-default-symbols-green.log` in the same directory.

The pane-key profile regression failed because F6 belonged to Focus Next Region.
The private-view parser regression failed because the new chrome action was absent.
Real prefix-region dispatch also failed before the parser correction.
The real framed pane-wire preservation case already passed on the macOS host.
Log: `/tmp/spaceport-qnz9/platform-pane-keys-red.log`.
The command suite passed all 38 tests after the default corrections.
The initial GPUI green batch stopped at concurrent sidebar implementation type errors.
Log: `/tmp/spaceport-qnz9/platform-pane-keys-green.log`.
Their owner fixed those errors; the focused retry then passed 60 keymap tests.
Log: `/tmp/spaceport-qnz9/platform-pane-keys-green-retry.log`.

The first focus fixture checked before the existing two-frame handoff completed.
Its initial correction wrongly assumed that drawing delivered platform frame callbacks.
Pinned GPUI documents that headless tests must call `simulate_next_frame` explicitly.
The fixture now reuses the existing sidebar `run_frames` helper for exactly two callbacks.
It asserts actual title-bar focus and subsequent return to the terminal.
No production focus behavior changed.
The failed fixture runs remain in `platform-pane-keys-green-retry.log` and `platform-shortcuts-final.log` under the same directory.

The final batch exited successfully.
It passed both real pane-wire and focus cases, 61 keymap tests, 15 repeat contracts, and five terminal-layout cases.
The layout cases include existing AltGr text preservation and ordinary chord forwarding.
Commands and GPUI all-target, all-feature Clippy passed with `-D warnings` and the validated pinned Sparkle SDK.
Log: `/tmp/spaceport-qnz9/platform-shortcuts-final-frames.log`.
Focused rustfmt, file-size, sorted-region, and diff checks passed.
The aggregate formatting check found concurrent accessibility implementation hunks.
Their owner retains that formatting work.
Independent review approved the default profile changes and requested custom-chord compatibility before final approval.

## Existing custom chords

The new Spaceport-only prefix defaults initially shadowed existing daemon custom chords.
Both the reachability regression and the real dispatch fixture failed with `Shadowed` instead of `Runs`.
Log: `/tmp/spaceport-qnz9/spaceport-prefix-precedence-red.log`.

The three rows now identify themselves once as `Target::Spaceport`.
The parser retains only omitted/default Spaceport command IDs as fallback ownership.
Explicit private-view assignments never carry that ownership.
A single shortcut origin distinguishes catalog defaults, Spaceport defaults, and owned bindings.
Herdr-native defaults and explicit assignments remain owned.
Both custom reachability and raw dispatch consult the same chord ownership lookup.
The private platform resolver lets descendant fixtures exercise the real Ctrl defaults on this macOS host.
It exposes no additional public API.

The focused tests passed after the correction.
They cover all three default custom chords, explicit private-view assignments, GUI assignment precedence, and Herdr-native help/workspace-close defaults.
The expanded real peer checks each default and explicit assignment through actual raw dispatch.
Ordered input markers prove that explicit assignments send no custom invocation.
The initial focused green log is `/tmp/spaceport-qnz9/spaceport-prefix-precedence-green.log`.
The final batch exited successfully.
It passed the two precedence cases, 62 keymap cases, 12 prefix cases, and 15 integration repeat cases.
The precedence fixture covers six real dispatch combinations across the three new defaults.
GPUI all-target, all-feature Clippy passed with `-D warnings` and the validated pinned Sparkle SDK.
Log: `/tmp/spaceport-qnz9/spaceport-prefix-precedence-final.log`.
Aggregate `cargo fmt --all --check`, file-size, sorted-region, and diff checks passed.
The integrating owner independently approved the final ownership correction with no material findings.
The review covered origin derivation, explicit/native precedence, shared reachability and dispatch, held-key ownership, profiles, and wire tests.
The scoped fix is implemented and reviewed.
Native Linux/Windows keyboard delivery, GNOME interception, and performance qualification remain pending.

## Verification boundary

Native Linux and Windows keyboard checks and desktop interception remain pending.
The original issue adds no working-session performance prerequisite.
Portable profiles prove both platforms' catalog selection and GPUI parsing.
Headless dispatch and actual framed peers prove application routing on this host.
They do not prove native OS key delivery or unseen platform `cfg` branches.

## Native route research, November 19

The fresh revision-one issue explicitly requires Linux and Windows keyboard qualification.
Windows-only results cannot close that scope.
Use the main window's Search spaces field to exercise the actual main shortcut owner.
The separate Settings window bypasses that owner.

Microsoft lists [Polish (Programmers)](https://learn.microsoft.com/en-us/globalization/keyboards/kbdpl1) as KLID `00000415`.
Native qualification must verify the owned GUI thread's actual keyboard layout.
A background helper cannot assume its own layout changed the GUI.
[LoadKeyboardLayoutW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadkeyboardlayoutw) documents foreground restrictions and Windows 8 system-wide behavior.
Use the owned window's [layout-change request](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-inputlangchangerequest), verify the result, and restore the original layout.
Derive scan codes with [MapVirtualKeyExW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-mapvirtualkeyexw).
Derive expected translated text with [ToUnicodeEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-tounicodeex) without mutating keyboard state.
Unicode packet injection cannot qualify AltGr translation.

GNOME 46 defines concrete conflicts with the retained shared Ctrl profile.
Its [workspace movement bindings](https://github.com/GNOME/gsettings-desktop-schemas/blob/46.0/schemas/org.gnome.desktop.wm.keybindings.gschema.xml.in#L241) use Ctrl+Alt+Shift+arrows.
Its [recording binding](https://github.com/GNOME/gnome-shell/blob/46.0/data/org.gnome.shell.gschema.xml.in#L287) uses Ctrl+Alt+Shift+R.
The inspected pre-correction Linux wrapper retained those Focus and Resume Palette defaults.
This source evidence establishes a default conflict, without claiming interception in an untested local session.

Current Linux CI has no qualified GNOME session.
Xvfb/XTest can qualify an X11 application route, but cannot establish GNOME or Wayland interception.
No layout change, native input, or desktop launch occurred during this read-only research.
Kata `3wd4` remains open.

## GNOME default correction

GNOME 46's source confirms all four workspace-movement conflicts and the recording conflict above.
The Linux wrapper now leaves the four direct pane-focus shortcuts unbound.
Existing Herdr prefix then h/j/k/l still focuses the corresponding pane.
Resume Palette uses Ctrl-Shift-R on Linux, since that command is deliberately excluded from the palette itself.
Windows retains the shared Ctrl defaults, and macOS keeps its original defaults.
Explicit shortcut configuration keeps its existing precedence.
The generated default fixture changes only those five Linux rows.
README documents the retained focus routes and the new Resume binding.

The reserved-chord and bounded-profile regressions fail first on Focus Left.
The separate Resume regression fails first with Ctrl-Alt-Shift-R instead of Ctrl-Shift-R.
Logs: `/tmp/spaceport-9s44-gnome-shortcuts-red.log` and `gnome-resume-red.log`.
The complete commands scope then passes forty-one tests, with none ignored.
Log: `/tmp/spaceport-9s44-gnome-commands-final.log`.
The fixture generator initially caught the five intended differences; its reviewed update retains all other rows.
The Linux-only keymap case also checks all four retained prefix routes and the new Resume binding.
The host keymap, prefix dispatch, and palette-resume scope passes seventy-three tests, with none ignored.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- keymap:: window::prefix::tests:: palette::tests::resume:: --nocapture`.
Log: `/tmp/spaceport-9s44-gnome-consumers.log`.
Independent review finds no material discoverability, platform, precedence, or profile-boundary issue.
That cfg-specific case requires Linux CI; a macOS pass cannot qualify its branch.
Native Linux/Windows keyboard delivery and actual GNOME session checks remain pending.
This correction does not close `3wd4`.

## Loaded-layout prerequisite diagnostic

The Windows native suite now has a separate loaded-layout diagnostic.
It creates only a private PowerShell helper, without an app, daemon, clipboard operation, or input event.
It enumerates at most 64 actual HKLs and identifies their eight-digit KLIDs.
One synchronous C# call temporarily activates each candidate on its own calling thread with flags zero.
It verifies the active HKL before reading its name and restores the original HKL in `finally`.
The diagnostic also exercises an intentional early failure and verifies that restoration succeeds.
Its receipt says whether that failure actually changed layouts; a single-layout host cannot prove that transition.

The [activation contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-activatekeyboardlayout) limits this operation to the calling thread without `KLF_SETFORPROCESS`.
The diagnostic never calls LoadKeyboardLayout or changes the GUI thread.
Its bounded receipt lists loaded KLIDs and whether Polish Programmers `00000415` is present.
An absent layout remains an unavailable input-test prerequisite, rather than a passing AltGr result.

At that checkpoint, native execution remained pending; Windows cross-lint does not compile or execute the embedded C#.
This diagnostic does not qualify actual key delivery or close `3wd4`.
Independent read-only review finds no material thread-scope, cleanup, bounds, or framework-compatibility issue.
The canonical static checks, focused Windows native-harness cross-lint, and diff check pass.
Logs: `/tmp/spaceport-9s44-layout-inventory-static.log`, `layout-inventory-canonical.log`, and `layout-inventory-windows-lint.log`.

## Shortcut catalog integration, November 19

Main `e13b88c30` includes PR 134's symbol aliases and revised Ctrl shortcuts.
The review branch keeps those aliases instead of leaving Linux symbol commands unbound.
Canonical and symbol spellings follow the existing font-key catalog pattern.
The Linux wrapper still excludes GNOME's four window-movement chords and uses Ctrl-Shift-R for Resume.
Windows reads the shared Ctrl catalog directly, preserving its separate platform policy.
The existing character-input guard and Windows keyboard-mapper correction remain unchanged.

Direct Ctrl-F6, Ctrl-Shift-F6, Ctrl-Shift-F11, and Ctrl-Shift-PageUp/PageDown coexist with retained private prefix routes.
Bare pane keys remain unbound by the app's default catalog.
Explicit private-view assignments and daemon custom chords keep their existing precedence.
The native held-window fixture now uses Ctrl-Shift-N and retains its ownership and repeat-count assertions.
The native chrome fixture uses Ctrl-F6 through the existing isolated `Running` owner.

The incoming snapshot first fails on retained GNOME exclusions and the task's Close Workspace rows.
The existing `UPDATE_SHORTCUT_FIXTURE` producer regenerates it from the resolved catalog.
The command suite then passes 46 tests with none ignored.
The focused keymap, prefix, browser, group, tab, Zen, and Core consumer run passes 334 tests with none ignored.
Independent review finds no material shared-contract or browser-ownership issue.

Crew records 3wd4 as done after PR 134 merges.
Its source note explicitly records that no native keyboard check ran.
The review squad asks Clint for any newer qualification evidence without changing his task status.
Native Linux shortcut delivery, non-US layouts, and actual GNOME interception remain unverified here.
On `7b7498762`, [Windows native qualification](https://github.com/2389-research/spaceport/actions/runs/44624169019/job/135913747987) passes Ctrl-F6 navigation and held Ctrl-Shift-N.
The held-window case preserves three windows after two fresh presses and ten held events.
All four Linux CI configurations also pass on that source.
The native layout inventory still contains only US `00000409`; Polish Programmers remains unavailable.
Those results do not qualify non-US keyboard delivery or actual GNOME interception.

## Disposable Windows non-US qualification route

Read-only research finds a supported setup path without changing Doctor Biz's machines.
Use a disposable Windows CI session and its existing Polish Programmers keyboard profile.
Microsoft identifies that profile as `0415:00000415` and its KLID as `00000415`.
[InstallLayoutOrTip](https://learn.microsoft.com/en-us/windows/win32/tsf/installlayoutortip) can enable it in the current session.
Do not infer activation from saved language preferences or install a display-language pack.
The actual CI prerequisite remains unexecuted.

Enumerate actual HKLs and identify that exact KLID before sending input.
Post the documented layout request to the exact owned GUI HWND.
Verify the target GUI thread's actual HKL before continuing.
Use scan-code RightAlt+L and RightAlt+Shift+N in the real Search spaces field.
Require the actual translated `łŃ` value, unchanged runtime identities, and no Workspace dialog.
The shifted N case exercises the current Ctrl-Alt-Shift-N default against character preference.
Restore the GUI's original layout and remove only a profile added by this disposable test.
Never substitute Unicode packet input for native translation.

Chromium's [scoped layout helper](https://github.com/chromium/chromium/blob/main/ui/events/test/keyboard_layout_win.cc) supports explicit activation and restoration.
VS Code's saved-layout mapper tests remain synthetic and cannot prove actual key delivery.
No external code is copied, and no layout, setting, runtime, or desktop changed during this research.
Implementation and actual Windows qualification remain pending; Linux/GNOME qualification remains separate.
Doctor Biz explicitly keeps Mac foreground checks pending.

### Disposable AltGr implementation and verification plan

The implementation uses the existing isolated Windows accessibility harness.
`accessibility_windows/non_us.rs` owns its private app, daemon, observer snapshots, and absolute child deadline.
`support/uia_altgr.ps1` owns native field focus, actual text assertions, and the intentional cleanup failure.
`support/native_altgr.cs` owns the enabled profile, GUI-thread HKL, scan-code keys, and restoration.
This changes test infrastructure only; production shortcut behavior remains unchanged.

- [x] Separate ordinary and non-US CI selection, using actual Cargo invocation tests.
- [x] Verify Rust harness compilation in the Windows all-feature configuration.
- [x] Review HWND/thread ownership, Win32 ABI, key release, and profile restoration independently.
- [x] Verify the Windows minimal harness configuration and final quick checks.
- [x] Execute the stock-profile prerequisite and both cleanup paths on disposable Windows CI; see `3a0bcb00d` below.
- [x] Verify native `łŃ`, no Workspace dialog, stable runtime identities, and complete cleanup on `3a0bcb00d`.

The scheduling regression fails first because the separate step and namespace exclusion are absent.
After wiring both, all three scheduling cases pass.
All eleven existing CI policy cases pass.
Both Windows harness feature configurations pass Clippy with warnings denied.
Formatting, file sizes, sorted regions, operations, localization, commit subjects, and diff checks pass.
Logs: `/tmp/spaceport-9s44-non-us-{scheduling-red,scheduling-green,ci-policy,windows-all,windows-minimal,static}.log`.
The original ordinary namespace loses no fixtures: every deferred namespace has its own supported-runner step.

The Rust test refuses session setup outside a GitHub-hosted Windows runner.
The helper independently requires the fixture's explicit disposable-session marker.
[EnumEnabledLayoutOrTip](https://learn.microsoft.com/en-us/windows/win32/tsf/enumenabledlayoutortip) records enabled profiles before mutation.
Unknown keyboard profile formats fail before setup.
Only an absent Polish profile is enabled; no default profile, language pack, or owner configuration changes.
Normal cleanup and an intentional exception restore the original GUI HKL and enabled profile set.
An added loaded HKL is removed only when absent before the test.
The watchdog releases owned keys first and attempts the same cleanup within the parent's five-second reserve.
A forced timeout remains a failed fixture and does not prove complete restoration.

The Windows results below record actual C# compilation, profile setup, translation, and cleanup.
Cross-compiling the Rust harness embeds these helpers; it does not execute or compile their C# code.
Linux/GNOME and Mac native qualification remain separate.

### Linux display prerequisite research

The open Kata finding still requires actual Linux keyboard and desktop qualification.
GNOME 46's [nested-session documentation](https://github.com/GNOME/gnome-shell/blob/46.0/docs/building-and-running.md#running-a-nested-instance) warns that outer compositors can intercept shortcuts.
A nested Shell alone cannot establish delivery of the disputed desktop chords.

GNOME's [GTK CI change](https://mail.gnome.org/archives/commits-list/2022-May/msg05630.html) runs Mutter on Xvfb inside a private D-Bus session.
That supplies an actual X11 compositor route, but does not itself qualify GNOME Shell behavior.
GNOME 46's [Mutter source](https://github.com/GNOME/mutter/blob/46.0/src/core/meta-context-main.c) also supports a headless Wayland backend and virtual monitors.
Those modes are alternatives; the source rejects combining X11 with its headless or nested flags.

Any Linux fixture must prove which compositor owns its isolated display before sending native keys.
It must retain the exact app window, runtime identities, key cleanup, and display-process ownership.
The existing runtime stager supplies Windows packages only; an explicit trusted Linux Herdr fixture remains a prerequisite.
No Linux desktop, runtime, configuration change, or new CI job ran during this research.
These sources narrow the next setup experiment; they do not qualify the remaining shortcut requirement.

### First native profile prerequisite on `3634e5039`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44671802770/job/136074462029) compiles the C# helper and reaches profile enumeration.
It refuses an ID that violates the helper's bare eight-character assumption.
The refusal happens before enabling a profile or sending AltGr input.
The old diagnostic does not reveal the actual returned string format.

The candidate parser accepts bounded bare KLIDs and language-qualified profile-list forms, with optional hexadecimal prefixes.
A supplied language must match the native `LANGID`; unknown forms still fail before mutation.
Cleanup retains the original raw profile identities rather than their parsed presentation.
Normalized technical IDs are reported when parsing succeeds.
This is a guarded parsing trial, not a claim about the unobserved Windows string or successful activation.

Real C# tests reproduce rejection of a language-qualified input under the old guard.
The final parser passes five accepted forms and ten malformed or mismatched cases.
Mono also compiles the complete native helper beside those real tests.
The private Linux containers use a read-only source mount and remove themselves after each run.
Their exact IDs are verified absent afterward; no desktop or Windows API executes in those tests.
Logs: `/tmp/spaceport-9s44-profile-parser-{red,green,final}.log`.
The runtime is Mono `6.8.0.105+dfsg-3.6ubuntu2` in Ubuntu 24.04 on Linux ARM64.
The final image is pinned at `sha256:ab18ecc70c55c703eae968948f9f1dbbe662aa8d9badac7c982e6a6237b9d74c`.
The later `3a0bcb00d` native run qualifies the Windows path below.

### Actual Windows non-US qualification on `3a0bcb00d`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44677192575/job/136091920328) passes the Polish Programmers fixture.
Real scan-code input produces `łŃ` in the owned native search field.
The shifted AltGr shortcut opens no workspace dialog and preserves exact runtime identities.
The helper restores the GUI layout, original enabled profiles, and released keys.
It verifies cleanup after both normal completion and an intentional exception.
The actual receipt appears at `2026-11-20T05:06:24.5175055Z`; the Rust test also passes.
See the [scoped receipt](review-squad/windows-native/qualification-3a0bcb00d.json).

This qualifies this disposable Windows session and Polish layout, without claiming all keyboard layouts.
The same job passes all 25 ordinary UIA cases, including the chat-action correction.
It separately fails Core resize geometry; the overall job remains red.
Linux non-US input and real GNOME interception remain pending, so Kata `3wd4` stays open.

### Disposable GNOME prerequisite probe, November 20

The pinned Ubuntu ARM64 container installs real GNOME Shell 46, Mutter, and Mesa packages.
The headless command creates a surfaceless renderer, virtual monitor, and private Wayland display.
GNOME Shell then exits with signal 11 while initializing its login-manager connection.
A real private system D-Bus moves the failure to unavailable `org.freedesktop.login1` activation.
The ordinary container does not provide the working login session this package expects.
No Spaceport process, keyboard input, or shortcut assertion runs.
This does not qualify GNOME interception or indicate a Spaceport regression.

Both exact Shell children and the private bus are reaped.
The named container is stopped, deleted, and freshly verified absent.
The Mac desktop, account settings, and unrelated containers remain untouched.
The [scoped receipt](review-squad/linux-native/gnome-prerequisites-3a0bcb00d.json) records versions and failure boundaries.
Logs remain `/tmp/spaceport-9s44-gnome-{prerequisites,compositor,system-session,cleanup}.log`.
A future route needs a working isolated GNOME login session, followed by actual app input checks.

### Remote GNOME startup prerequisite on `1ab605fde`

The separate branch-scoped [probe run](https://github.com/2389-research/spaceport/actions/runs/44686213368) passes on a disposable Ubuntu 24.04 runner.
Actual logs identify GNOME Shell 46.0 and Mutter 46.2 with a surfaceless renderer.
The private session publishes virtual monitor `Meta-0`, socket `wayland-0`, and the Shell bus owner.
Rust directly owns and reaps the non-forking private bus, Shell, and query processes.
The [scoped receipt](review-squad/linux-native/gnome-startup-1ab605fde.json) preserves the source, times, versions, warnings, and cleanup boundary.
RealtimeKit and GNOME SessionManager remain absent; the probe does not establish full desktop readiness.
It starts no Spaceport process and sends no keys.
The next prerequisite uses a private GTK field and Mutter's persistent input session to test Polish evdev input.
Spaceport shortcut behavior and actual GNOME interception remain unqualified, so Kata `3wd4` stays open.

The first GTK input attempt on `c3f01ab84` fails before remote-session creation or key injection.
GJS exposes `Gtk.Window:is-active` as a Boolean property; the probe incorrectly calls it as a method.
The [failed-attempt receipt](review-squad/linux-native/gtk-input-c3f01ab84.json) retains the exact boundary and cleanup observations.
The private layout restores, and the input driver, Shell, and bus all reap.
The correction reads the property and reports the selected transport separately from actual sequence delivery.
The [actual rerun on `942a5f061`](review-squad/linux-native/gtk-focus-942a5f061.json) passes that API boundary but times out with an inactive window.
GTK's [`present` contract](https://docs.gtk.org/gtk4/method.Window.present.html) does not guarantee compositor keyboard focus.
The next attempt requests native Alt-Tab activation after the owned window maps.
It retains the active-window guard, exact text requirement, original deadline, and cleanup checks.
No Spaceport keyboard qualification follows from either failed attempt.
The [Alt-Tab attempt on `d793d0287`](review-squad/linux-native/gtk-activation-d793d0287.json) also times out with a mapped, inactive window.
It acknowledges the activation sequence, closes the remote session, restores the private layout, and reaps all direct children.
The next diagnostic reads Shell `Mode`, `OverviewActive`, and `ShellVersion` before choosing another activation change.
Child-log timestamps identify their later bounded emission by the parent, rather than exact event times.
The [actual diagnostic on `63db1a89c`](review-squad/linux-native/gtk-shell-state-63db1a89c.json) reports user mode with the overview open at mapping and deadline.
GNOME 46 installs the Alt-Tab handler only for normal action mode.
The next correction closes the private overview through its supported UI property and verifies closure before native activation.
The original active-window, exact-text, deadline, and cleanup requirements remain.
The [actual rerun on `8e43cd345`](review-squad/linux-native/gtk-input-8e43cd345.json) passes the GTK native-input prerequisite.
The private overview changes from true to false before activation.
Native evdev Right Alt plus L, then Shift plus Right Alt plus N, produces `łŃ`.
Key release, remote-session closure, private layout restoration/readback, and all direct process reaps pass.
Spaceport has not started in this route; its keyboard normalization, shortcuts, and terminal delivery remain pending.
Mutter-only or nested startup cannot substitute for those checks.
