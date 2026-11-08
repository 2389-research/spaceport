# T11: macOS native page fixture activation

At integrated Spaceport revision `c3cfdf82`, the existing `native_pages`
integration fixture built and launched a real macOS window using the shared
private sandbox in `crates/sheprd-gpui/tests/live_gui.rs`. It had no daemon,
personal app state, or Core process. The pinned Core 0.1.18 binary was built
separately but was not used by this fixture.

Command:

```sh
CARGO_TARGET_DIR=/tmp/spaceport-t04-install-offer-target CARGO_INCREMENTAL=0 \
  cargo +1.99.0 test --locked --offline -p sheprd-gpui \
  --features integration-test --test live_gui native_pages -- \
  --ignored --nocapture --test-threads=1
```

The test binary linked and the fixture started, then exited 1 after 7.68 s.
Its recorded reason was: `PAGES native FAIL: the window never became active,
and WebKit ignores the pointer in an inactive window`. The Rust test reported
0 passed and 1 failed. The build also emitted the already recorded macOS
`__eh_frame` linker warning and `block v0.1.6` future-compatibility notice;
there was no new Rust source warning.

The same fixture was then run once with a diagnostic-only AppKit probe at the
activation deadline. The probe uses safe `objc2-app-kit` methods enabled only
for `integration-test`; it logs process IDs, not the frontmost app's user data.
The first diagnostic compile needed the binding's `libc` feature for
`processIdentifier`, adding only that dependency edge to `Cargo.lock`. The
subsequent `--locked --offline` build linked and ran the fixture. Its
[raw output](t11-native-pages-diagnostic.log) records a failure after 7.14 s
with `app_active=false`, `app_hidden=false`,
`activation_policy=NSApplicationActivationPolicy(0)` (Regular),
`window_visible=true`, `fixture_pid=91989`, and `frontmost_pid=Some(427)`.
A read-only process lookup named PID 427 `loginwindow` at that time. The test
reported 0 passed and 1 failed, with the same existing linker and Cargo
notices and no new Rust source warning.

Both runs stopped before page pointer interaction, so neither establishes
embedded-page behavior or the current Core group UI. The frontmost system
process explains why the fixture could not receive pointer input in this
session, but it does not prove why `loginwindow` owned the foreground or
classify a fork regression. The next native check needs an unlocked, idle
desktop and the pinned-upstream comparison described in the
[decoy audit](t11-sidebar-activation-audit.md). A49 remains `not_run` and
G-PLATFORM open.
