# T01 development identity evidence

T01 is in progress. A42 is not passed.

## State directory boundary

`sheprd_client::local_path::state_dir_with` now selects `<XDG_STATE_HOME>/spaceport`, or the platform fallback root plus `spaceport`. The app's preferences, diagnostics, and Unix control socket derive their locations from this function. Existing `herdr/gpui` state is not migrated or read by this function. The Windows control pipe has a separate name and still needs isolation.

The existing path test was changed to require the independent directory. It failed before implementation with `state/herdr/gpui` versus `state/spaceport`, then passed. `cargo +1.99.0 test --locked -p sheprd-client --lib` passed: 396 active tests, 2 ignored, on macOS. Windows path assertions were updated but have not run on Windows.

## GUI configuration path

`Config::path()` now derives `<config root>/spaceport/config.toml`, with local overrides at `<config root>/spaceport/config.local.toml`. The temporary mod path also resolves under the Spaceport root until that excluded feature is removed. The path test failed to compile before `app_config_path` was added, then passed. Ten existing config loading tests passed. The test link showed the same compact-unwind warning seen in T00 and the `block` future-compatibility notice; this change added no new warning.

## Windows control pipe name

The Windows pipe name now begins `\\.\pipe\spaceport-control-`, retains the user SID and the deterministic explicit-state-root hash, and uses the existing owner, DACL, first-instance, and remote-client refusal policy. A cross-platform pure test failed to compile before the naming function existed, then passed with the new name and path spelling cases. The Windows listener and client checks have not run on Windows.

## Private view settings

The GUI's inherited view settings reader now resolves `<config root>/spaceport/view.toml`; `HERDR_CONFIG_PATH` cannot redirect it to upstream. Its callers for config loading, sound, settings persistence, CLI paths and diagnostics share that path. The new test failed to compile before the path existed, then passed. The adjacent config and sound suite passed 275 tests with one ignored after one old sound-path assertion was updated. A macOS test binary emitted the same compact-unwind linker warning seen at baseline. Windows path expectations were updated but have not run on Windows.

The built development CLI, run with `XDG_CONFIG_HOME=/tmp/spaceport-cli-config-fixture`, returned `local`, `managed`, and `view` JSON paths all under that root's `spaceport/` directory. The path command did not create the fixture root. Its text, help and `docs/cli.md` were updated to describe the Spaceport paths. The executable and common CLI help footer still say Sheprd; those identity changes remain open. The legacy remote Herdr settings module still has its own remote `herdr/config.toml` path and must be removed with the Herdr 1 feature unit.

The updater still has upstream app and release identities; a separate compile-time gate now blocks release-version builds until those identities are replaced and verified. No side-by-side or native identity acceptance has been claimed.

## Running development GUI identity

The development GUI now uses `spaceport.development` for its Windows AppUserModelID (notifications, taskbar relaunch and Jump List) and Linux Wayland app ID / X11 WM_CLASS. Its window and OS notification name is `Spaceport`. These are source-level development placeholders, not owner-controlled production identifiers. The updater's `APP_ID`, release URLs, installer IDs, package desktop entry, and compile-time `HERDR_RELEASE_VERSION` gate are unchanged.

TDD: `cargo +1.99.0 test --locked -p sheprd-gpui --bin sheprd constants::tests::running_development_app_has_its_own_os_identity -- --exact` failed before the code change with `left: "so.pen.herdr-gpui"`, `right: "spaceport.development"`, then passed. With the same toolchain and `--locked`, `constants::tests::` passed 2 tests, `taskbar::tests::` passed 3, the exact `endpoint::lifecycle_tests::window_notices::system_notification_click_opens_its_origin_and_rejects_a_reconnect` test passed, and `window::tests::` passed 184. The pre-existing macOS compact-unwind linker warning and `block` future-compatibility notice appeared; no new diagnostic was observed.

No native GUI, Linux desktop shell, or Windows notification/taskbar run was performed. A development Linux window intentionally does not match the inherited upstream desktop entry, so desktop icon matching still needs an independent entry. The Windows notification deep-link scheme still uses the upstream handler; the running development app's OS ID is isolated, but protocol activation and package identity remain release work. A42 stays not run and G-RELEASE stays open.
