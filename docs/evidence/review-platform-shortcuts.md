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
The Linux wrapper still retains those Focus and Resume Palette defaults.
This source evidence establishes a default conflict, without claiming interception in an untested local session.

Current Linux CI has no qualified GNOME session.
Xvfb/XTest can qualify an X11 application route, but cannot establish GNOME or Wayland interception.
No layout change, native input, or desktop launch occurred during this read-only research.
Kata `3wd4` remains open.
