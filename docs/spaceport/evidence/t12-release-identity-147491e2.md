# T12 release identity snapshot at `147491e2`

This source audit updates the earlier [release identity audit](release-identity-audit.md)
after the development command rename. It inspected repository files only. No
package was built, signed, installed, updated or published; no account or
personal app state was read. T12, G-RELEASE and G-PLATFORM remain blocked, and
A42–A44 are `not_run`.

| Surface | Source state at this revision | Release consequence |
| --- | --- | --- |
| Development app | `crates/sheprd-gpui/Cargo.toml` builds one `spaceport` GUI/CLI binary. `crates/sheprd-launcher/Cargo.toml` builds `spaceport-launcher`; CLI help, the Spaceport-owned managed skill, Windows relaunch/login paths and package binary inputs use them. The window uses `Spaceport` and `spaceport.development`. | This is an isolated development name, not a production bundle or publisher identity. |
| State and local control | `sheprd-client/src/local_path.rs` chooses the `spaceport` state root; the Unix socket is `control.sock` beneath it, Windows uses `spaceport-control-<SID>`, and links parse `spaceport://`. | Existing exact-window refusal stays; no upstream state migration or personal skill rewrite was done. |
| macOS package | `package-macos.sh` places `spaceport` in `Sheprd.app`; `assets/macos/Info.plist` still says `so.pen.herdr-gpui`; `sign-macos.sh` enforces that upstream ID. | Bundle name, ID, signing and side-by-side ownership are not Spaceport release ready. |
| Windows package | The zip and Inno recipe use `spaceport.exe` and `spaceport-launcher.exe`, but the archive/installer name remains `Sheprd`, and `sheprd.iss` retains upstream AppUserModelID and installer ownership. | Native shell and install ownership have not been qualified. |
| Linux package | `package-linux.sh` places `bin/spaceport` in a `Sheprd-*` archive with `org.sheprd.Sheprd.desktop` and `herdr-gpui` license location. | The desktop entry and package IDs do not match the development app ID. |
| Updater and publication | `sheprd-updater` still uses `so.pen.herdr-gpui`, the `sheprd-labs/sheprd` release URLs and upstream update-key inputs. Its compile-time `HERDR_RELEASE_VERSION` gate rejects a Spaceport release build. `dispatch.sh` still requires the upstream repository. | Keep the gate closed until owner-controlled app IDs, source, key, signing account and update tests exist. |
| Inherited website | Both Netlify files ignore Git-triggered builds. `site/install.sh`, `site/install.ps1`, and the inherited website still point to upstream downloads; the PowerShell script expects `sheprd.exe`, while the new Windows archive has `spaceport.exe`. The inherited site still advertises phone/TestFlight. | These are not Spaceport installers. The source guard does not prove a hosting account was disconnected or prevent manual deployment. |

The upstream attribution and separate `sheprd-acp` chat companion remain in
source. The command rename did not grant the fork ownership of the Sheprd
installer, update channel, website, signing keys or installed user files.
Before any release, supply and verify production IDs, publisher and release
repository, signing/update keys and channel, independent icon and package
names, owner-safe side-by-side install/update behavior, and macOS/Windows/Linux
native results. Do not infer those values from `spaceport.development` or the
new command filename. This snapshot is a source observation, not proof that
the desktop app works end to end.
