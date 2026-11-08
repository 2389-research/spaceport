# T01 development identity evidence

T01 is in progress. A42 is not passed.

## State directory boundary

`sheprd_client::local_path::state_dir_with` now selects `<XDG_STATE_HOME>/spaceport`, or the platform fallback root plus `spaceport`. The app's preferences, diagnostics, and Unix control socket derive their locations from this function. Existing `herdr/gpui` state is not migrated or read by this function. The Windows control pipe has a separate name and still needs isolation.

The existing path test was changed to require the independent directory. It failed before implementation with `state/herdr/gpui` versus `state/spaceport`, then passed. `cargo +1.99.0 test --locked -p sheprd-client --lib` passed: 396 active tests, 2 ignored, on macOS. Windows path assertions were updated but have not run on Windows.

## GUI configuration path

`Config::path()` now derives `<config root>/spaceport/config.toml`, with local overrides at `<config root>/spaceport/config.local.toml`. The temporary mod path also resolves under the Spaceport root until that excluded feature is removed. The path test failed to compile before `app_config_path` was added, then passed. Ten existing config loading tests passed. The test link showed the same compact-unwind warning seen in T00 and the `block` future-compatibility notice; this change added no new warning.

The separate daemon settings reader still resolves upstream `herdr/config.toml` and can influence the GUI. The updater still has upstream app and release identities. No side-by-side or native identity acceptance has been claimed.
