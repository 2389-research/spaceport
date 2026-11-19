# Windows CI restoration

Review child `jvcf`, under `9s44`. Source verification: 2026-11-17.

Doctor Biz authorized the review fixes, including restoring Windows CI coverage.
This work changes source only. It does not change GitHub variables or publish code.

## Cause and change

D66 paused Windows checks, native tests and packaging behind `WINDOWS_ENABLED`.
The aggregate separately inferred that state from the repository name.
Enabling the variable therefore scheduled jobs whose success the aggregate rejected.

The ordinary Windows jobs now run independently of the performance variable.
Formatting, clippy and workspace unit/CLI tests cover default and all features.
Both Windows runner architectures retain the existing native `live_gui` invocation.
It selects `native_sidebar` and `native_local_controls` on both runners.
It also selects `native_gui_live` on x64.
The existing ARM64 live-daemon exclusion remains because Herdr has no native ARM64 Windows build.
The package job retains its input selection, installer smoke test and pinned tools.

The aggregate requires successful Windows checks and native tests when code inputs change.
It requires the package result according to package inputs.
Skipped or failed active jobs cannot pass.
The preview source gate again requires both Windows check and native matrix jobs.

Only the release-mode performance step retains `vars.WINDOWS_ENABLED == 'true'`.
Its release harness and 100 ms p95 budget remain unchanged.
Kata `h8g5` remains a Windows performance qualification gap.
A green active-platform run does not establish performance acceptance.

Repository, actor, event, permission and credential boundaries remain intact.
No action pin, runner matrix or production release permission changed.

## Verification

- TDD baseline: six transition tests produced 12 failing assertions against the original workflow.
  They exposed disabled ordinary jobs, the actual aggregate environment mismatch,
  missing Windows source-gate names and the missing performance-only condition.
  Baseline output: `/tmp/spaceport-windows-ci-red.log`.
- Targeted tests passed: `test_ci_policy.py`, `test_check_ci.py` and
  `test_spaceport_release_transition.py` (32 tests total).
  Commands used `uv run --no-project python -m unittest discover -s scripts/release/tests -p FILE -v`.
- Tests evaluate real workflow conditions and resolve its own aggregate environment expressions.
  They execute the actual aggregate shell across push, pull-request and manual events.
  Cases cover code/package inputs, unset/false/true performance variables and invalid job results.
- The native command test executes the real workflow shell with a temporary argument-capturing Cargo substitute.
  It proves fixture selection and the ARM64 exclusion, not native GUI execution.
- `actionlint .github/workflows/ci.yml` passed.
- `zizmor --offline .github/workflows/ci.yml` reported no findings, with two existing ignores.
- `bash scripts/check-file-size.sh` passed.
- `uv run --no-project python scripts/release/test-release-security.py` passed (three tests).
- `scripts/check-spec.sh` passed after regenerating the D66 views.
  Its 22 tool tests and package check do not establish application or schema conformance.
- `git diff --check` passed.
- Root independently reviewed the actual workflow, release job contract, condition evaluator and aggregate tests.
  That review found no material issues.
- Full release-script discovery ran 381 tests, with 18 skips, four failures and one error.
  Command: `uv run --no-project python -m unittest discover -s scripts/release/tests -v`.
  Log: `/tmp/spaceport-windows-ci-suite.log`.
  Three untouched `test_sparkle_native_cache.py` tests reject an unexpected executable-census row.
  Two untouched `test_sparkle_native_regressions.py` cleanup tests lack `preferences-cleanup.json` after that refusal.
  Loading `test-sparkle-native.py` directly from unchanged `HEAD` reproduces the census error on this Mac.
  This source fix does not change those scripts or claim the full suite passed.

## Remaining qualification

Doctor Biz authorized the public draft checkpoint on November 18.
Hosted native GUI and installer jobs subsequently ran; see [CI qualification](review-ci-qualification.md).
Full workspace qualification remains pending the latest repairs.
Performance remains explicitly deferred to `h8g5`.

## Windows UI Automation

The x64 native job also selects the existing `accessibility_windows` integration target.
It runs every ignored case in that target without a test-name filter.
The step uses the job's pinned Herdr executable and the existing isolated sandbox.
It runs after the ordinary native GUI fixtures, with one test thread.
The ARM exclusion follows the existing unsupported live-daemon boundary.
Workflow triggers, credential boundaries, and the performance gate remain unchanged.

The test reads the real UI Automation tree and checks native keyboard focus moves.
It covers main-window chrome, terminal exposure, prefix region commands, Tab, and Escape.
It does not qualify screen-reader speech, Core transcripts, or the full Settings keyboard matrix.
Those acceptance checks remain separate.

The scheduling regression first fails because no accessibility step exists, then passes with the step.
Its argument recorder proves command selection, not native execution.
All 24 CI policy cases, six release-transition cases, and three release-security cases pass locally.
`actionlint` passes; offline `zizmor` reports no findings and retains two existing ignores.
Windows GNU Clippy passes the changed integration target with all features and warnings denied.
That check proves compilation, not Windows execution.
Independent review corrected a stale Version-button assertion to require the actual noninteractive accessible label.
The existing PowerShell capture has no per-call timeout; the job's 90-minute limit remains its final bound.
Actual hosted UI Automation execution remains pending.
