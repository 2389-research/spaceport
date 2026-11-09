# T11: native macOS check while loginwindow owned focus

Integrated revision `ee21e53af` (before the source-only profile dependency
note). This is an attempted native check, not A49 acceptance. The fixture used
`crates/sheprd-gpui/tests/live_gui.rs` with a private HOME/XDG/socket sandbox,
no daemon and no personal Core. Its executable was built with Rust 1.99.0,
`--locked --offline`, and `CARGO_INCREMENTAL=0` before the visible run.

`CARGO_INCREMENTAL=0 just test-sidebar` first stopped before launch because
the shell selected Rust 1.97.1 while the workspace requires 1.99.0. With
`RUSTUP_TOOLCHAIN=1.99.0 CARGO_INCREMENTAL=0 just test-sidebar`, all three
matching fixtures launched. The ordinary `native_sidebar` and the intentional
invalid-settings fixture failed at `Settings shortcut was not handled`; the
intentional invalid-paint fixture passed its expected failure assertion.
Before that input step, the live window reported a valid 512-point app icon,
Dock badge changes, repeated sidebar child geometry, installed Nerd Font
symbols, and native bidirectional text order. Those partial observations do
not qualify the whole sidebar, Settings, or retained UX.

A diagnostic-only build confirmed that the source Settings binding was
`cmd-,`, GPUI displayed `⌘,` for it, and the source view had focus, but its
window was inactive. Activating the app/window and waiting three seconds did
not make it active. A separate diagnostic run allowed the keystroke dispatch
to continue; it returned false, the action counter stayed zero, and no
Settings window opened. The existing headless Settings lifecycle suite then
passed 7/7, including its shortcut scenario. All diagnostic source edits
were removed; the integration worktree is unchanged by them.

The independent `native_pages` fixture also failed before pointer input. Its
AppKit report said the fixture window was visible, activation policy Regular,
app inactive, and frontmost PID 427. A read-only process lookup identified
PID 427 as `loginwindow`. An earlier System Events frontmost-name query had
reported Helium, so that name was not a reliable unlock check in this
session. The same `loginwindow` condition is recorded at an earlier revision
in [the Pages activation note](t11-native-pages-activation.md). This run does
not establish a Spaceport shortcut or Pages product regression. It also does
not prove those paths work when the desktop is unlocked.

The next run needs Doctor Biz to unlock the Mac briefly, then confirm the
frontmost PID is no longer `loginwindow` before launching the prebuilt fixture.
Do not blindly retry while `loginwindow` owns focus. Native macOS acceptance
A49 stays `not_run`; G-PLATFORM remains open. Linux and Windows were not run.
