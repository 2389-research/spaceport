# d4z8 — Windows and Linux package identities and signing scaffolding

Base `6a678a90f5` on `main`. The C2 update-channel decision was answered on
2026-11-26: macOS production updates stay with Sparkle (D65), and Windows and
Linux updates are a separate later decision. This ticket therefore builds no
updater and no adoption of one; it gives the Windows and Linux packages their
own identity and records what signing still needs.

Survey first, because the brief's Linux half named artifacts the repository
does not have: there is no `.deb`, `.rpm` or AppImage route, and
`package-linux.sh` / `package-linux-distro.py` were deleted with the custom
updater. Linux ships as a Cargo binary plus the Nix flake (`flake.nix`), which
installs the desktop entry and icon itself. So the Linux half of this ticket is
the desktop-entry identity, its install paths and the version scheme, not a new
distribution package. Windows matches the brief: `package-windows.py` writes the
zip, `windows-installer.py` compiles and smokes `spaceport.iss`, and
`windows-signing.py` / `windows-sign-file.ps1` hold the signing path.

## Identity decided

| Item | Value | Why |
| --- | --- | --- |
| Windows product name | `Spaceport` | The app already calls itself Spaceport everywhere else; `windows_resource.rs`'s `APP_NAME` feeds `FileDescription` and `ProductName`, which Explorer and Task Manager show. |
| Windows publisher | `2389 Research, Inc.` | Same string the About box prints; D66 moved the repository to 2389 Research. Used for `AppPublisher` and the executable's `CompanyName`. |
| Windows AppId | `{62B8722B-1FA2-48E0-9609-46AA5D431983}` (new) | A fork is a separate product. Reusing the inherited Sheprd AppId would make Setup replace another team's installation and PATH entry on upgrade; a new AppId installs beside it and finds only its own versions. The GUID is permanent from here. |
| Windows directory | `%LOCALAPPDATA%\Programs\Spaceport` | Follows the AppId split. An inherited Sheprd installation keeps its directory and PATH entry. |
| Windows startup value | `Spaceport` | `launch_at_login.rs` already writes a `Run` value named `Spaceport`, while the installer removed one named `Herdr UI`, so Launch at login's entry outlived an uninstall. One name now. |
| Windows AppUserModelID | the app ID the shipped executable embeds (`spaceport.development` today) | The app registers `Software\Classes\AppUserModelId\{APP_ID}` for toasts and groups its taskbar button by it; a shortcut naming anything else posts its notifications apart. `smoke` now reads the build record out of `spaceport.exe` and requires the installer's define to equal it. |
| Windows artifact names | `Spaceport-VERSION-TARGET.zip`, `Spaceport-VERSION-TARGET-setup.exe` | The B4 finding's coordinated cut; CI's `windows-package` job builds and smokes the same paths. |
| Windows installer text | `Add spaceport to PATH, for terminals and agents` | The artifact installs `spaceport.exe`, so the PATH task names that command. |
| Windows version scheme | `YYYYMMDD.COUNTER`, unchanged | Already enforced by both packaging scripts and `release_version::numeric`. |
| Linux desktop file | `scripts/release/spaceport.desktop`, installed as `spaceport.development.desktop` | The source file name was the inherited `org.sheprd.Sheprd`, an app ID the app never reports. The installed name follows the build's app ID, which `StartupWMClass` also names, and `constants/tests.rs` pins that. |
| Linux package and icon paths | package name `spaceport`, `share/icons/hicolor/scalable/apps/spaceport.svg`, `Categories=Development;` | The icon is a hicolor theme name, so `Icon=spaceport` matches the installed file. |

Decisions left open, deliberately:

- **A released Windows identity.** Packages built today carry the development
  app ID because `preview_identity::select` gives every non-macOS build the
  development identity and `constants::APP_ID` still refuses a release build
  (`HERDR_RELEASE_VERSION` must be unset). Shipping a production Windows
  identity, and the update route it would carry, is the Windows/Linux half C2
  did not answer.
- **A Linux distribution package.** `.deb`, `.rpm` or AppImage naming,
  dependencies and file layout need a package to exist first; nothing in this
  ticket invents one.
- **Windows signing.** D61 defers it. The path is wired and tested but unused.

## What stays inherited

`sheprd-acp` keeps its name (its fate is D4/t6vb) and is still installed
beside `spaceport.exe`. `site/install.ps1` and `site/install.sh` remain the
inherited Sheprd installers for a different product, so the `.iss` no longer
claims either replaces the other. macOS preview identities, the retained
`Sheprd.icns` / icon-catalog names, and every Herdr runtime identity are
untouched.

## Signing scaffolding and gates

Signing was already scaffolded; this ticket records the gates in
`scripts/release/README.md` (Signing Gates) instead of adding machinery.

| Platform | Mechanism | Credential it still needs | State |
| --- | --- | --- | --- |
| macOS | Developer ID + notarization + stapling | Developer ID certificate for Team `HD9NM9NSMK`, `notarytool` credentials, Sparkle Ed25519 key, in the protected `macos-preview` environment | Configured for previews (D61/D64/D65) |
| Windows | Authenticode with an RFC 3161 timestamp | Azure Artifact Signing endpoint, account and certificate profile plus an Azure CLI OIDC login, through `ARTIFACT_SIGNING_ENDPOINT` / `ARTIFACT_SIGNING_ACCOUNT` / `ARTIFACT_SIGNING_PROFILE` | Wired, nothing calls it; a protected signing job must set up that login |
| Linux | Detached signature over the archive and its checksum list | A signing key and an archive to sign | Not started |

No certificate, key or secret is in the working tree, and
`test-release-security.py` keeps signing credentials out of every workflow.

## Validation

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | passed |
| `cargo clippy -p spaceport-gpui --all-targets -- -D warnings` | passed (existing `block 0.1.6` future-incompatibility notice) |
| `cargo clippy -p spaceport-launcher --all-targets -- -D warnings` | passed |
| `cargo test -p spaceport-gpui --bin spaceport -- constants` | passed, 4 tests, including the desktop-entry one |
| `cargo test -p spaceport-launcher` | passed, 3 tests; `windows_exe.rs` is `cfg(windows)` and ran nothing here |
| `just test-release-scripts` | 418 tests; two failures that predate this change and concern commit subjects already in history (`test_commit_messages`, `test_changelog`) |
| `just check-i18n` | passed, 2,124 messages in 114 files |
| `just check-sorted` | passed |
| `just check-file-size` | passed |
| `just check-spec-tools` | passed |

Not run, and not claimed: the Windows installer build and smoke (needs a
Windows runner; `windows-package` CI covers it, and the new
AppUserModelID-versus-build-record check runs there), Inno Setup compilation,
`desktop-file-validate`, and any native Windows or Linux UI. Headless source,
package-tool and unit checks are what this change was verified with.
