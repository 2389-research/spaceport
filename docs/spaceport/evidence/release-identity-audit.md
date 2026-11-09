# T12 release identity audit: source state

Current transition: D60 authorizes removal of the inherited custom updater;
[the removal evidence](t12-custom-updater-removal.md) records its progress and
qualification. Engine-specific paths and results below describe earlier
snapshots, not a functioning current Spaceport update route. Production
identity, platform/package update ownership, signatures and native update
qualification remain open; deleting the engine does not close those gates.

The canonical development bundle now uses `Spaceport.app` and Spaceport bundle display names; `t01-dev-bundle-name.md` records the recipe and native artifact checks. The production plist, packaging names and release identities described below remain separate open gates.

Current test-suite correction: `evidence/t04-release-contract-regressions.md` records the later 241-test release Python suite pass (eight skips) and a real development-binary CLI smoke pass. This supersedes the inherited red suite result in the GitHub removal evidence, while leaving all identity, signing and native package gates open.

T12 is blocked. This is a source audit of the current development branch, not a package or update test. No release or user installation was changed.
The [native platform results](native-platform-results.md) record the macOS
fixture failures and the unrun Windows/Linux gates.

| Surface | Current source state | Required Spaceport change |
| --- | --- | --- |
| Updater identity | `crates/sheprd-updater/src/lib.rs` has `APP_ID = "so.pen.herdr-gpui"` and uses `HERDR_RELEASE_VERSION` / `HERDR_UPDATE_PUBLIC_KEY`. Development builds without that key stay disabled. | Use an owner-controlled production ID and key; reject the upstream identity at build and install boundaries. |
| Update source | `crates/sheprd-updater/src/release.rs` points to `sheprd-labs/sheprd` releases and the asset resolver uses the upstream repository. Linux and Homebrew updater paths still name `herdr-gpui` and `sheprd`. | Use a separate Spaceport release source and package ownership after it is supplied and verified; keep updates disabled until then. |
| macOS | Development `NSProcessInfo.processName` now follows the `Spaceport` window title. `assets/macos/Info.plist` still uses `so.pen.herdr-gpui`; `package-macos.sh` assembles `Sheprd.app`, while `sign-macos.sh` checks that upstream ID. The phone helpers and Core are no longer packaged. | Give the app its own bundle ID/name/icon and signing checks. |
| Windows | The development GUI uses `spaceport.development` for notifications, taskbar and Jump List, while `scripts/release/sheprd.iss` still uses the upstream installer identity. The installer no longer registers inherited link schemes. The updater accepts `sheprd.exe` and `herdr-gpui.exe` as replacement targets. | Use an independent installer, app ID, executable and replacement target; test side-by-side safety on Windows. |
| Linux | The development window uses `spaceport.development` as its app ID. `scripts/release/package-linux.sh` still creates `Sheprd-*`, installs `org.sheprd.Sheprd.desktop`, and aliases `herdr-gpui`; its desktop entry cannot match the development window. | Use a separate desktop/package/executable identity and remove upstream aliases and excluded binaries. |
| Release workflow | `scripts/release/dispatch.sh` requires the upstream repository; `.github/workflows/promote.yml` targets the Sheprd release and tap. | Configure an owner-controlled Spaceport release route; keep publishing gated by explicit authorization. |

The production app and package IDs, signing identity, update key/channel, release repository, and final icon are not supplied or validated. G-RELEASE remains open. The source currently fails A42–A44 as a complete product claim; no native side-by-side or signed-update test has run. This inventory sets targets for later bounded changes, not permission to reuse Sheprd credentials, keys or endpoints.

The development window title, macOS process name and Windows notification display name are `Spaceport`; the macOS app bundle still says Sheprd. CLI, toast and Jump List links now use `spaceport://`; the parser rejects inherited schemes. Windows protocol toasts require a handler command that points exactly to the running executable, then fall back to GPUI delivery when none is verified. No Spaceport handler is installed by this development change, and native activation remains unverified.

## Release build gate

`sheprd-updater/src/lib.rs` now rejects any build with `HERDR_RELEASE_VERSION` set. Before the gate, a check with a fake version and fake public key compiled successfully while `APP_ID` and release URLs still named upstream. With the gate, that same check fails at compile time with `Spaceport release identity is not configured`; a normal development check passes. The updater library suite passed 140 tests with three ignored. This is an intentional release block until the independent identity and update contract are implemented and tested; it does not make the current packages safe to ship. The existing release workflow will fail rather than produce a package with Sheprd's identity.

