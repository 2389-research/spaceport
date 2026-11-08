# T12 release identity audit: source state

T12 is blocked. This is a source audit of the current development branch, not a package or update test. No release or user installation was changed.

| Surface | Current source state | Required Spaceport change |
| --- | --- | --- |
| Updater identity | `crates/sheprd-updater/src/lib.rs` has `APP_ID = "so.pen.herdr-gpui"` and uses `HERDR_RELEASE_VERSION` / `HERDR_UPDATE_PUBLIC_KEY`. Development builds without that key stay disabled. | Use an owner-controlled production ID and key; reject the upstream identity at build and install boundaries. |
| Update source | `crates/sheprd-updater/src/release.rs` points to `sheprd-labs/sheprd` releases and the asset resolver uses the upstream repository. Linux and Homebrew updater paths still name `herdr-gpui` and `sheprd`. | Use a separate Spaceport release source and package ownership after it is supplied and verified; keep updates disabled until then. |
| macOS | `assets/macos/Info.plist` uses `so.pen.herdr-gpui`; `package-macos.sh` assembles `Sheprd.app`, while `sign-macos.sh` checks that upstream ID. The package script bundles `herdr-core`, mobile, automation and nearby helpers. | Give the app its own bundle ID/name/icon and signing checks; package only retained companions and never Core. |
| Windows | `scripts/release/sheprd.iss` uses upstream `AppUserModelID` and Sheprd installer identity. The updater accepts `sheprd.exe` and `herdr-gpui.exe` as replacement targets. | Use an independent installer, app ID, executable and replacement target; test side-by-side safety on Windows. |
| Linux | `scripts/release/package-linux.sh` creates `Sheprd-*`, installs `org.sheprd.Sheprd.desktop`, and aliases `herdr-gpui`. `crates/sheprd-gpui/src/constants.rs` has the same upstream desktop ID. | Use a separate desktop/package/executable identity and remove upstream aliases and excluded binaries. |
| Release workflow | `scripts/release/dispatch.sh` requires the upstream repository; `.github/workflows/promote.yml` targets the Sheprd release and tap. | Configure an owner-controlled Spaceport release route; keep publishing gated by explicit authorization. |

The production app and package IDs, signing identity, update key/channel, release repository, and final icon are not supplied or validated. G-RELEASE remains open. The source currently fails A42–A44 as a complete product claim; no native side-by-side or signed-update test has run. This inventory sets targets for later bounded changes, not permission to reuse Sheprd credentials, keys or endpoints.

## Release build gate

`sheprd-updater/src/lib.rs` now rejects any build with `HERDR_RELEASE_VERSION` set. Before the gate, a check with a fake version and fake public key compiled successfully while `APP_ID` and release URLs still named upstream. With the gate, that same check fails at compile time with `Spaceport release identity is not configured`; a normal development check passes. The updater library suite passed 140 tests with three ignored. This is an intentional release block until the independent identity and update contract are implemented and tested; it does not make the current packages safe to ship. The existing release workflow will fail rather than produce a package with Sheprd's identity.
