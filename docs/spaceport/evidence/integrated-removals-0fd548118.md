# Integrated browser/resource removals and retained UI checks

Application verification snapshot: `0fd548118`. Herdr 1 remains allowed under D58. D59 adds explicit Herdr and Sheprd team credits in About without changing the existing copyright/license attribution.

## Product changes

- `7af7d2249` removes the native embedded browser engine, preview assets, page-only control schema and webview dependency graph. Shared native tabs/groups, H1/Core terminals, Devices, rich chat rendering and explicit system URL/file handlers remain. The same change fixes bounded tab reveal during growth and preserves pending Core connection views.
- `dd40a96f0` distinguishes unknown agents from idle. `0e14bf17d` removes host CPU/memory/GPU/disk/uptime sampling and dashboards, their settings and the `system` command, while retaining Devices activity/history, agent navigation, ports, process inspection and worker cancellation.
- Development bundles use `Spaceport.app`; About uses the app identity and credits both upstream teams. Protocol 35 records both the browser and resource-control schema removals.

## Verification actually run

- `cargo +1.99.0 test -p sheprd-gpui --all-features --bin spaceport --locked --offline`: **4,005 passed, zero failed, nine existing opt-in tests ignored**. The first combined run had ten failures. Corrections preserve exhaustive settings/error checks, replace the retired blank-browser fixture with a real headless port-menu click, and keep strict pseudo-locale assertions using independent text-cache fixtures. A subsequent run found one final old Sheprd clipboard-restart expectation; it now uses the shared app identity and the full rerun passed.
- `cargo +1.99.0 test -p sheprd-control-protocol --all-features --locked --offline`: **65 passed**. Removed requests/replies are rejected; exact retained targets round-trip. A redundant constant-version assertion introduced one Clippy diagnostic and was removed while keeping behavioral wire tests.
- All-target/all-feature `sheprd-gpui` check passed. Format, file-size, sorted-region, i18n and diff checks passed. Release Python discovery passed **250 tests with eight skips**. Package validation and all **21** tool tests passed.
- Strict Clippy remains **failed**. The structured comparison against pre-cut `4625f9f23` is retained in `integrated-clippy-comparison.json`. Counts include both production and test compilations and are diagnostics, not distinct bugs. This is an unresolved quality gate; no lint allowance or hook bypass was used.
- The macOS debug test link still emits the inherited compact-unwind `__eh_frame` warning; Cargo reports dependency `block 0.1.6` future incompatibility. No new application source warning appeared in the all-feature check.

No native GUI, Windows/Linux execution, signed release package, performance budget, live runtime or real forwarded-service scenario ran for this snapshot. The earlier user-reported remote smoke success applies to the separate `30996ebbf` development artifact. It is not evidence for these later UI changes. T01/T04/T08 acceptance and backend/platform/release gates remain open as recorded in their registries.

## Checkout cleanup

Doctor Biz authorized removal of inactive sibling worktrees and requested future worktrees under the main checkout's ignored `.worktrees/` directory. The cleanup removed 161 clean inactive worktrees, preserved every branch tip and the original user-test artifact, and kept four checkouts with local PACT state. Six sibling checkouts remain, plus the separate external Codex checkout. Both audits are preserved in the main repository's `.git/maintenance/` directory. Reported allocated file sizes include APFS shared extents and are not a claim of equivalent free-space gain. No branch, commit, unique ledger object, running app or personal runtime was deleted.

PACT writes remain paused under the recorded [local reference incident](pact-local-reference-incident.md). Kata and Chronicle carry this run's progress. The next independent deletion unit is the remaining provider-polling/sign-in implementation, preserving verified reported usage and host/port primitives; it does not require another runtime migration.

## Development candidate

`ee9f5861e` was built with `RUSTUP_TOOLCHAIN=1.99.0 CARGO_INCREMENTAL=0 just bundle debug`. The canonical `Spaceport.app` was copied to `target/test-builds/ee9f5861e/Spaceport.app` and archived as `Spaceport-macos-arm64-ee9f5861e.zip` (48.3 MiB). Its adjacent `build.json` records exact executable, helper and archive hashes. `codesign --verify --deep --strict`, plist identity/companion inventory, real bundled CLI smoke and ZIP integrity checks passed. Only `spaceport` and `sheprd-acp` are bundled. `otool -L` lists no WebKit or JavaScriptCore framework. This is a macOS arm64 debug candidate with an ad hoc signature, not a notarized or release-qualified package.

An additional removed-family check first incorrectly assumed JSON output for `system --json`. Unknown top-level families use the normal launch parser's text usage error; the corrected check verified exit 2, empty stdout and `Unknown option: system` on stderr. No app startup or runtime connection occurred. The earlier `30996ebbf` test artifact remains intact; the candidate was not launched or installed. Native UI and manual remote regression remain unverified for this artifact.
