# Sparkle preview updates

Doctor Biz approved this design on 2026-11-10. Base: `6614c98a7`.

Goal: installed macOS previews can check for, verify, install and relaunch a
Spaceport update through Sparkle's native UI. The first enabled build needs a
manual installation. This does not complete other platforms' T12 gates.

Architecture: Sparkle 2.10.1, the MIT `sparkle-updater` Rust wrapper, and the
existing protected macOS preview workflow. Ship the pinned framework inside
the app; ordinary development builds do not link or start it. Retain the
controller on the AppKit main thread. Use the existing quit/detach lifecycle.

## Global constraints

- App identity remains `ai.2389.spaceport`; Team remains `HD9NM9NSMK`.
- Never update, install, bundle, stop or migrate Herdr or Herdr Core.
- Development identity `spaceport.development` cannot start update checks.
- Keep the inherited production release refusal and `RELEASE_BUILD` policy.
- New Ed25519 signing key stays outside Git and Cargo; protected CI only.
- Require signed appcasts and verify archives before extraction. Automatic
  checks require native consent; automatic installation is off by default.
- Use immutable version releases and the fixed preview feed asset at
  `https://github.com/harperreed/spaceport/releases/download/macos-preview-feed/appcast.xml`.
- Publish the signed feed only after the referenced release asset passes
  verification. Serialize preview publication; reject version regression.
- Keep exact CLI targeting, normal confirmation, and daemon ownership.
- Source files stay below 1,000 lines. Tests cover real behavior; native end
  to end tests use isolated signed fixtures and exact child cleanup.
- Run quick local checks; full workspace/platform checks and release builds
  run in CI. Record actual results, failures and deferred checks.

## Task 1: Framework packaging and signing

Add a checksum-pinned SDK acquisition script, framework validation/signing
helper and targeted Python tests under `scripts/release/`. Extend the existing
macOS packager and signer to copy the full framework with symlinks intact and
sign its known nested executables, services and framework inside out. Preserve
the two executable allowlist and all existing identity/notary safeguards.
Use SDK SHA256 `1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27`.
SDK input is `SPARKLE_FRAMEWORK_PATH`, the directory containing
`Sparkle.framework`. Public trust configuration is a checked-in JSON file,
`scripts/release/sparkle.json`, shared by packaging and feed publication.
First write failing tests for absent/tampered SDK, malformed framework and
bundle configuration; then implement and run the covering tests.

## Task 2: Native app integration

Add optional macOS-only `sparkle-updater = "=0.1.0"` in workspace dependencies,
member inheritance and a `macos-updater` feature. Add a small `updates` module
with typed errors, pure eligibility tests, and a retained GPUI global for the
safe wrapper. Start only in a matching preview app bundle, using the existing
quit lifecycle. Add a localized Check for Updates menu/command through the one
command catalog and CLI contract; unsupported development/platform routes
must report that honestly. Test eligibility, command routing and native UI.
No arbitrary feed URL setter or silent installer route is exposed.

## Task 3: Signed feed and CI

Create a separate Spaceport Ed25519 key with the official SDK tools and export
it into the existing private Apple signing directory with restrictive modes.
Put only the public key in `sparkle.json`; store the private key as protected
environment secret `SPARKLE_PRIVATE_KEY`. Add deterministic feed tests for
version, URLs, signatures, size and rollback refusal. Sign final stapled DMG
bytes and the XML using official `sign_update --ed-key-file -`. Publish a
fixed feed release asset after the immutable version release is verified.
Enable the feature/SDK only on relevant macOS jobs; no signing secrets in
build jobs. Keep full CI and existing source/tag/notarization gates.

## Task 4: Qualification and delivery

Record D65 and current updater scope in the authoritative task package and
release docs. Run real signed fixture updates and negative tamper, identity,
key and downgrade checks; prove GUI relaunch leaves isolated daemon children
alive. Review each task and the full branch. Commit reviewed work, push/merge
to main, wait for exact-source full CI, tag a fresh immutable preview version,
and verify the published DMG and signed feed. Record evidence and remaining
platform/native gates; do not describe unrun checks as passed.

## Progress

- Worktree created; official SDK archive downloaded and SHA256 verified.
- Task 1 complete at `6f0b7c5e3`; independent spec and quality review clean.
- Task 2 complete at `c5c299d1c` plus `d1255a42c`: rpath and compiled-worktree
  review findings fixed with regressions; independent re-review clean.
- Task 3 code and workflow integration pass targeted tests and independent review.
  Publication orchestration lacks focused branch tests; qualify bootstrap against
  real GitHub during delivery and retain the replacement-path test gap explicitly.
- Native fixture research records Sparkle key/identity rotation behavior. The
  exact app ID is a release publication gate, not an unconditional installer rule.
- No new preview tag, release, feed or native install/relaunch has run yet.
- Task 1 license follow-up `27f7157e1` reviewed clean: SDK root LICENSE is pinned,
  bundled and validated again before signing credentials.
- Task 4 native Spaceport daemon/terminal quit/restart test committed at
  `6c30c7dc3`; five tests pass, including real owned-child-only failure cleanup.
  Independent review clean. Native updater host compiles and offered signed v2.
- Thirty-five fixture driver tests and the native read-only feed/selection
  cases pass. Normal quit sentinel is verified. Manual install/relaunch and
  archive/identity/actual-payload rollback still need the unlocked desktop.
- Native updater install/relaunch is blocked by locked desktop. Doctor Biz said
  he cannot unlock it now. Continue review/CI; hold the public tag for native QA.
- macOS uses its own protected per-user Foundation temp root and unique-domain
  CFPreferences store. Fixtures pin temp ownership and isolate/clean only their
  own random preference domains; do not claim HOME redirects every Apple store.
- Whole-branch review closed four findings: stale Busy menu state, cleanup
  skipped by diagnostics, unrelated errors passing negative cases, and timed
  feed-signature recovery. Exact-zero expiry is required in packaging/runtime.
  A real aged-state red/green case confirms refusal after a 21-day failure record.
- Final review is clear for source commit/push and draft PR. Full platform CI
  remains the next step; no public tag until the remaining native checks pass.
- Draft PR #1 is open. CI run `42959221177` caught SDK deletion by Rust-cache
  and stale application-menu assertions. The SDK now lives under `RUNNER_TEMP`;
  its path regression and 13 real feed/signature tests pass locally. Updated
  menu tests cover About, Check for Updates, and the following separator.
  A fresh full CI run must verify these fixes across platforms.
