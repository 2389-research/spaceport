# Inherited custom updater removal

Decision D60 authorizes deleting the custom Rust updater after Doctor Biz
identified it as the inherited implementation. Base: `d74e1d33e`; code commit:
`f8c9c912c907f432c918add9e97eafa3128599d0` (reviewed tree
`8020a744a45867653a63b4c0222c57374a9c97ea`). Work used the isolated
`.worktrees/custom-updater-removal` checkout.

This is a replacement transition. SP-023 still requires Spaceport updates
across supported desktop and installation routes. Sparkle was discussed for
macOS and WinSparkle as a Windows candidate; no replacement implementation,
Homebrew ownership policy, production signing key/feed or publication was
approved by the deletion request. T12 remains blocked on G-RELEASE and
G-PLATFORM. A43 and A44 remain unqualified.

## Removed and retained

The custom updater crate, worker, download/staging/install/restart helpers,
three custom manifest feeds and update archives, their signing inputs and
publication paths are deleted. App update panels/previews/actions, settings,
CLI family, control operations/replies, helper dispatch, engineering harness,
updater-only CI tasks, tests, strings, docs and screenshots are removed.
There is no replacement updater state machine or disabled engine left behind.

`crates/sheprd-gpui/src/constants.rs` now owns the existing app version and
the exact release-build assertion previously held by `sheprd-updater`.
Ordinary builds still report `dev`; any `HERDR_RELEASE_VERSION` still fails
with `Spaceport release identity is not configured`. A scratch compiler test
uses the actual constants and proves development execution plus release
refusal. Its unrelated credential boolean fixture does not test credential
policy. No broad compiler warning suppression is used.

The private control schema advances from 35 to 36. Retired update operations,
CLI commands and private apply/install/relaunch helper flags reject before
GUI startup. `[updates]` table, inline, dotted and layered config forms now
produce a typed removal error; no preference or user file is rewritten.

General package building, platform signing/notarization, artifact integrity,
normal app/runtime lifecycle and Windows OS/installer Restart Manager are
retained contracts. They are distinct from the custom update installer.
Shared release-notes rendering still serves chat/daemon content. The small
progress enum still serves Windows file-transfer taskbar progress; its
unused update-panel renderer is removed. Generic installer reinstall checks
still preserve startup choices and verify that no GUI starts automatically.

The release artifact contract retains packages/SBOM, strict checksums,
signatures/attestations and package aliases. It no longer claims to verify
complete old upstream release sets containing retired updater feeds/assets.
Current install/site instructions say that Spaceport has no qualified update
replacement. Historical attribution remains.

## Verification actually run

- New tests first failed on the existing crate/dependency, accepted update
  commands/config and wire operations. The release absence guard also failed
  before scripts/feed removal. Green source/version integration tests: 2;
  CLI/config unit regressions: 2; protocol tests: 66; real executable CLI
  tests: 21, including exit 2, empty stdout and empty private state for retired
  updater helpers.
- `CARGO_INCREMENTAL=0 cargo +1.99.0 test -p sheprd-gpui --all-features --locked --offline`:
  final headless app suite **3,899 passed, zero failed, nine opt-in ignored**.
  Integration groups added 37 passed and eight ignored: **3,936 passed and
  17 ignored in this command**. Those ignored/native/platform tests are not
  qualified by the active cases.
- The first full app run had two failures, both from a stale beta-update
  Settings search row. Removing only that catalog entry made all nine focused
  search tests pass, then the full rerun passed. The exhaustive assertions
  remain intact.
- Default and all-feature all-target app checks passed. Format, file-size,
  sorted-region and i18n gates passed (20 i18n harness tests; 2,222 messages in
  114 files valid/used). Protocol negatives were moved to their topic file and
  the complete protocol suite passed again.
- `python3 -m unittest discover -s scripts/release/tests`: **256 tests, eight
  skips, passed**. `python3 scripts/release/test-release-security.py`: **24
  tests, one real-SBOM opt-in skip, passed**. The security suite's obsolete
  assertion for an already-deleted mockup tool was replaced with the actual
  retained release-security CI invocation. General packaging/security tests
  remain; deleted manifest tests covered only the retired engine.
- Focused site tests: 31 passed. Actionlint passed. Zizmor exited zero with
  no findings and retained its existing offline/shell-audit warnings and
  repository suppressions; no suppression was added by this change.
- Strict app Clippy is still **failed**: exactly the 253 inherited diagnostics
  and no new groups. An initial run found four new test diagnostics from
  `.err().expect()`; direct typed-error assertions removed them without an
  allowance. Strict protocol Clippy also remains failed on three existing
  constant assertions in unchanged test files. No lint gate is called green.

Detailed local reports/logs use `/tmp/spaceport-custom-updater-*` and
`/tmp/spaceport-updater-*`. The retained
[structured comparison](custom-updater-comparison.json) records the lock and
final app-lint observations. Twelve package/version records leave the lock,
including `sheprd-updater`; none are added or upgraded. The inherited macOS
debug-test linker unwind-size warning and `block 0.1.6` future-compatibility
notice remain.

Independent combined source/release review approved specification compliance
and task quality. It verified release refusal, exact helper/control removal,
normal primary/secondary/quit paths, shared consumers and retained package
security checks. Its two minor findings (a stale comment and repeated
identical `cfg(test)` attributes) were corrected and re-reviewed. No unresolved
source finding remains for this bounded deletion.

## Unresolved gates

Replacement update mechanisms, package ownership policy, production app IDs,
signing/publisher identity, release hosting, icons and actual native/signed
update qualification remain open. An inherited macOS release assembly still
expects `binaries/*/sheprd` while current builds/smoke use `spaceport`; this
separate packaging mismatch must be fixed before release enablement. The
preserved compile-time guard prevents an accidental production release.

No native desktop/screenshots, Windows/Linux execution, real installer/OS
restart, actual Herdr process-survival or signed update scenario ran here.
No personal installation, Herdr runtime, package manager, signing credential,
PACT object or published artifact was a test fixture. The release and product
acceptance statuses remain blocked/unqualified rather than claiming an
update-capable product.

## Development candidate

`RUSTUP_TOOLCHAIN=1.99.0 CARGO_INCREMENTAL=0 just bundle debug` passed on
`f8c9c912c`. The candidate is
`target/test-builds/f8c9c912c/Spaceport.app`; adjacent `build.json` records
the binary/helper/archive hashes and control protocol 36. The ZIP is
`target/test-builds/Spaceport-macos-arm64-f8c9c912c.zip` (45,526,811 bytes).
Earlier candidates remain intact.

The copied bundle passed `codesign --verify --deep --strict`, exact development
identity/companion inventory, `scripts/release/smoke-cli.py`, rejection of
`update` and the four retired helper flags before startup, linked-framework
inventory and `unzip -t`. Only `spaceport` and `sheprd-acp` are bundled;
there is no runtime or replacement update framework. This macOS arm64 debug
candidate is ad hoc signed and not notarized. It was not launched, installed,
published or used to qualify a native remote workflow.

The code was integrated into `wip/spaceport-t00`, not main. Integrated code
matches the reviewed task snapshot. Final integrated format, file-size,
sorted-region, i18n and diff checks passed. Package validation, all 21 tool
tests and the bounded T12 context packet passed; these establish package
integrity, not native application or update qualification.
