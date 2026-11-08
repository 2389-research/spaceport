# T01 development identity evidence

T01 is in progress. A42 is not passed.

The root README now opens with Spaceport's development status and links to the
authoritative specification, task and capability records, and release audit.
It labels the remaining inherited Sheprd README as historical reference, since
that material still names upstream releases and features being removed. The
source links resolve in this checkout and `git diff --check` passed. This is
documentation clarity, not an independent product identity or native install
test.

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

With `HERDR_RELEASE_VERSION=20261107.0`, `cargo +1.99.0 check --locked -p sheprd-updater` failed as required at the existing compile-time assertion: `Spaceport release identity is not configured` (E0080). A normal development build passed through the targeted GUI tests above.

No native GUI, Linux desktop shell, or Windows notification/taskbar run was performed. A development Linux window intentionally does not match the inherited upstream desktop entry, so desktop icon matching still needs an independent entry. Protocol activation and package identity remain release work. A42 stays not run and G-RELEASE stays open.

## Development deep links

From parent revision `ca1167065fda163d3152046739709c431f7e70b0`, the development GUI's link parser and writer now use only `spaceport://`. The local CLI accepts that scheme as its sole link argument, keeps the exact decoded device ID, and rejects inherited `sheprd://` and `herdr-gpui://` links. Windows toast XML and Jump List entries get the new scheme from the same `Link::to_url()` writer. The control socket parses links again before routing them. Input bounds, absolute checkout paths, duplicate-parameter rejection, and notification tag checks remain in place.

Three tests failed before the code change: the URL writer returned `sheprd://device?id=ssh%3Abox.example`, inherited schemes were still recognized, and the CLI treated `spaceport://device?id=ssh%3Abox.example` as an unknown option. After the change, `cargo +1.99.0 test --locked -p sheprd-gpui --bin sheprd link::tests::` passed 9 tests, `cli::tests::links::` passed 3, `cli::tests::development_links::cli_hands_a_spaceport_link_to_the_exact_device` passed, and `control::server::links::tests::` passed 3. The test build emitted the pre-existing macOS compact-unwind warning and `block` future-compatibility notice.

The Windows toast path now requires the registered command to equal `"<current executable>" "%1"` before it uses protocol activation. The pure guard test failed with a handler pointing to Sheprd and passed after the guard; wrong arguments and malformed commands also select the GPUI fallback. An unavailable or non-UTF-8 current executable path also fails closed. The guard test ran on macOS; Windows registry lookup and native toast delivery did not run. No installer, registry key, macOS URL handler, or other package registration was changed or exercised. The inherited release scripts still register upstream schemes and remain blocked by the release gate. A42 stays not run and G-RELEASE stays open.
