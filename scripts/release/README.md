# Release Packaging

**Spaceport development status:** These are inherited Sheprd packaging notes,
not instructions to release this fork. The current release build rejects
`HERDR_RELEASE_VERSION` because general production identity, update delivery,
and native package checks remain unresolved. The separate macOS preview
route below uses the approved 2389 Developer ID inputs and `ai.2389.spaceport`;
that approval does not enable general production releases. See the
[release identity audit](../../docs/spaceport/evidence/release-identity-audit.md)
and [open gates](../../docs/spaceport/plan/gaps.json). Do not run an inherited
publish or install route. D64 approves the separate tagged prerelease route below.
The inherited custom updater, its feeds and update archives have been removed.
D65 selects Sparkle for installed macOS previews, with a separate Spaceport
signing key and preview feed. Other platform/package update routes and the
complete native qualification remain release gates.

Run with Bash; packaging scripts never build or execute the GUI, bundle a daemon,
or publish artifacts. Output directories must already exist. Existing
artifacts are refused. Versions are calendar versions `YYYYMMDD.COUNTER`: an
eight-digit UTC date and a same-day counter starting at 1, without `v`,
prerelease/build suffixes, or leading zeros.

## Spaceport macOS previews

The separate `macos-preview.yml` workflow builds and signs **macOS arm64 previews only**.
Pushing `vYYYYMMDD.COUNTER` as `harperreed` to `2389-research/spaceport` builds an
optimized app, signs and notarizes it, and publishes a GitHub **prerelease** with
the DMG and `SHA256SUMS`. Manual runs on main accept an exact commit SHA and an
explicit version; they upload workflow artifacts without publishing a release.
The [20261110.5 prerelease](https://github.com/2389-research/spaceport/releases/tag/v20261110.5)
completed this route; its [qualification record](../../docs/spaceport/evidence/t12-macos-preview-20261110.5-signed.md)
includes verification of the actual published download.
Both routes require the selected commit to belong to main and have a complete,
successful main CI run. D66 temporarily pauses Spaceport Windows CI until
Kata `h8g5` fixes native performance; macOS/Linux checks remain mandatory,
and the aggregate gate requires Windows jobs to be skipped. No Windows
performance budget or acceptance result changes. The signing and publishing jobs re-fetch the tag and
refuse it if it moved. Only the publishing job receives `contents: write`.

The unsigned build has no signing secrets and runs `smoke-cli.py` on its actual
executable. The signing job uses the protected `macos-preview` environment and
only that run's checksummed artifact. It verifies signatures, notarization tickets
and Gatekeeper, mounts the final DMG read-only, checks the app identity, and runs
the mounted CLI before writing the checksum. This does not prove native GUI behavior.

For a fresh repository, merge these workflows to main and configure the
`macos-preview` environment in repository Settings > Environments. Allow main
branches and `v*` tags through its selected deployment rules. Restrict who can
create or update those tags with repository access rules. The activated repository
now belongs to `2389-research`; organization members inherit its repository
permissions. The move retains the existing signing environment and its main/tag
rules. Workflow eligibility still requires `harperreed` as both initiating and
triggering actor. Review environment protection before expanding signing authority. Add these
**environment** values, never repository source files:

| Kind | Name | Value |
| --- | --- | --- |
| Variable | `MACOS_SIGNING_IDENTITY` | `Developer ID Application: 2389 Research, Inc (HD9NM9NSMK)` |
| Secret | `MACOS_CERTIFICATE_P12_BASE64` | Base64 of the matching Developer ID certificate and private key P12 |
| Secret | `MACOS_CERTIFICATE_PASSWORD` | That P12's export password |
| Secret | `APPLE_API_PRIVATE_KEY` | App Store Connect private `.p8` key contents |
| Secret | `APPLE_API_KEY_ID` | That API key's ID |
| Secret | `APPLE_API_ISSUER_ID` | That API key's issuer ID |
| Secret | `SPARKLE_PRIVATE_KEY` | Separate Spaceport preview Ed25519 seed exported by Sparkle's key tool; used only to sign updates/feed |

Doctor Biz authorized hosted setup, and all six environment secrets and the
identity variable are configured for `2389-research/spaceport`. An installed local
identity is insufficient for hosted signing: CI needs the matching P12 and its
password. Do not use a different company's certificate. The importer
requires exactly one signing identity and checks Team `HD9NM9NSMK`.

After main CI passes on the commit to release, select an unused version and push
one tag (replace the example version and SHA):

```sh
git tag -a v20261110.6 FULL_MAIN_COMMIT_SHA -m 'Spaceport macOS preview 20261110.6'
git push origin refs/tags/v20261110.6
```

A read-only history check rejects reruns and any earlier run using the exact
version, even if that run failed before signing. It requires complete, unchanged
history within 1,000 runs and refuses API failure, truncation or unknown titles.
Malformed version tags cannot sign and do not reserve a valid version. The
serialized workflow checks again before signing. Inspect retained notarization
receipts after a failure; never rerun, move or delete a used tag to force another
submission. Use a new version after resolving the failure. D65 adds the preview
updater feed; Homebrew and general production publication remain excluded.

### Preview updates

Updater-enabled installed previews expose **Check for Updates…** in the app
menu and `spaceport run check-for-updates`. Sparkle asks permission for automatic
checks; installation requires confirmation by default. Ordinary development and
compiled worktree builds cannot start the updater. The first enabled preview
requires one manual installation; older previews cannot bootstrap themselves.
Updates replace Spaceport and its bundled chat adapter, never either Herdr runtime.

The build embeds pinned Sparkle 2.10.1 and its license, preserving framework
symlinks. Signing rechecks all original framework bytes before credentials,
then signs nested code inside out. The final app and DMG retain the existing
Developer ID/notarization/Gatekeeper checks. `sparkle.json` is the shared public
SDK version/checksum, feed URL and Ed25519 trust configuration.

Only the protected signing job receives `SPARKLE_PRIVATE_KEY`, over the SDK
signer's stdin. It signs the final stapled DMG and appcast. Publication verifies
those signatures with the public key, publishes the immutable dated prerelease,
downloads and verifies that public artifact, then replaces `appcast.xml` on the
fixed `macos-preview-feed` release. The marker tag stays fixed. Signed previous
feed versions must increase; an existing marker missing its asset fails closed.
GitHub replaces assets through delete/upload, so checks can briefly fail while
the feed changes. Keep the prior signed feed in retained workflow evidence.

The feed uses the explicit release download URL, since GitHub's `latest` route
excludes prereleases. The client requires signed feeds with exact numeric
`SUSignedFeedFailureExpirationInterval=0` and verifies archives before extraction.
Zero disables Sparkle's default recovery after 20 days of feed-signature failures.
Retain the signing key backup; rotation needs a planned signed migration.
Sparkle deliberately allows trusted publisher key and app
identity rotation; exact Spaceport app ID/Team checks belong to this release
pipeline. Wrong-key ZIP and feed tests exercise signature refusal without the
same-Team signed-DMG key-rotation fallback. These policies do not authorize an
inherited upstream feed, production release or daemon installation.

Preview builds use `SPACEPORT_MACOS_PREVIEW_VERSION`, `ai.2389.spaceport`, and
`Spaceport.app`. The compile-time selector rejects other target operating systems
and malformed versions. The app ID files beside `crates/sheprd-gpui/build.rs` are
the source for compiled and bundle identities; a final newline is allowed.
Ordinary `just bundle` builds retain `spaceport.development`. The general
`HERDR_RELEASE_VERSION` refusal and the app constants’ `RELEASE_BUILD` policy
remain in force. Preview signing grants no provider credential reads or automatic skill install.
The existing three-line `--build-info` output remains unchanged.

## Build Identity

Packaging and `just bundle` require Python 3 and read a versioned identity record
embedded in the supplied executable, never the packaging checkout's Git state.
On Linux, linked-worktree binaries select `assets/icons/sheprd-linux-worktree.svg`;
other builds select `assets/icons/sheprd-linux.svg`. Both circular variants are
installed as `share/icons/hicolor/scalable/apps/sheprd.svg`, so manual archives
and distribution packages use the same layout without a pixmaps fallback.
Regenerate both SVGs with `just icons-linux`; see the
[icon guide](../../assets/icons/README.md#linux-desktops) for the composition and
desktop lookup behavior. On macOS, linked-worktree binaries select
`assets/icons/Sheprd-worktree.icns` and `Sheprd-worktree.car`; other builds use
`Sheprd.icns` and `Sheprd.car`. The installed icons keep their standard filenames.
No binary is executed, so foreign
Linux architectures and both macOS slices work on the packaging host. macOS
inputs must have identical identities (branch and PR included). Missing, malformed,
or conflicting records fail closed; older binaries must be rebuilt.

`spaceport --build-info` prints three newline-terminated `key=value` lines:
`worktree=0|1`, `branch=...`, and `pr=...`, before any GUI or daemon startup.
The same record is retained in optimized binaries for packaging. This metadata is
an identity hint, not a signature or proof of provenance; supply trusted binaries.

At build time, Git is anchored at the crate manifest. Only differing canonical
Git/common directories mark a linked worktree, not branch names, `--dev`, or an
ordinary checkout with a separate Git directory. Stable builds have an empty
branch; linked builds use the branch or detached short SHA. Printable Unicode,
including trailing Unicode whitespace, is preserved; branch names containing
control characters use the short SHA instead. Missing Git/source archives degrade
to stable. Cargo watches existing HEAD, current ref, packed refs, and checkout
pointer files, not the entire ordinary `.git` directory. When a loose ref is
missing, its nearest existing refs directory is watched for creation. Source
archives do not automatically detect later Git initialization.

`HERDR_BUILD_PR_NUMBER=123 cargo build ...` overrides PR lookup; an explicitly
empty value disables it. Values must be empty or positive ASCII decimal digits.
Otherwise attached linked branches get a best-effort
`gh pr list --head BRANCH --state open --limit 1 --json number --jq '.[0].number'`
lookup in the manifest repository, with prompts disabled and a two-second timeout.
Only open PRs are considered; numeric and `#numeric` branches are treated as branch
names, never PR numbers. Missing gh/auth/network/open PR never prevents a build.
There is no runtime lookup.
Cargo does not poll remote PR changes: force a rebuild or change the override
when PR metadata changes without a local HEAD/ref change. Set an explicit override
for reproducible builds and matching cross-architecture PR metadata.

## Packaging Commands

The release workflow's `ci-gate` job runs `scripts/release/check-ci.py` against
the exact release SHA, waiting for a CI run that is still going. A successful
CI workflow alone is insufficient: every platform check (both its
default-features and all-features legs), native Windows test job, and Linux
optimized build must have completed successfully in the same owner-triggered
push or manual run on `main`. Skipped or missing jobs fail the gate. CI runs
the macOS optimized builds on pull requests only, where the final gate requires
them: on `main` they would hold the account-wide macOS runners this workflow's
own macOS builds wait for. `just release` dispatches without waiting, so the
release builds overlap CI; `sign`, `windows-sign`, and everything after them
wait behind `ci-gate`, so nothing is signed or published before CI passes. CI
never cancels a run on `main`, so a later merge cannot strand a release commit
without one. For documentation-only commits, start a full run with
`gh workflow run ci.yml --repo sheprd-labs/sheprd --ref main` before releasing.

```sh
cargo install cargo-about --version 0.9.2 --locked --features cli
python3 scripts/release/generate-notices.py OUTPUT_DIR/THIRD-PARTY-NOTICES.txt
bash scripts/release/package-macos.sh VERSION ARM64_BINARY X86_64_BINARY OUTPUT_DIR OUTPUT_DIR/THIRD-PARTY-NOTICES.txt
bash scripts/release/sign-macos.sh VERSION OUTPUT_DIR/Spaceport.app OUTPUT_DIR universal import
bash scripts/release/package-linux.sh VERSION TARGET BINARY OUTPUT_DIR OUTPUT_DIR/THIRD-PARTY-NOTICES.txt
bash scripts/release/render-cask.sh VERSION SHA256 [TOKEN] > /path/to/rendered/TOKEN.rb
```

macOS assembly requires Apple command-line tools. It produces unsigned
`Spaceport.app`, containing the arm64 or universal GUI, plist, icon, root `LICENSE` and `NOTICE`,
protocol license/attribution, Octicons license, and third-party notices.
Both plist versions are set to the supplied version.
The package floor is macOS 14.2 (`LSMinimumSystemVersion`; Homebrew can only
express `:sonoma`). The binary strongly links CoreAudio's process-tap functions,
introduced in 14.2, through rodio's cpal backend, so dyld refuses to launch it
on 14.0 and 14.1. GPUI checks for macOS 15 at runtime before using its newer
window behavior. The `macos-14` CI job that ran the workspace tests on that
floor is commented out in `ci.yml` to save macOS runners; re-enable it if a
macOS 14 regression appears. Release builds run on `macos-15` runners. Build release inputs with
`MACOSX_DEPLOYMENT_TARGET=14.2` to match the plist; do not supply binaries
targeting a newer OS.

Signing requires macOS, Apple tools, `openssl`, `jq`, network access to Apple's
notary service, and these environment variables:

- `MACOS_CERTIFICATE_P12_BASE64`: base64-encoded Developer ID Application P12.
- `MACOS_CERTIFICATE_PASSWORD`: nonempty P12 password.
- `MACOS_SIGNING_IDENTITY`: `Developer ID Application: 2389 Research, Inc (HD9NM9NSMK)`
  or the installed certificate SHA1 `FF599C9C778C879ECB7DC8AEF258890A1268F7E6`.
- `APPLE_API_PRIVATE_KEY`: literal multiline App Store Connect API `.p8` contents,
  not a filename or base64 value.
- `APPLE_API_KEY_ID`: API key ID.
- `APPLE_API_ISSUER_ID`: team API issuer UUID.

Test P12 import with Apple's `security` tool before using hosted signing;
an OpenSSL roundtrip alone does not establish Apple importer compatibility.
The local `20261109.1` preview imported the supplied matching P12 into a
temporary keychain, signed successfully and restored the keychain search list;
the login keychain stayed locked. This does not qualify a hosted run. See the
[artifact evidence](../../docs/spaceport/evidence/t12-macos-signed-preview.md).
Local installed-identity mode reuses the existing certificate and does not
export or import a P12; it requires access to that keychain's signing key.

Use a trusted disposable signing runner and trusted binaries. Do not use shell
tracing around secret setup. The script uses a private temporary directory and
explicit temporary keychain and never changes the default keychain. It snapshots
the user search list, restores it after keychain creation (which may alter it),
and restores it again during cleanup. It traps exit/INT/TERM to delete credentials
and the keychain. Do not run concurrent keychain-changing jobs on that user account.
As with any trap,
SIGKILL/power loss cannot be cleaned up. Apple security tools receive passwords
as arguments, so the runner must not have untrusted local users. No GUI executable
is run, including for version probing. Signing works on a copy of the input app.

The bundle holds `Contents/MacOS/spaceport` and the `sheprd-acp` chat runner.
Packaging takes the chat helper from beside each architecture's `spaceport`.
Herdr Core is separately installed. The helper is signed first without the
app's entitlements, then `spaceport`, then the bundle, all with hardened runtime and secure
timestamps. The zipped app must receive JSON status `Accepted`, then the app is
stapled and validated. The DMG contains `Spaceport.app` and an `/Applications` symlink;
it is signed, separately notarized, stapled, and checked with codesign and
Gatekeeper before publishing the local output filename:
`Spaceport-VERSION-ARCHITECTURE-apple-darwin.dmg`. Any failed operation aborts.
Existing DMG outputs are refused before loading credentials.

Linux targets are `x86_64-unknown-linux-gnu` or `aarch64-unknown-linux-gnu`.
The release workflow builds both natively on Ubuntu 24.04 architecture runners,
using the same `scripts/install-linux-deps.sh` library setup as CI. Both archives
are mandatory in the single immutable asset manifest; signing, provenance and
consumer verification cover both. The SBOM unions both Linux, both macOS, and
the Windows target graphs, each the app's joined with those of the programs shipped beside it (`SHIPPED` in
`generate-sbom.py`). No separate per-platform manifest or publication job is used.
The caller must supply the matching release binary; packaging does not cross-build
or resolve shared libraries. Output is `Sheprd-VERSION-TARGET.tar.gz`, with a
same-named root containing `bin/spaceport` and `bin/sheprd-acp`
(the companion taken from beside the binary), a scalable SVG icon, desktop entry, license and
notice under `share/`.
Install that tree into a chosen prefix with its `bin` on
PATH. Archives are not promised to be bit-for-bit reproducible.

`scripts/release/package-linux-distro.py VERSION TARGET ARCHIVE OUTPUT_DIR
--nfpm NFPM` then repackages that manual archive as `Sheprd-VERSION-TARGET.deb`,
`.rpm` and `.pkg.tar.zst`. It extracts only the expected regular files, installs
them under `/usr`, and builds all three formats before publishing any, so every
Linux format ships the tarball's exact bytes. nfpm is the only packaging tool;
`scripts/release/install-nfpm.sh` downloads its pinned release and checks a
SHA-256 copied from that release's Sigstore-verified `checksums.txt`. Package
dependencies are declared explicitly, not inferred: the binary requires
GLIBC_2.39 (it is built on Ubuntu 24.04), links ALSA, FreeType, Fontconfig, xcb and
xkbcommon, and dlopens the Vulkan loader and libwayland-client. RPM
requirements are sonames so they resolve on any RPM distribution. Entries are
stamped with the release date, and the release host name is kept out of the
RPM header, so repackaging one release produces identical bytes.
The same completed packages are also published as `Herdr-VERSION-TARGET.deb`,
`.rpm` and `.pkg.tar.zst` to preserve the inherited download names. The
`Sheprd-*` packages retain their Sigstore sidecars and provenance; the bare
legacy aliases must match them byte-for-byte at publication and verification
and appear in both checksum manifests.
`scripts/release/smoke-linux-packages.sh VERSION TARGET DIST_DIR` then installs
each package into digest-pinned Ubuntu 24.04, Debian 13 and Fedora 42 containers,
plus Arch Linux on x86_64 (no official ARM64 image exists), runs `spaceport --help`
and the shipped programs' `--help` or `--version`,
fails if `ldd` reports a missing library or a dlopened loader is absent, and removes
the package again. When packaged dependencies change, update `DEPENDS` in
`package-linux-distro.py` and rerun the smoke script.

The Windows targets are `x86_64-pc-windows-msvc` and
`aarch64-pc-windows-msvc`, built natively on `windows-2025` and
`windows-11-arm` after clippy, the protocol/client suites and the CLI tests pass
there. `scripts/release/package-windows.py VERSION TARGET BINARY LAUNCHER
OUTPUT_DIR THIRD_PARTY_NOTICES` (standard-library Python, since the runner has no
zip tool) writes one `Sheprd-VERSION-TARGET.zip` per target with a same-named root
containing `spaceport.exe`, the windowed `spaceport-launcher.exe` that the
installer's Start menu shortcut opens, `sheprd-acp.exe`, which the app starts
chats with (taken from beside `BINARY`), real legacy executable
and helper copies, and `licenses/` holding the same license
and notice files as the Linux tree. Entries carry a fixed timestamp.

`scripts/release/windows-installer.py` then installs the pinned Inno Setup
compiler (`install-iscc`, checked against its SHA-256), compiles
`sheprd.iss` over the extracted zip tree into
`Sheprd-VERSION-TARGET-setup.exe` (`build`), and installs and uninstalls it on
the runner (`smoke`), checking the files, their subsystems, `spaceport.exe`'s
output, the Start menu shortcut's target (the launcher) and AppUserModelID,
the user `PATH` and Settings entries, and a silent reinstall that preserves
startup settings without launching the app.
CI's `windows-package` job runs the same
optimized build, packaging, installer build, and install/uninstall check on
`windows-2025` for any pull request that changes their inputs
(`scripts/ci-changes.sh` lists them), and the required gate then requires it. The installer is per-user (no elevation),
installs to `%LOCALAPPDATA%\Programs\Herdr UI` as `site/install.ps1` does, adds
that directory to the user `PATH`, and creates the Start menu shortcut to the
launcher.
The smoke test seeds a disabled `Herdr UI` startup entry targeting the legacy
launcher and requires the command and `StartupApproved` bytes to survive the
upgrade unchanged. Setup renames only shortcuts targeting this installation;
it preserves a preexisting desktop shortcut even without a new desktop task.
Both zips and both
installers are mandatory in the asset manifest and receive checksums, Sigstore
sidecars and provenance like every other asset, after the Authenticode signing
below when it is set up.

## Windows Signing

The `windows` job's zip and installer are unsigned, and so are every local and
pull request build: `windows-installer.py build` signs only when given `--sign`,
and `sheprd.iss` only sets `SignTool` when ISCC receives one. The release
workflow then signs in `windows-sign`, one job per target on `windows-2025`,
gated by the protected **`release`** environment like `sign`.

Signing is optional until it is set up: `validate` turns it on once the
`WINDOWS_SIGNING_SUBJECT` repository variable (below) is set, and warns while it
is not. Until then `windows-sign`, `windows-smoke` and `winget` are skipped,
`sign` and `attest` take the unsigned `windows-package-TARGET` artifacts, and
the release ships unsigned Windows zips and installers, and Windows
SmartScreen warns on first run. Once the variable is
set, `windows-sign` requires the Azure setup below, and a release without it
fails instead of falling back to unsigned. When signing is on:

1. `windows-signing.py unpack` extracts the tested zip without running anything.
2. `azure/login` signs in to Azure with the job's OIDC token, and
   `azure/artifact-signing-action` signs `spaceport.exe`,
   `spaceport-launcher.exe` and `sheprd-acp.exe`
   with SHA-256 and an RFC 3161 timestamp from
   `http://timestamp.acs.microsoft.com`.
3. `windows-signing.py repack` refuses unless the tree holds exactly the zip's
   files and each executable differs from the tested one only by its appended
   or replaced trailing signature (checksum, certificate table entry, alignment
   padding). The script then writes
   the zip again with the same entry order and timestamps.
4. `windows-installer.py build --sign` compiles the installer from the signed
   zip. ISCC calls `windows-sign-file.ps1` as its `SignTool`, with
   `SignedUninstaller=yes`, to sign the uninstaller it embeds and the final
   `-setup.exe`, through the ArtifactSigning module and Azure CLI login of
   step 2 and the same digest and timestamp.
5. `windows-signing.py verify` requires Windows to report every signature as
   valid and timestamped, from exactly `WINDOWS_SIGNING_SUBJECT`.

`windows-smoke` then installs and uninstalls each signed installer on its own
architecture, outside the signing environment, and checks the signatures of
the installer and of the installed app, launcher and uninstaller. `attest`
then takes only the signed `windows-signed-TARGET` artifacts, so checksums,
Sigstore and GitHub provenance (`actions/attest-build-provenance`) cover the
bytes users download.

Signing needs this one-time Azure setup, which only the maintainer can do:

1. An Azure Artifact Signing (formerly Trusted Signing) account with a
   completed identity validation and a public-trust certificate profile.
2. A Microsoft Entra app registration with a federated credential for issuer
   `https://token.actions.githubusercontent.com` and subject
   `repo:sheprd-labs/sheprd:environment:release`, holding the **Artifact Signing
   Certificate Profile Signer** role on the certificate profile and nothing
   else. No client secret: only an OIDC token from a job in the protected
   `release` environment can use it.
3. These variables on the `release` environment (not the repository), none
   secret, except `WINDOWS_SIGNING_SUBJECT`, below:

   | Variable | Value |
   | --- | --- |
   | `AZURE_CLIENT_ID` | the app registration's client ID |
   | `AZURE_TENANT_ID` | its tenant ID |
   | `AZURE_SUBSCRIPTION_ID` | the subscription holding the signing account |
   | `ARTIFACT_SIGNING_ENDPOINT` | the account's regional endpoint, such as `https://eus.codesigning.azure.net/` |
   | `ARTIFACT_SIGNING_ACCOUNT` | the signing account name |
   | `ARTIFACT_SIGNING_PROFILE` | the certificate profile name |

4. `WINDOWS_SIGNING_SUBJECT` as a **repository** variable: the certificate
   subject Windows reports, such as `CN=..., O=..., L=..., S=..., C=...`. The
   `windows-sign` job verifies against this value. The subject is the validated
   identity, so read it from a first signed
   file (`(Get-AuthenticodeSignature file.exe).SignerCertificate.Subject`).
   Set it only once the Azure variables above are in place: it is the switch
   that turns signing on. Artifact Signing certificates last
   days, so pin the subject, never a thumbprint.

## Winget

After a stable release publishes, the `winget` job (and the same job in
`promote.yml` when a beta is promoted) submits it to
[microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) as
`Penso.HerdrUI`. It downloads the published `SHA256SUMS`, installs the pinned
[Komac](https://github.com/russellbanks/Komac) after checking its SHA-256, and
runs `winget.py submit`: Komac writes the new version's manifests from the
previous winget-pkgs version and the `-setup.exe` release URLs (x64, plus
arm64 whenever the release lists it), and the script refuses to submit unless
every installer is a per-user (`Scope: user`) Inno Setup installer with the
release's URL and SHA-256, the publisher is `Herdr UI`, and the product code is
the installer's `AppId`. Only then does `komac submit` open the pull request.
Betas never reach winget.

`Penso.HerdrUI` and the publisher must never change: winget matches installed
copies and upgrades by them. `winget.py` holds both, and its tests tie them to
`sheprd.iss`.

The maintainer has to:

1. Create the protected **`winget`** environment, with required reviewers and
   deployments limited to `main`, like `homebrew`.
2. Fork `microsoft/winget-pkgs` to the account that will submit, and store a
   classic personal access token for that account with only the `public_repo`
   scope as the `winget` environment secret `WINGET_TOKEN`. Komac cannot open
   pull requests with a fine-grained token.
3. Submit the first version by hand, after the first signed release: Komac's
   update needs an existing version to start from. Run
   `komac new Penso.HerdrUI --version VERSION --urls URL_X64 URL_ARM64` with
   the two published `-setup.exe` URLs, check that it detects `inno` and
   `user` scope, retain publisher `Herdr UI`, set package name to `Sheprd`, the license
   and URLs, and submit. Until that version is merged, the job only prints a
   warning.

Without `WINGET_TOKEN` the job also prints a warning and submits nothing, so a
release does not fail while winget is not set up. Once winget is live, that
warning is the only sign a lost token leaves.

## Artifact Signing

The blocked public-release scripts retain inherited Sheprd package identities;
the separate macOS preview route uses the Spaceport identity described above. Ordinary CI accepts
both `penso/herdr-gpui` and `sheprd-labs/sheprd` so
the rename can be validated before transfer; release/promotion dispatch and
publication are restricted to `sheprd-labs/sheprd`, with the same `penso` owner
checks and protected environments. Before publication, configure the successor's
environment approvals, secrets, Azure OIDC federation and provenance trust policy.
This source change performs no transfer or deployment. Netlify's Git integration
and the root `netlify.toml` now own website publication; there is no Cloudflare
Actions deploy job.

The compatibility exceptions are intentional:

- macOS preview bundles are `Spaceport.app`, with `spaceport` and `sheprd-acp`,
  signed for Team `HD9NM9NSMK`. The preview does not approve inherited Homebrew
  destinations or any public release identity.
- Homebrew publishes `sheprd-labs/tap/sheprd` for new installs. For existing ones,
  `penso/tap/herdr-gpui` installs `Sheprd.app` to
  **`Herdr.app`**, preserving its existing installation destination. The cask exposes
  `spaceport` on PATH. Moving that target or cask token needs a separate migration.
- Linux packages retain package ownership **`herdr-gpui`**, version `VERSION-1`,
  but ship one app command at `/usr/bin/spaceport`. The desktop ID is
  `org.sheprd.Sheprd`, matching the app's Wayland app ID and X11 `WM_CLASS`.
  The chat helper remains `sheprd-acp`. Distro packaging rejects symlinks.
  Nix wraps the same `spaceport` command.
- Windows retains AppId `{7FA70894-4FC0-4923-BDCB-B05F7475610A}`,
  `%LOCALAPPDATA%\Programs\Herdr UI`, AppUserModelID `so.pen.herdr-gpui`,
  winget `Penso.HerdrUI`, publisher `Herdr UI`, and the configured Authenticode
  subject. ZIPs and installers ship `spaceport.exe` / `spaceport-launcher.exe`
  and `sheprd-acp.exe`; **all executables are signed**. Shortcuts
  and Explorer text show Sheprd. The installer registers neither inherited
  `sheprd://` nor `herdr-gpui://` scheme.
  Setup gracefully stops automation running under either executable name.
- Runtime service/state identities and the upstream `herdr-core` / `hc` and
  `herdr` command remain compatible. Preview signing uses the explicit environment
  interface above; no local `.envrc` or signing hook is sourced.

`scripts/verify-release.sh --legacy` selects legacy package names and the
`penso/herdr-gpui` Sigstore/provenance identity. Default verification requires
the successor identity; neither mode falls back to another trust identity. The
manifest generator's corresponding option is `--legacy-names`. Both modes cover
the retained package artifacts and checksum layouts. They no longer verify
complete historical upstream releases containing retired updater feeds and
archives.

The release base set contains the DMG, two manual Linux archives and their
`.deb`, `.rpm` and Arch packages, two Windows zips, two Windows installers,
and the SBOM. All base files receive SHA-256 and Sigstore sidecars and GitHub
provenance in the separate protected OIDC job.
Starting with `20261102.2`, `artifact-manifest.py create` validates and combines
the signer's SHA-512 sidecars into one `SHA512SUMS` file. `SHA256SUMS` covers
every other published file, including `SHA512SUMS`. The six bare Linux package
aliases also appear in both manifests and must equal their signed base files.
The compact layout retains every platform, signature, and checksum algorithm.
Earlier versions use the per-file SHA-512 layout. Missing or additional files
fail closed. `artifact-manifest.py base-names VERSION DIRECTORY` lists the
retained package and SBOM base names.
The publication job verifies downloaded draft bytes and the complete exact asset
set before making it public; Homebrew still verifies and uses only the final DMG.

## Beta Channel

`just release-beta` publishes a normal release as a GitHub prerelease and skips
Homebrew; `just release-promote VERSION` runs `.github/workflows/promote.yml` to
make that published beta the latest stable release and update the tap, without
rebuilding or re-uploading anything. `generate-changelog.sh VERSION OUTPUT_DIR
[BETA_TAG...]` folds the listed unpromoted beta tags into the release after them.
Beta publication does not provide an in-app update channel.

## Homebrew

`homebrew/Casks/sheprd.rb` and `herdr-gpui.rb` are deliberately templates, not installable
unverified releases. Render with the final stapled DMG's SHA-256 (64 hex digits).
Rendering is deterministic, normalizes hexadecimal to lowercase, and writes only
stdout. Do not redirect onto the input template. The rendered cask belongs at
`Casks/sheprd.rb` in `sheprd-labs/homebrew-tap`, installed as
`brew install sheprd-labs/tap/sheprd`, or `Casks/herdr-gpui.rb` in
`penso/homebrew-tap`. Downloads use repository
`sheprd-labs/sheprd`, tag `vVERSION`, and the exact DMG filename above.

`bash scripts/release/update-homebrew.sh VERSION RENDERED_CASK TAP_REPOSITORY`
publishes that cask to the resolved default branch of `sheprd-labs/homebrew-tap` or
`penso/homebrew-tap`, refusing a cask that is not that tap's; each release runs it for both, with `SHEPRD_TAP_SSH_KEY` and
`HOMEBREW_TAP_SSH_KEY` respectively, because Homebrew will not move an installed
cask to an untrusted tap. Run only after the
published DMG checksum has been verified. It requires `git`, `ssh`, `curl`, `jq`,
and the literal private deploy key in `HOMEBREW_TAP_SSH_KEY`, supplied only to the
approved Homebrew workflow step. It uses HTTPS GitHub metadata for strict SSH host
verification, stages only the cask, and skips identical same-version content.
It rejects numeric version downgrades, changed same-version content (including
checksums), and malformed or missing versions in an existing cask without
evaluating Ruby. Leave newer tap releases intact; publish a new version for changed
assets, and manually review/repair malformed tap metadata before retrying.
The key and clone
are trap-cleaned on exit/INT/TERM (not SIGKILL or power loss). No personal SSH keys,
PAT, credential helper, or global author configuration is used. Keep tap Actions
disabled and the key scoped to this tap; see the root README's protection setup.

## Third-Party Notices

Python 3.9+, the pinned Rust toolchain, and **cargo-about 0.9.2** are required:

```sh
cargo install cargo-about --version 0.9.2 --locked --features cli
python3 scripts/release/generate-notices.py OUTPUT_DIR/THIRD-PARTY-NOTICES.txt
```

The Python wrapper checks that exact tool version, then runs `cargo about generate
--locked --all-features --workspace --fail --config scripts/release/about.toml
--format json`. It renders the tool's unique license texts and package associations
directly, preserving copyright in its evidence rather than inventing holders from
Cargo authors. No separate Handlebars template is needed. Cargo-about may download
package sources and retrieve upstream license evidence; when no matching text is
found, its pinned standard SPDX corpus supplies canonical text. Each report section
identifies collected evidence versus canonical text. Canonical text does not by
itself recover every upstream attribution. Network/cache differences may affect
retrieved evidence, so byte-identical reports across machines are not guaranteed.

`about.toml` filters to the union of `aarch64-apple-darwin`,
`x86_64-apple-darwin`, `aarch64-unknown-linux-gnu`, and
`x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, and
`aarch64-pc-windows-msvc`, matching the packaging
targets. All features and build/dev dependencies remain included, a
conservative superset of any one release binary, not its exact linked inventory.
The accepted-license list covers the current graph's reviewed choices; Apache is
preferred for dual licenses. `CDLA-Permissive-2.0` covers `webpki-roots` trust data
and requires its agreement text with redistribution. MPL is accepted only for
`cbindgen 0.28.0` (build tool), `option-ext 0.2.0`, `nucleo-matcher 0.3.1`,
`cssparser 0.38.0` and `dtoa-short 0.3.5` (via `ammonia`), the
`symphonia` MP3 crates, and
`dwrote 0.11.5` (Windows only). The wrapper rejects version
changes to those exceptions until reviewed. The report gives version-specific
crate-source download links, including these unmodified MPL sources. Keep those
sources available and re-review obligations if dependencies are modified.

The UTF-8 report includes the lockfile SHA-256, package name/version, Cargo source,
declared license expression and cargo-about license texts. A supplemental scan
preserves available package-root LICENSE,
LICENCE, COPYING, COPYRIGHT and NOTICE files (including suffixes), all files
recursively in root `license`, `licenses`, `licence`, `licences`, `legal`
directories, and every explicit Cargo `license-file`. This preserves notices and
alternative license texts even when they were not selected by cargo-about.
Escaping paths, empty/unreadable discovered files, unresolved or unaccepted
licenses, failed tool invocations, missing package coverage, and a changed
lockfile fail generation. A missing source license file is not alone a failure
when cargo-about resolves a standard SPDX license. No report is written until
resolution and collection succeed; existing reports are refused to prevent stale
reuse. Neither Cargo.lock nor dependency versions are updated by this command.

All package interfaces require a nonempty report. macOS includes it in
`Contents/Resources/THIRD-PARTY-NOTICES.txt`; Linux includes it in
`share/licenses/herdr-gpui/THIRD-PARTY-NOTICES.txt`; Windows includes it in
`licenses/THIRD-PARTY-NOTICES.txt`. Secret-free build jobs generate
reports before packaging. Each macOS binary artifact carries its report; assembly
requires the two reports to match byte-for-byte before signing. Local DMG builds
also generate notices before sourcing signing configuration.

All bundles also preserve root `LICENSE`, `NOTICE`, protocol `LICENSE-APACHE`
and `NOTICE.md`, and `assets/icons/LICENSE-octicons` as separate files.

**Public release review is required:** generation is an inventory, not legal
approval or proof that every embedded third-party component was found. Review
license expressions (including unknown/custom and dual-license choices), text
coverage, copyright and NOTICE obligations, nested vendored code, native libraries,
and non-Cargo assets before approving the release environment. Do not treat the
presence of a report or a configured allowlist as legal approval. Where review
finds missing upstream notices, add version-bound evidence with provenance rather
than fabricated fallback text or a broad license exception.

## Local macOS preview

Use Apple Silicon macOS with Xcode command-line tools, the pinned Rust toolchain,
`jq`, and `cargo-about 0.9.2`. The build command takes explicit profile, architecture,
and signing mode; its defaults are **debug arm64 unsigned**. Acquire the pinned
SDK once in an unused output directory, then export its path before building:

```sh
python3 scripts/release/sparkle_framework.py acquire target/sparkle-sdk
export SPARKLE_FRAMEWORK_PATH="$PWD/target/sparkle-sdk"
bash scripts/release/build-macos.sh VERSION debug arm64 unsigned
bash scripts/release/build-macos.sh VERSION debug arm64 installed
# The just recipe always requests installed-identity signing:
just dmg VERSION debug arm64
```

For `installed`, supply `MACOS_SIGNING_IDENTITY`, `APPLE_API_KEY_PATH` (the existing
`.p8` file path), `APPLE_API_KEY_ID`, and `APPLE_API_ISSUER_ID` through a trusted
process environment. This mode neither imports certificates nor changes keychains.
`import` uses the six protected-environment inputs listed above and a temporary
keychain; the P12 must contain only the selected Developer ID Application identity.
No `.envrc` is needed or sourced. Never put private key contents or passwords in
shell history. The signer rejects the wrong app, Team selection or architecture
before opening credentials, then verifies the signed app's actual Team and ID.

`release` selects optimized compilation; `universal` also builds the Intel target
(which must first be installed with `rustup target add x86_64-apple-darwin`).
The arm64 executable receives CLI smoke checks. Intel cross-compilation and lipo
validation do not establish Intel-native runtime correctness. Debug artifacts are
labelled in `SpaceportBuildProfile`, their output directory and the DMG JSON record.
Builds use `--locked`, `CARGO_INCREMENTAL=0`, `MACOSX_DEPLOYMENT_TARGET=14.2`, and
`CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_STRIP=none`. All Apple signing variables,
including the private-key path, are removed before Cargo and notice generation.
Only `spaceport` and `sheprd-acp` ship; no Herdr daemon is installed or bundled.

Output lives in `target/distribution/VERSION-PROFILE-ARCHITECTURE`. Unsigned mode
returns `Spaceport.app`; signing returns `Spaceport-VERSION-ARCHITECTURE-apple-darwin.dmg`
and a `.dmg.json` record with SHA256, app ID, Team, version, profile and architecture.
The adjacent `Spaceport.app` remains the unsigned assembly; the DMG contains its
signed and stapled copy. Existing output directories are never overwritten.

Notarization submits each app ZIP and DMG **once**, saves the returned UUID, then
waits on that UUID. Immutable version/architecture reservations and archive SHA256
receipts live in ignored `target/notarization` (override with an absolute
`SPACEPORT_NOTARY_RECEIPTS`). Versions are unique across debug and release profiles.
Receipts survive failure and cancellation; repeated versions refuse a new submit,
even when signing timestamps change the bytes. Pending/rejected results retain
status and rejection logs. A missing UUID means **outcome unknown**, never permission
to submit again. Inspect Apple history and the retained receipt before taking any
further action. There is no automatic resume or re-submit command. Keep/download
receipts from a hosted run before any later manual run; hosted runners have no
cross-run local state. The history guard also refuses repeat versions; downloaded
artifacts provide evidence, not an automatic recovery mechanism.

Temporary signing credentials are removed on exit/INT/TERM, and imported keychains'
original search list is restored. SIGKILL and power loss cannot run cleanup.
The local route builds, signs, notarizes and retains artifacts only: it does not
launch/install an app, publish a feed or create a public release. Its assembled
preview contains updater configuration, but compiled worktree builds remain
ineligible. The tagged workflow performs the separately authorized publication.

## Contributor CI Approval

In repository **Settings > Actions > General**, keep **Require approval for all
external contributors** enabled. This GitHub setting, outside the workflow YAML,
controls whether fork pull-request workflows can start. Review the proposed
changes (especially workflows and build scripts), then use **Approve workflows
to run**. Any maintainer with write access can approve; adding collaborators with
write access also gives them that authority.

The CI workflow accepts `pull_request` runs without requiring the PR author,
original actor, or source repository to be the owner. An owner rerun only changes
`github.triggering_actor`; approval does not change the other identities.
Pushes and manual dispatches remain restricted to owner-triggered `main` runs.
The final **Format, lint, and test** gate requires successful platform checks for
code changes, and the macOS optimized builds on pull requests, and permits
skipped platform jobs only for confirmed non-code edits.

Fork PR execution uses GitHub-hosted runners and read-only permissions. GitHub
withholds repository secrets from fork `pull_request` runs, including after
approval. PR jobs must not request protected environments or OIDC, and checkouts
must not persist credentials. Do not switch to `pull_request_target` to run PR
code or add a privileged `workflow_run` consumer of PR artifacts. CI caches are
public data: never place credentials in them. Release builds use their own
workflow and must not consume fork-produced artifacts or caches; signing and
publication retain their owner/main restrictions and protected environments.

After changing this policy, validate with a real external fork: confirm its run
waits for approval, approve it and confirm the platform jobs execute, then push
another contributor commit and check approval is required again. Offline tests
cover job eligibility and credential declarations, not GitHub's approval service.

## Local Validation

```sh
for script in scripts/release/*.sh; do bash -n "$script" || exit; done
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/release/tests -v
PYTHONDONTWRITEBYTECODE=1 python3 scripts/release/test-release-security.py
actionlint .github/workflows/release.yml
zizmor --offline .github/workflows/release.yml
```

Tests run on macOS and Linux with Python 3, Git and `jq`. They use dummy credentials
and mocked Apple tools/OS detection, check rejection and cleanup paths (including
macOS-only production guards), and inspect both Linux target tarballs.
Tap tests mock HTTPS/SSH transport and push only to disposable local repositories;
they cover default-branch resolution, cask-only commits, no-op updates and failures.
They do not perform real signing, notarization, Gatekeeper assessment or native
launch testing. Real release validation still needs the trusted signing runner.
