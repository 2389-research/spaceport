# Branding audit follow-ups

## Scope and authority

Doctor Biz requested fixes for the active findings in the November 12 branding audit.
The audit source is `bd1782f9c6507abe5436983aa31a9be91f2b316f`.
The worktree is `.worktrees/branding-audit` on `fix/branding-audit-followups`.

Doctor Biz separately approved enabling GitHub private vulnerability reporting for `2389-research/spaceport`.
He approved using Spaceport's private reporting form in the issue chooser and security policy.
This approval does not authorize changing repository access or submitting a vulnerability report.

Fix BR-01 through BR-09 from the audit.
Preserve legal credits, protocol names, helper executable names and existing personal settings.
Keep Windows CI paused and the inherited production release route blocked.
The existing cross-platform installer/signing qualification group remains outside this bounded fix.
Canonical broad task status remains in `plan/tasks.json`; this note records execution and evidence only.

## Success criteria

- Real GUI-namespace events reach the diagnostics collector.
- The remote helper script finds current user and system Spaceport bundle locations.
- Existing supported helper locations and dependency logging still work.
- New config files and active instructions use the Spaceport CLI and state paths.
- The Nix launcher, desktop metadata and installed icon agree with the app's development identity.
- The issue chooser and policy use the approved Spaceport reporting recipient.
- A fresh GitHub read confirms private reporting is enabled and the description matches retained scope.
- Font sample and artwork guidance use current branding.
- Existing settings bytes, runtime ownership and release guards remain intact.

## Execution checkpoints

1. Add failing regressions for the GUI log root, real native diagnostic capture, remote bundle discovery and generated guidance.
2. Apply the smallest code/template changes and run the targeted checks.
3. Correct active docs, Nix/desktop metadata and approved reporting guidance.
4. Verify the approved GitHub reporting setting and repository description with fresh reads.
5. Run quick canonical checks and one independent review of the complete diff.
6. Record exact results and platform limits, then deliver the reviewable branch through the established Kata/PR workflow.

## Verification approach

Use Rust 1.99.0, `--locked` and the existing compiler cache with incremental compilation disabled.
Use private filesystem and native GUI fixtures; never use a personal daemon or installation as a test fixture.
Keep source-level, headless, native and platform qualification evidence distinct.
Use existing packaging/desktop validators and CI for platform-specific checks unavailable on this Mac.
Native window grouping on Linux remains a concrete manual check unless an actual Linux desktop run verifies it.

## Current state

The BR-01 through BR-09 code, template, metadata and documentation corrections are implemented.
The audit report pins source citations to the inspected commit.
Red probes and detailed check logs remain in the ignored `target/branding-fixes/` directory.
Native before/after captures are preserved in `evidence/branding-followups/`.

GitHub repository ID `1449575010` remains public at `2389-research/spaceport`.
Private reporting was enabled with HTTP 204 and freshly read back as `enabled: true`.
The enable request ID is `D6A3:24EB5F:3C12E17:BC07B98:6AF6A3AC`.
The repository description was updated and freshly verified:
“Spaceport is a native workspace for terminals and AI agents, powered by Herdr and Herdr Core.”
No report was submitted, and repository access was not changed.

Independent review found a missing macOS gate on the Fonts capture helper call.
The call now carries the same platform gate as its definition.
The stale desktop-identity comment was also corrected.
The reviewed platform gate and comment fixes are applied.
Changes are ready for review on the task branch; hosted checks and integration remain pending.

## Verification results

Rust commands used `RUSTUP_TOOLCHAIN=1.99.0`, `CARGO_INCREMENTAL=0`, `HERDR_BUILD_PR_NUMBER=''` and the shared display-copy target.
All Cargo commands used `--locked`.
Run the following test filters with `cargo test --locked -p sheprd-gpui --features integration-test --bin spaceport FILTER`:

| Filter | Result |
| --- | --- |
| `config::tests` | 268 passed; one existing opt-in test ignored |
| `discovery` | 23 passed, including current and older helper locations |
| `constants::tests` | Two passed |
| `settings_window::controls::fonts::tests` | Eight passed, including narrow layout and composition |
| `i18n::tests` | 11 passed, including pseudo-locales |
| `log_window::tests` | 15 passed |
| `log_targets::tests` | One passed |

`cargo test --locked -p sheprd-client settings_files` passed six tests.
`cargo clippy --locked -p sheprd-gpui -p sheprd-client --all-targets --features integration-test -- -D warnings` passed.
The debug test linker still reports the existing compact-unwind size warning.
Cargo also reports the existing future incompatibility in `block 0.1.6`.

`cargo test --locked -p sheprd-gpui --features integration-test --test live_gui native_local_controls -- --ignored --exact` passed before and after the Fonts text change.
Both runs exercised light and dark native fixtures, real startup logs and popup focus checks.
The new diagnostic assertion failed against the old filter before the fix.
Config guidance, helper discovery and Linux archive regressions also failed before their fixes.

`uv run --no-project python -m unittest discover -s scripts/release/tests -p test_release.py -v` passed nine tests.
The Linux distro, icon and smoke modules ran 23 tests, with one real-nfpm test skipped.
`uv run --no-project python scripts/release/test-release-security.py` ran 24 tests, with one skip.
`actionlint .github/workflows/nix.yml` passed.
`zizmor .github/workflows/nix.yml` reported no findings in its default offline mode.

`just format-check check-file-size check-sorted check-i18n` passed.
The specification check passed with JSON Schema validation and 272 local links.
All 48 local Markdown links in the changed documentation resolve, including heading targets.
YAML parsing and `git diff --check` passed.
Review and a focused credential scan found no credentials in the intended text files.
The captures show synthetic Settings windows only.

## Existing release-suite failures

The full Python release suite ran 417 tests: four failures, one error and 19 skips.
All five failures reproduce using untouched release scripts and tests from `bd1782f9`.
The baseline was extracted with `git archive`, without editing another worktree.
These failures are outside the changed Linux archive and metadata paths.

Three failures occur in `test_sparkle_native_cache.CacheProcessTests`.
The process census rejects an unexpected `ps` row before cache cleanup.
Two occur in `test_sparkle_native_regressions.FailureCleanupTests` because preference cleanup evidence is absent.
Keep these failures visible; do not weaken cleanup refusal to make the tests pass.
Investigate the process census parser against actual macOS output in a separate bounded fix.
Detailed results are in `target/branding-fixes/release-suite.log` and `baseline-sparkle-failures.log`.

## Remaining qualification

Actual Nix builds and Linux desktop grouping remain unverified locally.
This Mac lacks Nix and `desktop-file-validate`.
Nix validates the installed desktop file when a real Nix build runs.
The existing Nix workflow authorization gate remains unchanged.
Windows CI remains paused under D66.
