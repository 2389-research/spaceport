# D65 macOS preview updates

Doctor Biz approved Sparkle for the preview channel on 2026-11-10. This record
qualifies components; it does not close T12, A43 or the native platform gates.

The implementation pins Sparkle 2.10.1 and the `sparkle-updater` 0.1.0 Rust
wrapper. Public SDK/trust configuration lives in `scripts/release/sparkle.json`.
The private Ed25519 key stays outside Git and Cargo, with a separate protected
`macos-preview` environment secret. Its public key is distinct from Developer ID.

The macOS-only optional feature retains Sparkle on GPUI's main thread and uses
its standard native UI. Matching compiled/bundled identity, version, feed, key
and policy are required. Compiled worktrees and development builds fail closed.
Packaging and runtime eligibility require exact numeric
`SUSignedFeedFailureExpirationInterval=0`; missing or nonzero values refuse startup.
The app menu and `spaceport run check-for-updates` share the command catalog;
unsupported and busy requests return errors. Existing AppKit termination runs
GPUI's ordinary save/detach hooks. `HERDR_RELEASE_VERSION` remains refused.

Packaging compares all framework bytes and the SDK's separate license with the
checksum-pinned archive, preserves framework links, and includes the license in
Resources. Signing checks the unsigned bundle again before credentials, signs
only known nested code inside out, and keeps the existing Developer ID, Team,
notarization, receipt, stapling, Gatekeeper and mounted-CLI checks.

The unsigned build generates and checksums release notes alongside the app;
the protected signing job compiles no Cargo tools and signs the final DMG and appcast. Publication verifies
both Ed25519 signatures, the exact version/source tag, newest published version,
the prior signed feed and actual public artifact before updating the fixed
preview feed asset. The marker tag stays fixed. GitHub asset replacement can
briefly make the feed unavailable; a failed check does not install anything.

Sparkle intentionally supports trusted publisher key and app-identity rotation.
Exact Spaceport app ID and Team are enforced by the release pipeline; this is
not an unconditional inner-app-ID rule in Sparkle. Wrong archive key tests use
ZIP to avoid the same-Team signed-DMG key-rotation fallback.

## Source checks and review

- Framework packaging/signing: `6f0b7c5e3`; license follow-up `27f7157e1`.
  Independent spec and quality reviews passed. Targeted license revision checks:
  10 SDK/policy, 9 package and 19 preview tests passed, including real SDK archive
  extraction and rejection after changing framework or license bytes. Apple
  command stand-ins cover script control flow only, not native signing evidence.
- App integration: `c5c299d1c`, corrected by `d1255a42c`. Review caught missing
  bundle rpath and compiled-worktree exclusion; both have red/green regressions
  and passed re-review. Reported final checks: 31 catalog tests, 10 build-identity
  tests, 19 feature and 18 default updater-related app tests, strict targeted
  Clippy and quick formatting/sorted/i18n/file-size checks. Actual Mach-O loading
  passed in a bundle layout with all `DYLD_*` overrides removed. Default builds
  link no Sparkle and receive no updater rpath.
- Feed/workflow integration: `bf327e03d`, independent review passed. Thirteen
  feed tests, three CI boundary tests and seven builder tests passed. Real
  disposable Ed25519 keys and the official SDK signed/verified archive bytes
  and XML. Negative cases cover tampering, wrong keys, foreign URLs, unsigned
  trailing data, length, version regression and foreign bundle metadata.
  Actionlint, ShellCheck and diff checks passed. Mutable publisher orchestration
  still needs actual GitHub workflow qualification; pure crypto tests do not
  prove publication succeeded.

Final whole-branch review found and corrected four issues: a cached Busy menu
state, diagnostics bypassing fixture cleanup, negative tests accepting unrelated
errors, and Sparkle's default recovery after 20 days of invalid feed signatures.
The re-review closed all four and the consent-comment correction. Final targeted
checks passed: 23 feature and 20 default update tests, 15 menu tests, 35 fixture
driver tests, strict app/example Clippy, and the affected signing policy tests.
The driver now checks specific native error chains and the actual download/
validation stage; generic network/init failures cannot pass a signature case.
Real filesystem/evidence faults verify unconditional owned-child cleanup and
failed-result retention. The standalone fixture has one statement-scoped unsafe
NSDate setter for a valid property-list value in its claimed test domain;
production code has no added unsafe.