## Source-reconciled identity matrix

An earlier pass changed the ad-hoc signed macOS development bundle ID at `f4375bbe`, using the already established `spaceport.development` placeholder. This slice also aligns the macOS development process name with the window title. The production plist, signing checks and release build gate remain as above.

| Surface | Development or retained identity | Legacy release identity still in source |
| --- | --- | --- |
| Window, process, art | `crates/sheprd-gpui/src/constants.rs` names the window `Spaceport` and gives Windows/Wayland the development-only `spaceport.development` OS ID. `app_icon.rs` now sets macOS `NSProcessInfo.processName` from that same window title. | `app_icon.rs` still embeds Sheprd artwork; `assets/icons/README.md` calls it Sheprd artwork. Final art is a G-RELEASE input. |
| macOS bundle | `justfile` now writes `CFBundleIdentifier=spaceport.development` into the ad-hoc signed `just run` and `just run-debug` bundles. `README.md` and `crates/sheprd-gpui/README.md` name this development ID. | `assets/macos/Info.plist` still sets `Sheprd`, `so.pen.herdr-gpui`, `sheprd` and `Sheprd.icns`; `scripts/release/package-macos.sh` assembles `Sheprd.app`, while `sign-macos.sh` requires the upstream ID. |
| Windows shell | `crates/sheprd-gpui/src/taskbar.rs` uses the development AppUserModelID; local CLI, toast and Jump List targets use `spaceport://`. The toast path requires a handler command for the running executable before protocol activation, and otherwise uses in-process delivery. No development handler is installed. The installer and script no longer register inherited link schemes. | `scripts/release/sheprd.iss` still uses installer GUID `{7FA70894-4FC0-4923-BDCB-B05F7475610A}`, AppUserModelID `so.pen.herdr-gpui`, publisher `Herdr UI`, and the old install directory. `package-windows.py` carries old executable aliases; `crates/sheprd-updater/src/windows.rs` accepts old replacement targets. |
| Linux desktop | `crates/sheprd-gpui/src/app.rs` applies the development Wayland/X11 ID from `constants.rs`. | `scripts/release/org.sheprd.Sheprd.desktop` declares `Name=Sheprd`, `Exec=sheprd`, `Icon=sheprd`, `StartupWMClass=org.sheprd.Sheprd`. `package-linux.sh` creates `Sheprd-*` and an `herdr-gpui` alias; `package-linux-distro.py` owns package name `herdr-gpui`; `crates/sheprd-updater/src/linux.rs` detects that package. The release desktop file does not match the development window. |
| Updater trust and artifacts | `crates/sheprd-updater/src/lib.rs` rejects `HERDR_RELEASE_VERSION` at compile time and labels ordinary builds `dev`. `release.rs` and `install.rs` contain signed-manifest and archive checks; this pass did not rerun them. | `APP_ID` remains `so.pen.herdr-gpui`; `release.rs` fetches `sheprd-labs/sheprd`, `release/repository.rs` also accepts `penso/herdr-gpui`, and `release/format.rs` accepts old asset names. `HERDR_UPDATE_PUBLIC_KEY` remains the compile-time trust input; `brew.rs`, `linux.rs` and `windows.rs` target upstream installations. |
| Release route | No Spaceport route is configured. `scripts/release/dispatch.sh` refuses any origin other than `sheprd-labs/sheprd` and any dispatcher other than `penso`. | `.github/workflows/release.yml` and `promote.yml` target that repository, signing variables and upstream Homebrew taps. The compile-time gate also blocks their release builds. These guards do not prove a Spaceport publishing path. |
| Companions | `sheprd-acp` is a potential retained host-side chat adapter, subject to T06/G-CHAT. Phone and automation helpers and bundled Core have been removed from build and package routes. Legal and protocol attribution remain required. | Old `herdr-*` chat and executable aliases remain in packages; independent release identity still needs owner-controlled inputs and native tests. |
| User-facing docs | The two development bundle ID references were updated with the recipe. Root `README.md` now describes the unqualified Spaceport development fork and links to live task evidence. | `scripts/release/README.md` still describes upstream Sheprd names, installs, URLs and updater ownership. It needs reconciliation with verified Spaceport artifacts; prose alone cannot establish parity. |

