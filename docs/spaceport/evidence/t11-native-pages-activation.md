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

This stopped before page pointer interaction, so it does not establish
embedded-page behavior or the current Core group UI. It matches the earlier
decoy fixture's observation that the test app stayed inactive, but neither
run identifies the frontmost process or proves whether the focus cause is
inherited desktop state or a fork change. The test was not retried. The next
native check needs the process/window activation facts listed in the
[decoy audit](t11-sidebar-activation-audit.md), with an otherwise idle desktop
and a pinned-upstream comparison. A49 remains `not_run` and G-PLATFORM open.