## Native qualification in progress

The isolated real Spaceport lifecycle test starts a private Herdr 1 daemon and
foreground terminal heartbeat. It drops its observer, quits the exact GUI once,
checks the heartbeat advances after GUI exit, restarts the GUI, and repeats.
The final test at `6c30c7dc3` passed with stable daemon, boot, pane and shell
identity across both GUI exits. All five tests passed, including a real private
daemon cleanup case without the normal sentinel. Cleanup signals only exact
owned child handles; it never signals a recorded shell PID. Targeted Clippy
with warnings denied, formatting and file-size checks passed. The existing
`block 0.1.6` future-compatibility notice remains dependency noise.

The separate native updater fixture uses the same Rust wrapper and GPUI event
loop in signed, uniquely identified test bundles. Developer ID signing works
locally. A real signed v1 host loaded the signed fixture feed and found v2.
Three real native read-only information checks passed: a same-length changed
signed feed and a wrong feed key returned native signature errors; a v2 host
refused an offered v1. Each downloaded no archive, preserved executable and
Info.plist hashes, and left the original process alive before cleanup. A fresh
wrong-feed-key run also verified normal GPUI shutdown via its private quit
sentinel, with no PID signaling. Twenty-five driver tests and strict example
Clippy passed at that stage. These are feed verification/selection checks, not manual UI tests.

A real aged-state red/green check seeded an NSDate 21 days in the past in a fresh
test domain. Sparkle's missing-expiry default accepted the wrongly signed feed
for selection; numeric zero instead returned the actual 1000/3002 signature
error, with unchanged app bytes and no archive download. Evidence remains under
ignored `target/sparkle-native/aged-wrong-feed-key-wor2xu1g` (red) and
`aged-wrong-feed-key-3vegqwgk` (green). This confirms the pinned
[upstream recovery behavior](https://github.com/sparkle-project/Sparkle/blob/2.10.1/Sparkle/SUAppcastDriver.m)
and its explicit zero-interval refusal path without changing the system clock.

On November 11, Doctor Biz unlocked the desktop. Genuine native Install Update
and Install and Relaunch buttons downloaded and replaced a signed ZIP fixture
with version 2. LaunchServices started the new executable, but overwrote HOME;
the fixture's isolation guard refused startup before its launch marker. The
case correctly failed and retained its evidence. A fixture-only bounded
same-PID exec now restores the validated private environment. Five real signed
startup tests pass, including LaunchServices, same-PID execution, the one-exec
bound, directory permissions and symlink escape refusal. Thirty-five existing
driver tests and strict example Clippy also pass. Independent source review
found no Important issue.

The desktop relocked before a corrected ZIP install could run. That case also
failed its deadline and cleaned up normally. No complete update/relaunch case
has passed yet, and no personal app was installed or replaced. See the
[local baseline record](t12-sparkle-preview-20261111.1-local-test.md) for the
actual native screenshots, exact build source and remaining gates.

macOS Foundation ignores the fixture's `TMPDIR` for its protected per-user temp
root. The fixture pins that exact root, owner and permissions; its own files
remain under private run directories. `cfprefsd` also stores the unique test
bundle's preference domain under the account's Preferences directory despite
the private HOME. This requires exact unique-domain ownership checks and scoped
cleanup; a private `NSHomeDirectory` alone does not prove preference isolation.
Public-API cleanup removes all values from the exact owned domains after their
processes exit; macOS may retain empty backing files, which the evidence records.
The two initial probe domains have private backups and now contain zero values.
No personal Spaceport preference domain or daemon is a fixture.

## Remaining delivery gates

- Complete native signed install/relaunch and negative updater cases, including
  wrong payload identity and downgrade behavior, with an unlocked desktop.
- Retain the reviewed fixture's exact results and complete its manual cases.
- Run full default/all-feature/platform CI on the exact integrated source.
- Tag a fresh version only after CI and native update checks pass. Verify the
  protected workflow, public DMG, public signed feed and download bytes.
- Update the download link and add the release-specific artifact evidence.

There is no new Sparkle-enabled public release or feed yet. The first enabled
build requires one manual installation. Other-platform updates, general release
ownership and complete native UX qualification remain separate open gates.