G-RELEASE remains `open`: production app/package IDs, release repository, signing accounts and keys, update channel and final icon have not been supplied or tested. Keep the release gate until these owner-controlled inputs and signed tamper/rollback checks exist. G-PLATFORM remains `open`: A42–A44 and A49–A51 have no native side-by-side, package or signed-update evidence. Other required product gates also block public release. No development placeholder is a production ID.

`SECURITY.md` still contains the upstream Sheprd advisory URL and release support policy. A development notice now labels that policy as upstream reference and says the link is not a Spaceport reporting channel. An owner-controlled private Spaceport contact and a replacement policy are release inputs; no contact was invented or upstream report route used.

The small development fix followed a red test: `python3 -m unittest scripts/release/tests/test_app_name.py -k development_bundle_uses_running_app_identity -v` failed because `justfile` wrote `so.pen.herdr-gpui.dev` while `constants.rs` said `spaceport.development`. After the recipe and docs changed, `python3 -m unittest scripts/release/tests/test_app_name.py -v` passed all five tests, including the macOS asset catalog check. These source tests do not prove LaunchServices, Keychain, signing or notification behavior in a running bundle.

Quick checks for the earlier bundle-ID slice: `just --list` parsed the recipe, `plutil -lint assets/macos/Info.plist` passed, all 27 cited source paths exist, and `python tools/spec.py check --write-report` passed package integrity. A direct search found no old `.dev` bundle ID in `justfile` or the two edited README files. Native macOS, Windows and Linux installs, signed update fixtures, production signing and release builds were not run.

## Development macOS process-name correction

The `app_icon::install` setter now reads `constants::WINDOW_TITLE` instead of a second literal `Sheprd`. The new source regression failed first on that literal, then `python3 -m unittest scripts/release/tests/test_app_name.py -v` passed all six tests after the fix. `CARGO_TARGET_DIR=/tmp/spaceport-t00-target CARGO_INCREMENTAL=0 cargo +1.99.0 check --locked -p sheprd-gpui --bin sheprd` passed on macOS, with the existing `block v0.1.6` future-compat notice. This source check and compilation do not prove a running bundle's Dock or process label. T12 stays blocked on G-RELEASE and G-PLATFORM; A42–A44 remain not_run.

## External Core packaging cut

At `13dc4573`, release and development package scripts no longer download, copy, sign, or install Herdr Core. The pinned fetch script and bundled Core license snapshot were removed; runtime discovery still selects a separately installed Core. `evidence/t12-external-core-package.md` records the bounded source and archive checks. A pre-existing installed package may still own a Core file, so migration must identify its owner before any cleanup. No installed file was changed.

The combined release suite passed 188 tests with nine skips after a smoke CLI fixture was updated for the removed local `browser open` command. Release security passed 25 tests with one skip. The release build gate still rejects `HERDR_RELEASE_VERSION`; upstream production IDs, signing and update routes remain in source. No native signed package, side-by-side install, or update ran. G-RELEASE and G-PLATFORM remain open.

## Inherited Windows link registrations

At parent `942885d2`, the app parsed only `spaceport://` links, but the Inno
Setup recipe and `site/install.ps1` still wrote `sheprd://` and
`herdr-gpui://` registry handlers pointing at their launchers. A new source
fixture failed on those rows before this cut. Both installers now leave those
schemes alone. The PowerShell install fixture keeps two unrelated throwaway
handlers in place so Windows CI can check that retained shortcut and Explorer
work does not overwrite them. The unused installer `LinkName` translation was
removed after the full release suite found it stale.

The focused absence, install-script and compatibility tests passed 12 tests
with two Windows-only skips. The release Python suite passed 187 tests with
eight skips after the translation fix. `just check-file-size`, `just
check-sorted`, and `git diff --check` passed. No Windows registry, native
installer, installed app, or updater ran here. Existing registry handlers from
older installations are not deleted by this new code; owner-safe cleanup is
still unresolved.

The installer GUID, directory, executable aliases, AppUserModelID, publisher,
release source, and updater replacement targets still name upstream owners.
Changing those requires owner-controlled production values and a side-by-side
install test. The release compile gate remains closed; T12 stays blocked and
A42–A44 remain not_run. This appended evidence needs a new signed observation
after integration.
