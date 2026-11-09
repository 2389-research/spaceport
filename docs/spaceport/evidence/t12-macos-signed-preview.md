# macOS signed preview workflow

Decision D61 authorizes a macOS-only build/sign/notarize artifact route.
Doctor Biz selected `ai.2389.spaceport`; ordinary development builds retain
`spaceport.development`. Windows signing is deferred for this task. The
existing public release pipeline, updater enablement, publishing and app
installation remain outside authorization.

## Verified signing inputs

Read-only certificate inspection found `Developer ID Application: 2389
Research, Inc (HD9NM9NSMK)`, valid through 8 August 2030. The matching signing
identity is installed. The provided directory's Team metadata matches it;
the named API key/issuer metadata authenticated a `notarytool history` query.
History contents and private key material were not emitted. Credentials
were not copied into source, committed or uploaded to GitHub.

These observations establish available local signing inputs, not a signed
artifact or working hosted workflow. Private paths and key contents remain
machine-local. A separate company's installed certificate and another app's
update key were not selected or used.

## Bounded implementation contract

Reuse the existing macOS build, assembly and signing scripts. A narrow
explicit macOS preview mode supplies the version and preview identity;
ordinary development and the blocked `HERDR_RELEASE_VERSION` route remain
distinct. App and bundle identity have one authoritative source. Signing
does not change credential policy or install Herdr.

A manual macOS-only workflow builds/smoke-tests without signing credentials,
then signs known same-run artifacts using a protected signing environment.
It produces downloadable artifacts, not a public release or update feed.
Local signing can reuse the exact installed 2389 identity and an API key
file instead of importing new credentials. No GitHub secret provisioning,
workflow dispatch, repository push, app launch or installation is included.

Notarization submission must occur once per submitted artifact. Persist its
digest and returned UUID before waiting/polling. A lost response, timeout
or pending status is not permission to submit again. Receipts and useful
diagnostics survive cleanup; no signing key belongs in a package or report.

## Source and verification

Implementation commit `d41e07a42220b269914a2ee573cff87b8f1f9f95` adds the local
preview route and manual `.github/workflows/macos-preview.yml`. Independent
review checked credential separation, exact identity/Team, package inventory,
notary receipts, hosted version history and retained notices. It found an
inexact post-sign identifier comparison and unsupported P12/CI validation prose;
both were corrected. The identifier regression failed first when an ID suffix
reached submission, then passed with exact field comparisons. Fresh-eyes review
found no further issue. No live Apple tools ran in the reviewer.

Checks on macOS arm64 with Rust 1.99.0 and locked dependencies:

- App unit tests: default 3,870 passed / nine ignored; all features 3,899 passed /
  nine ignored. Build/preview identity integration tests: 11 passed in each mode.
- Release Python discovery: 271 cases, eight skipped, no failures before the
  final exact-signature regression; focused signer suite after that fix: 13 cases
  passed. These Apple-tool stand-ins prove control flow, not native signing.
- Release security: 24 cases, one skipped. Package assembly contracts: nine
  passed. Actual third-party notices generated cleanly with cargo-about 0.9.2;
  only two obsolete exceptions for dependencies absent from Cargo.lock were removed.
- `cargo +1.99.0 fmt --all --check`, Bash syntax, ShellCheck with the release
  source path, actionlint, `just check-file-size check-sorted`, `just
  check-commits`, and `git diff --check` passed.
- Strict app Clippy (`--all-targets --all-features --no-deps -- -D warnings`)
  remains failed on exactly the 253 inherited diagnostics: code, message, primary
  file and multiplicity match the prior baseline. One new constant assertion
  finding was fixed; the two affected constant tests passed. An initial invocation
  without `--no-deps` also stopped on the inherited Core client type-complexity
  finding. No lint suppression was added.

The inherited `block v0.1.6` future-compatibility notice remains. Full workspace
and cross-platform CI were not run here. The hosted environment and secrets have not been provisioned; hosted P12
import has not been tested. Local isolated P12 import succeeded as recorded below. Production, updater, native UI and
other-platform acceptance remain open.

## Actual artifact qualification

The first actual `20261109.1 debug arm64 installed` build compiled and passed
CLI smoke, generated notices and assembled the exact app. Its first helper
signature failed with `errSecInternalComponent`. Read-only Security framework
status reported the default keychain locked while the selected identity remained
listed as valid. No notarization receipt or submission existed. The failure was
not retried blindly.

The supplied P12's public certificate fingerprint matches the approved Developer
ID certificate. The explicit `20261109.1 debug arm64 import` build succeeded with
an isolated temporary keychain. The user keychain search list matches its exact
pre-run bytes afterward; Security framework status still reports the default
login keychain locked. Temporary signing directories/keychain were removed.
No personal keychain credential, certificate trust or access policy changed.

| Observation | Actual result |
| --- | --- |
| App submission | `4c0673c3-22e4-4c84-b768-2923a5cecd40`, `Accepted` |
| DMG submission | `dd489338-f28a-438b-a3af-e5abd5803614`, `Accepted` |
| Identity | `ai.2389.spaceport`, Team `HD9NM9NSMK`, version `20261109.1` |
| Build | Debug, arm64; both app and `sheprd-acp` are arm64 |
| DMG | `Spaceport-20261109.1-arm64-apple-darwin.dmg`, 54,177,146 bytes |
| SHA-256 | `cb48d905612ab616ca4bfac98bd1ccffaec981decd16a17eccfc7e3743ce3fd9` |
| Actual app and DMG | Strict codesign verification, stapled-ticket validation and Gatekeeper assessment passed; `source=Notarized Developer ID` |
| Shipped executable | Compiled preview identity check and canonical `smoke-cli.py` passed from the read-only mounted DMG |

The stable local delivery directory is
`target/test-builds/signed/20261109.1/`. It holds the DMG, metadata, structured
`qualification.json` and a copy of the signed app extracted from that DMG. The
copied app passed signature, ticket and Gatekeeper checks again. The assembler's
adjacent app in `target/distribution` is unsigned; it is not the delivered app.
Persistent receipts live in
`target/notarization/ai.2389.spaceport-20261109.1-arm64/{app,dmg}/` and retain
submission output, archive digests, UUIDs and final status. Each artifact had
one submit. No credential is included in the delivered app or receipts.

Two mistakes in the throwaway inspection were corrected before this result:
macOS returned the canonical mount path, which differed from the temporary-path
alias, and the app correctly rejected an unsupported `--version` probe with
exit 2. The corrected check canonicalizes paths and uses the actual compiled
identity record and existing CLI smoke script. The final inspection exited 0
and detached only its own image device. These harness mistakes were not product
failures. Native tools also emitted deprecation warnings for inherited `hdiutil`
create/detach commands; no warning was suppressed.

No GUI was launched, installed app replaced, personal runtime contacted, workflow
dispatched, secret uploaded, release published or updater enabled. The local
artifact is ready for Doctor Biz's native test; it does not prove workspace,
chat, input, notification, side-by-side install or update acceptance. Hosted
signing and Intel/Windows/Linux native qualification remain unrun. T12,
G-RELEASE and G-PLATFORM remain blocked/open as recorded in their registries.
