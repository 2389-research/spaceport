# Release Packaging

**Spaceport development status:** The only desktop release route in this fork is the
macOS preview route below. The separate [iOS TestFlight route](#ios-testflight-route)
uploads the mobile client to TestFlight and publishes no desktop artifact. The inherited
upstream production route (its release
and promotion workflows, Linux packages, Homebrew, winget, artifact signing and
SBOM) has been removed; restore it from git history when the release gate
opens. The current release build rejects
`HERDR_RELEASE_VERSION` because general production identity, update delivery,
and native package checks remain unresolved. The separate macOS preview
route below uses the approved 2389 Developer ID inputs and `ai.2389.spaceport`;
that approval does not enable general production releases. See the
[release identity audit](../../docs/spaceport/evidence/release-identity-audit.md)
and [open gates](../../docs/spaceport/plan/gaps.json). D64 approves the
separate tagged prerelease route below.
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
Any collaborator with write access to `2389-research/spaceport` may push
`vYYYYMMDD.COUNTER`; the tag builds an optimized app, signs and notarizes it, and
publishes a GitHub **prerelease** with the DMG and `SHA256SUMS`. Manual runs on
main accept an exact commit SHA and an explicit version; they upload workflow
artifacts without publishing a release.
The [20261117.1 prerelease](https://github.com/2389-research/spaceport/releases/tag/v20261117.1)
first completed this route with Sparkle enabled, and
[20261125.1](https://github.com/2389-research/spaceport/releases/tag/v20261125.1) is the
newest published one.
The [qualification record for 20261117.1](../../docs/spaceport/evidence/t12-macos-preview-20261117.1-signed.md)
covers that public DMG and signed feed; the old feed URL redirect was not rerun.
Both routes require the selected commit to belong to main and have a complete,
successful main CI run. D66 temporarily pauses Spaceport Windows CI until
Kata `h8g5` fixes native performance; macOS/Linux checks remain mandatory,
and the aggregate gate requires Windows jobs to be skipped. No Windows
performance budget or acceptance result changes.
The signing and publishing jobs re-fetch the tag and refuse it if it moved.
Only the publishing job receives `contents: write`.

The unsigned build has no signing secrets and runs `smoke-cli.py` on its actual
executable. The signing job uses the protected `macos-preview` environment and
only that run's checksummed artifact. It verifies signatures, notarization tickets
and Gatekeeper, mounts the final DMG read-only, checks the app identity, and runs
the mounted CLI before writing the checksum. This does not prove native GUI behavior.

Unsigned macOS builds restore dependency caches, pinned helper binaries and the public Sparkle SDK.
The main-only `cache-macos.yml` workflow warms matching profiles when build inputs change.
It also runs nightly, building both macOS targets and running the release-mode CLI suite on main.
Pull requests run those macOS release builds only when `scripts/ci-changes.sh` reports `macos_release=true`.
Release tags read main caches because GitHub isolates caches between different tags.
Signing and publication jobs remain cache-free.
Tool installers can choose different binary directories on hosted runners.
`cache-release-tools.sh CACHE_DIRECTORY TOOL...` stages those binaries into one fixed cache directory.
The cache action checks their pinned versions and adds that directory to PATH.
Cache only Cargo release profiles. Empty packaging directories must not restore beside build outputs.

`bash scripts/release/prepare-sparkle-sdk.sh SDK_DIRECTORY [PINNED_ARCHIVE]` acquires missing SDKs and validates existing bytes.
It refuses changed archives or framework files and preserves them for diagnosis.
Keep SDK directories outside Cargo's target tree, which rust-cache cleans.
To warm caches manually, run `gh workflow run cache-macos.yml --repo 2389-research/spaceport --ref main`.

For a fresh repository, merge these workflows to main and configure the
`macos-preview` environment in repository Settings > Environments. Allow main
branches and `v*` tags through its selected deployment rules. Restrict who can
create or update those tags with repository access rules. The activated repository
now belongs to `2389-research`; organization members inherit its repository
permissions. The move retains the existing signing environment and its main/tag
rules. Workflow eligibility names no login: any collaborator with write access
may push a `v*` tag or dispatch the workflow on main. That access is the
signing boundary, so required reviewers belong on the `macos-preview`
environment in repository Settings. Add these
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
updater feed; general production publication remains excluded.

### Updater channels

Sparkle is the production update mechanism, and the same route serves both signed
macOS channels. Installed updater-enabled builds expose **Check for Updates…** in
the app menu and `spaceport run check-for-updates`. Sparkle asks permission for
automatic checks; installation requires confirmation by default. Ordinary
development and compiled worktree builds cannot start the updater. The first
enabled build requires one manual installation; older copies cannot bootstrap
themselves. Updates replace Spaceport and its bundled chat adapter, never either
Herdr runtime.

Only the preview channel has a publication route today. The `macos-stable-feed`
release name and its URL are configured and checked, but no workflow creates that
feed, no release builds the production channel, and publishing it needs its own
owner authorization.

The build embeds pinned Sparkle 2.10.1 and its license, preserving framework
symlinks. Signing rechecks all original framework bytes before credentials,
then signs nested code inside out. The final app and DMG retain the existing
Developer ID/notarization/Gatekeeper checks. `sparkle.json` is the shared public
SDK version/checksum, Ed25519 trust configuration and channel table: one feed per
signed channel, each the fixed release asset named beside it.

```json
"feeds": {
  "macos-preview": { "release": "macos-preview-feed", "url": ".../macos-preview-feed/appcast.xml" },
  "production":     { "release": "macos-stable-feed",  "url": ".../macos-stable-feed/appcast.xml" }
}
```

The preview packager and signer require `channel=macos-preview` in each
compiled identity record before accepting a binary. Rebuild unsigned inputs
that predate this field. Existing published previews still read the same signed
feed. The preview packager writes its URL as `SUFeedURL`; the app checks it
against the same table at startup, so a preview never follows the stable feed and
a production build never follows the preview's. Feed URLs are https release
assets served from one host, and a bundle carrying any other address refuses to
start the updater. `sparkle_framework.py configure --channel production` writes
the stable channel, which no release route invokes yet.

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

Signed builds use `SPACEPORT_MACOS_PREVIEW_VERSION` (the preview channel) or
`SPACEPORT_PRODUCTION_VERSION` (the production channel), one at a time, and
report `ai.2389.spaceport` in `Spaceport.app`. The compile-time selector rejects
other target operating systems, malformed versions and both variables at once.
The app ID files beside `crates/spaceport-gpui/build.rs` are the source for
compiled and bundle identities; a final newline is allowed. Ordinary `just bundle`
builds retain `spaceport.development`. The general `HERDR_RELEASE_VERSION`
refusal remains in force, so no production release is built from that route. The
app reads its build channel (`development`, `macos-preview` or `production`) from
one compiled constant and derives everything else from it. Signed builds share one
app identity, so the channel - never the bundle ID - decides which feed is
followed. Signing grants no provider credential reads or automatic skill install.
The existing three-line `--build-info` output remains unchanged.

### Android APK on preview releases

`android-release.yml` runs when a tag-push run of the macOS preview workflow succeeds.
It never changes the macOS route. `android-request.py` runs from main, never from the tagged commit.
It re-proves the tag, its commit and main ancestry.
It then compares the APK's sources with the newest earlier release that carries an `-android.apk` asset.
The sources are the Android app, the controller crate, the shared crates it reaches by path, and the root build inputs.
The `android-release` environment must allow `main`: under `workflow_run`, the job's ref is main, not the tag.
It needs no reviewer: pushing the `v*` tag is the release decision, as for macOS.
Anyone who may push a `v*` tag can therefore cause a signed Android build.
If they differ, the protected `android-release` environment signs a release APK, and the workflow attaches it to the tag's release.
`mobile/README.md` lists the environment's secrets and the pinned certificate variable.

## iOS TestFlight route

`ios-testflight.yml` signs the Spaceport Mobile app and uploads it to TestFlight. It is a
mobile route: it publishes no desktop artifact, and it changes neither the macOS preview
nor the Android route. It runs when a tag-push run of `macOS signed preview` succeeds,
because that run's `publish` job creates the tag's release and this route attaches its
`.ipa` there. It never runs for a pull request or another repository. A manual run on main
takes an explicit `version`. A supplied tag must exist and point to the selected main commit.
With no tag, the workflow uploads to TestFlight and keeps the files as workflow artifacts.

For an automatic upload, the workflow checks the release tag against the triggering commit
and confirms the commit belongs to `main` before building its code.

`build` runs `./mobile/build-apple --release` with no credentials and archives only what
that produced — `mobile/.build/SpaceportMobileCore.xcframework`, `mobile/apple/Generated`
and the generated `SpaceportMobile.xcodeproj` — under a `SHA256SUMS`. `sign` downloads
only that checksummed artifact and holds the credentials. `publish` re-proves the tag and
attaches `Spaceport-VERSION-ios.ipa`, its `.sha256` checksum, and its `.json` upload receipt
to the GitHub release if it exists.

The iOS simulator suite is a **skipped gate**, not a silent pass. `mobile/README.md`'s
verification target is Product → Test inside Xcode, whose live tests need a disposable SSH
host, seeded Herdr sessions and a hand-driven Duo fold, and `just test-ios-duo` needs an
open simulator on a desktop. Neither is a CI-runnable suite, so the build job records the
absence with `TODO(ios-simulator-gate)` and runs nothing in its place.

`bash scripts/release/sign-ios.sh VERSION OUTPUT_DIR` performs signing, verification and
the upload:

- VERSION is a calendar version with a counter from 1 through 99. The iOS build number is
  `YYYYMMDD*100+COUNTER`, the rule `android-request.py` applies to the Android
  `versionCode`, passed to xcodebuild as `CURRENT_PROJECT_VERSION` so every later tag
  uploads a greater TestFlight build. `MARKETING_VERSION` stays `0.1` from
  `mobile/apple/project.yml`, which keeps the existing App Store Connect record and the
  export-compliance note in `mobile/README.md` valid.
- Install an App Store distribution profile for each Release target. The app uses
  `SPACEPORT_MOBILE_PROFILE` and Agent Status uses `SPACEPORT_ACTIVITY_PROFILE`.
  Builds that include the Share extension also use `SPACEPORT_SHARE_PROFILE`.
  Supply the profiles as base64 with the corresponding `_B64` environment variables.
  For a local run, supply all required `_UUID` variables to use Xcode provisioning updates instead.
- The script archives the Release scheme for `generic/platform=iOS` and exports with
  manual signing. Each bundle identifier maps to its own profile in the export options.
  Xcode preserves the build number derived from the tag.
- Before upload, the script checks the exported app and its expected extensions. It verifies
  their signatures, bundle identifiers, build numbers, Team and shared App Group.
  Each embedded profile must match its bundle and disable development debugging.
- It uploads with `xcrun altool --upload-app --type ios` and the App Store Connect key,
  then writes `Spaceport-VERSION-ios.ipa.json` beside the `.ipa` and its `.sha256` with the
  bundle id, build number, marketing version, Team, App Group, digest and upload result,
  and prints the same facts as `key=value` lines. altool's exit status is the result; a
  failure reports Apple's product errors and refuses a second upload of that build number.
- Credentials follow `sign-macos.sh`: the P12 is imported into a temporary keychain inside
  a private temporary directory, the user keychain search list is restored, the `.p8` sits
  under a private `private_keys` directory that `altool` reads, and exit/INT/TERM remove the
  keychain, the profiles this run installed and the directory. SIGKILL and power loss cannot
  run cleanup, and Apple's tools receive the P12 password as an argument, so the runner must
  have no untrusted local users. Existing outputs are refused before credentials are read.

### Owner switch-on checklist

The build job can run before you configure signing. GitHub can create a missing environment
when a job references it. Missing signing inputs stop the sign job before upload.
[GitHub deployment environments](https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments)

1. In **Settings → Environments**, create the environment `ios-testflight`. Allow `main`
   through its deployment rules: under `workflow_run` the job's ref is main, not the tag,
   exactly as the `android-release` environment requires. Required reviewers are optional —
   pushing the `v*` tag is the release decision, as it is for macOS and Android.
2. Create this **variable** on that environment (not in a source file):
   `SPACEPORT_APPLE_TEAM_ID` — the 10-character Apple Developer Team ID. Every bundle, provisioning profile and the distribution certificate must belong to this team.
3. Create these **secrets** on that environment:

   | Secret | What it is |
   | --- | --- |
   | `IOS_DISTRIBUTION_P12_BASE64` | Base64 of an `Apple Distribution: 2389 Research, Inc (TEAMID)` certificate exported from the team's developer account as a `.p12` with its private key. The script rejects a P12 holding more than one identity. This is not the macOS Developer ID certificate. |
   | `IOS_DISTRIBUTION_P12_PASSWORD` | That P12's export password. |
   | `SPACEPORT_MOBILE_PROFILE_B64` | Base64 of the App Store distribution profile for `ai.2389.spaceport.mobile`. |
   | `SPACEPORT_ACTIVITY_PROFILE_B64` | Base64 of the App Store distribution profile for `ai.2389.spaceport.mobile.agent-status`. |
   | `SPACEPORT_SHARE_PROFILE_B64` | Base64 of the App Store distribution profile for `ai.2389.spaceport.mobile.share`. Required when the release includes ShareFile. |
   | `APPLE_API_PRIVATE_KEY` | The App Store Connect `.p8` contents. Reuse the macOS route's key. |
   | `APPLE_API_KEY_ID` | That key's ID. Reuse the macOS route's value. |
   | `APPLE_API_ISSUER_ID` | That key's issuer ID. Reuse the macOS route's value. |

4. Use a **Developer** App Store Connect API key for this upload. The base64 profile route
   uses pre-issued profiles. An Account Holder or Admin can create those profiles separately.
   TestFlight tester management has its own role requirements.
   [Apple build uploads](https://developer.apple.com/help/app-store-connect/manage-builds/upload-builds/)
   · [App Store distribution profiles](https://developer.apple.com/help/account/provisioning-profiles/create-an-app-store-provisioning-profile/)
5. In the developer account, confirm the App Store Connect record
   `ai.2389.spaceport.mobile` exists. Enable `group.ai.2389.spaceport.mobile.widget`
   for each shipped bundle identifier. Create a distribution profile for each target after
   enabling the group. When a profile expires, replace its matching environment secret.
6. Confirm the app record carries the export-compliance answer described in
   `mobile/README.md`; the Release plist already ships `ITSAppUsesNonExemptEncryption`
   `false`, and a build that Apple will not import is reported by the upload step.
7. Push a `v` tag for a commit whose macOS preview run succeeds:

   ```sh
   git tag -a v20261218.1 FULL_MAIN_COMMIT_SHA -m 'Spaceport 20261218.1'
   git push origin refs/tags/v20261218.1
   ```

   After the macOS preview publishes the release, this workflow uploads the build to TestFlight
   and attaches the IPA, checksum and upload receipt to the release. The receipt and the
   signing job summary record Apple's upload result. Processing continues in App Store Connect.

A rerun of the same tag rebuilds and re-uploads the same build number, which Apple refuses.
Use a new version instead, and inspect the retained `.ipa` artifact and job log first. No
receipt here proves native behavior on a device: the skipped simulator gate above covers
what this route does not.

## Build Identity

Packaging and `just bundle` require Python 3 and read a versioned identity record
embedded in the supplied executable, never the packaging checkout's Git state.
On Linux, linked-worktree binaries select `assets/icons/spaceport-linux-worktree.svg`;
other builds select `assets/icons/spaceport-linux.svg`. Both circular variants are
meant to be installed as `share/icons/hicolor/scalable/apps/spaceport.svg`,
without a pixmaps fallback.
Regenerate both SVGs with `just icons-linux`; see the
[icon guide](../../assets/icons/README.md#linux-and-windows) for the composition and
desktop lookup behavior. No Linux package is built here now; the Nix flake
installs the ordinary Linux icon. On macOS, linked-worktree binaries select
`assets/icons/Spaceport-worktree.icns` and `Spaceport-worktree.car`; other builds use
`Spaceport.icns` and `Spaceport.car`. The installed icons keep their frozen
bundle filenames (`Contents/Resources/Sheprd.icns`, `Assets.car`).
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

```sh
cargo install cargo-about --version 0.9.2 --locked --features cli
python3 scripts/release/generate-notices.py OUTPUT_DIR/THIRD-PARTY-NOTICES.txt
bash scripts/release/package-macos.sh VERSION ARM64_BINARY X86_64_BINARY OUTPUT_DIR OUTPUT_DIR/THIRD-PARTY-NOTICES.txt
bash scripts/release/sign-macos.sh VERSION OUTPUT_DIR/Spaceport.app OUTPUT_DIR universal import
```

macOS assembly requires Apple command-line tools. It produces unsigned
`Spaceport.app`, containing the arm64 or universal GUI, plist, icon, root `LICENSE` and `NOTICE`,
protocol license/attribution, Octicons license, and third-party notices.
Both plist versions are set to the supplied version.
The package floor is macOS 14.2 (`LSMinimumSystemVersion`). The binary strongly links CoreAudio's process-tap functions,
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

The Windows targets are `x86_64-pc-windows-msvc` and
`aarch64-pc-windows-msvc`. No workflow publishes a Windows package now; CI's
`windows-package` job, paused for the fork under D66, is the only caller of
these scripts. `scripts/release/package-windows.py VERSION TARGET BINARY LAUNCHER
OUTPUT_DIR THIRD_PARTY_NOTICES` (standard-library Python, since the runner has no
zip tool) writes one `Spaceport-VERSION-TARGET.zip` per target with a same-named root
containing `spaceport.exe`, the windowed `spaceport-launcher.exe` that the
installer's Start menu shortcut opens, `sheprd-acp.exe`, which the app starts
chats with (taken from beside `BINARY`), and `licenses/` holding the license
and notice files. Entries carry a fixed timestamp.

`scripts/release/windows-installer.py` then installs the pinned Inno Setup
compiler (`install-iscc`, checked against its SHA-256), compiles
`spaceport.iss` over the extracted zip tree into
`Spaceport-VERSION-TARGET-setup.exe` (`build`), and installs and uninstalls it on
the runner (`smoke`), checking the files, their subsystems, `spaceport.exe`'s
output, the Start menu shortcut's target (the launcher) and AppUserModelID,
that the AppUserModelID is the one build record the executable itself carries,
the user `PATH` and Settings entries, and a silent reinstall that preserves
startup settings without launching the app.
CI's `windows-package` job runs the same
optimized build, packaging, installer build, and install/uninstall check on
`windows-2025` for any pull request that changes their inputs
(`scripts/ci-changes.sh` lists them), and the required gate then requires it. The installer is per-user (no elevation),
installs to `%LOCALAPPDATA%\Programs\Spaceport`, adds that directory to the
user `PATH`, and creates the Start menu shortcut to the launcher.
The smoke test seeds a disabled `Spaceport` startup entry naming the installed
launcher and requires the command and `StartupApproved` bytes to survive the
upgrade unchanged. Setup leaves a preexisting desktop shortcut alone even when a
run does not select the desktop task.

## Signing Gates

Signing is scaffolding here: the steps exist, no build signs, and no credential
lives in the repository. A signed artifact needs all of the following for its
platform.

| Platform | Mechanism | Credential, and where the step plugs in | State |
| --- | --- | --- | --- |
| macOS | Developer ID signature with hardened runtime, notarization, stapling | A 2389 Research Developer ID certificate for Team `HD9NM9NSMK`, `notarytool` credentials for the same team, and the Spaceport Sparkle Ed25519 key, all in the protected `macos-preview` environment. `sign-macos.sh` and the preview workflow run them. | Configured for previews (D61, D64, D65). |
| Windows | Authenticode with an RFC 3161 timestamp on the executables and the installer | An Azure Artifact Signing endpoint, account and certificate profile plus an Azure CLI OIDC login, named by `ARTIFACT_SIGNING_ENDPOINT`, `ARTIFACT_SIGNING_ACCOUNT` and `ARTIFACT_SIGNING_PROFILE`. `windows-installer.py build --sign` points ISCC at `windows-sign-file.ps1` for Setup.exe and its uninstaller; `windows-signing.py unpack`, `repack` and `verify` sign and check the zip's tree. | Nothing calls them (D61 defers Windows signing); a protected signing job must set up that login and the three variables first. |
| Linux | Detached signature over the published archive and its checksum list | No key exists, and no archive is published: this repository builds the `spaceport` binary and the Nix package only. A `.deb`, `.rpm` or AppImage and its signing arrive with the Linux update route. | Not started. |

Every path stays unsigned until then, and each is tested: `windows-package` CI
builds the zip and the installer with no sign tool and runs `smoke` (unsigned)
on them, the macOS preview route refuses an unsigned input before packaging,
and `--sign`, `smoke --signed` and `windows-signing.py verify` reject an
artifact without a valid timestamped signature from one signer.
`test-release-security.py` keeps signing credentials out of every workflow, so
restoring signing needs a signing job, not a code change here.

## Package Identities

- macOS preview bundles are `Spaceport.app` under `ai.2389.spaceport`, Team
  `HD9NM9NSMK`, signed and notarized as above; ordinary development bundles
  keep `spaceport.development`. The preview approves no other public release
  identity.
- Windows packages are Spaceport's own: product name `Spaceport`, AppId
  `{62B8722B-1FA2-48E0-9609-46AA5D431983}`, `%LOCALAPPDATA%\Programs\Spaceport`,
  publisher `2389 Research, Inc.`, and `Spaceport-VERSION-TARGET` archive and
  installer names. The Start menu shortcut and the executable's `FileDescription`
  are both `Spaceport`. The installer's AppUserModelID is the app ID the shipped
  executable embeds in its build record — `spaceport.development` while packages
  carry the development identity — so notifications and taskbar grouping stay
  together. Setup neither upgrades nor adopts an inherited Sheprd installation:
  that one keeps its AppId, directory, shortcuts and startup entry. The installer
  registers neither inherited `sheprd://` nor `herdr-gpui://` scheme, and
  `CloseApplications` stops only the executables it replaces.
- Linux packaging installs `scripts/release/spaceport.desktop` as
  `spaceport.development.desktop` under `share/applications` and the icon as
  `share/icons/hicolor/scalable/apps/spaceport.svg`, both by Nix
  (`flake.nix`); `Icon=spaceport`, `Categories=Development;` and
  `StartupWMClass` naming the app ID the window reports. The chat helper remains
  `sheprd-acp`. A distribution package (`.deb`, `.rpm`, AppImage) does not exist
  yet, and its package name is `spaceport` when it does.
- `site/install.ps1` is still the inherited Sheprd installer, for a different
  product than these packages, so the two no longer share a directory, a
  shortcut or an AppUserModelID.
- Runtime service/state identities and the upstream `herdr-core` / `hc` and
  `herdr` command remain compatible. Preview signing uses the explicit environment
  interface above; no local `.envrc` or signing hook is sourced.

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
`Contents/Resources/THIRD-PARTY-NOTICES.txt`; Windows includes it in
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
Pushes and manual dispatches name no login either: any collaborator's push or
dispatch of `main` runs the full CI, and `check-ci.py` accepts whoever ran it.
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
publication retain their main-only restrictions and the protected signing
environment.

After changing this policy, validate with a real external fork: confirm its run
waits for approval, approve it and confirm the platform jobs execute, then push
another contributor commit and check approval is required again. Offline tests
cover job eligibility and credential declarations, not GitHub's approval service.

## Shared release helpers

`_release_common.py` holds the rules more than one release script applies:
the calendar-version and full-commit patterns behind `is_version` and
`is_commit`, and `gh`, which captures stderr and bounds each GitHub CLI call
by timeout. Scripts load it by file path, as they already load each other,
because their dashed names are not importable modules. Add a helper there only
when a second script needs the same rule; each script keeps its own refusal
messages, size bounds and timeouts.

## Local Validation

```sh
for script in scripts/release/*.sh; do bash -n "$script" || exit; done
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/release/tests -v
PYTHONDONTWRITEBYTECODE=1 python3 scripts/release/test-release-security.py
actionlint .github/workflows/*.yml
zizmor --offline .github/workflows/
```

Tests run on macOS and Linux with Python 3, Git and `jq`. They use dummy credentials
and mocked Apple tools/OS detection, check rejection and cleanup paths (including
macOS-only production guards).
They do not perform real signing, notarization, Gatekeeper assessment or native
launch testing. Real release validation still needs the trusted signing runner.
