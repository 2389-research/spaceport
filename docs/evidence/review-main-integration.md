# Review fixes on current main

Scope: Kata `9s44`, all 53 review findings.
Status: `46738c99c` passes CI; the later consent run exposes a Windows UI Automation timeout.
On 2026-11-18, Doctor Biz answered “Commit and open draft PR.”
That approval covers committing and pushing the reviewed 906-file checkpoint for Clint and CI.
It does not authorize merging the draft into main or releasing it.

Published checkpoint: `150eed475c4097c63dff86edf8e2b215c64a1d02`.
Draft: [PR 125](https://github.com/2389-research/spaceport/pull/125).
The remote branch and draft head were read back and match that commit.
The next verified draft head is `e8acf1aa6262f91d3610d5b97b5bb5a928b0ffa7`.
It includes main `099e380ff` and the combined-source checks recorded below.
Before this update, the published head was `6d89f5e42c0478aa865df329720f4a017e3764f5`.
Its first complete CI run failed; see [the qualification repairs](review-ci-qualification.md).
Main subsequently advanced to `57d6ea140722a613ce17beb9ec6d942e5d4c504f`.
The topic branch incorporates that main alongside the CI repairs.
See [the newer integration evidence](review-jargon-main-integration.md).
Main then advanced to `44c5325e8b902f70d90f26d19d9188ae87c49d78` through PR 121.
See [the reorder integration evidence](review-reorder-integration.md) for the current combined candidate.

## Sound extraction integration

Main advanced again to `4feee1830b1daffa2ea42792478cddd042d348da` during validation.
PR 124 moves sound policy and playback into `sheprd-sound`, with lenient decoding in `sheprd-lenient`.
The draft branch incorporates that 63-path delta.
It does not change main itself.

Resolved two test conflicts while preserving the fork's guards.
Connection tests retain both the extracted sound cases and worker-panic cases.
The Settings sound-volume test keeps its private fixture and uses the extracted recording service.
Review found five retained service tests missing after extraction.
Restored their queue, preview, cancellation, worker-error, and expiry assertions in the sound crate.
Replaced the reload test's polling sleep with a controlled loader completion channel.
Independent review confirmed both corrections preserve the original contracts.

The broader localization test exposed a pre-existing clipboard-busy wording mismatch.
Aligned Rust Display with its existing English catalog, without changing the typed error or visible localized text.
The equality regression failed first, then all 53 localization cases passed.

The sound integration passes 643 distinct focused GPUI cases, with one opt-in case ignored.
Its scopes cover connection, endpoint, state, volume, localization, app settings, and Herdr settings.
The sound crate passes 34 tests, including the five restored cases.
The lenient crate passes its one test.
Both offscreen Metal pixel cases pass again.
Canonical host and Windows GNU lint pass both feature configurations again.
Formatting, size, sorting, operations, localization, and specification validators pass.
The specification gate still explicitly skips JSON Schema validation.
The earlier client and Core tests qualify their unchanged source components.

Logs use `/tmp/spaceport-9s44-sound-integration-*.log` and `sound-restored-tests.log`.
The focused test summary is `/tmp/spaceport-9s44-sound-integration-test-summary.json`.
Native sound-device behavior, desktop interaction, release timing, and hosted platform checks remain unqualified.

Requested optional landing-order coordination while moving conflicts prevented CI from starting.
Withdrew that request once the draft became conflict-free and CI started.
The workflow permits manual dispatch only on main; this work does not bypass that policy.

## Newer-main integration

The topic branch is incorporating main `099e380ff90d72a63c32dc19868ae89c81c32c74`.
This updates the draft branch; it does not merge the draft into main.
Resolved the two textual conflicts in the forward and Devices setup code.
The combined tree passes its focused tests, final lint, and pixel checks.

Devices setup keeps its `CliChild` owner and one deadline for process exit and both captured outputs.
It uses main's shared `wait::exit` helper with the owned child's inner handle.

The forward retains `OwnedChild`, null stdout, and the 250-millisecond master-state poll.
Independent lifecycle review approved this bounded exception to main's EOF optimization.
The detached EOF reader can remain blocked while an inherited writer stays open.
Reusing the existing capture reader would add idle polls every 25 milliseconds.
A new event-waker API would expand this integration beyond its ownership fix.
The forward keeps four monitor wakeups per second and up to 250 milliseconds of exit-detection delay.
This resolution makes no forward performance-improvement claim.

The remaining shared waits, process watch, metadata wakes, and render-time notice changes remain included.
Independent review found two missed notice API adaptations in the combined tree.
Updated the status-text test caller and three production warning callers to use explicit timestamps.
The test compile first failed with E0061, then passed with its caller corrected.
Production warning callers use `is_some_at`; all-feature tests alone could hide their missing-method regression.
Independent review approved the corrected callers and found no further material lifecycle issue.

The combined source passes 416 client tests and 78 Core tests.
Four client and fourteen Core opt-in tests remain ignored.
Sixteen focused GPUI scopes pass 466 distinct tests, with three opt-in tests ignored.
The count removes duplicate test names across overlapping scope filters.
No test failed in these final focused runs.
Logs: `/tmp/spaceport-9s44-new-main-client-core.log` and `new-main-verified-SCOPE.log`.
The scope summary is `/tmp/spaceport-9s44-new-main-test-results.json`.
Static validators and the specification gate also pass against the combined source.
Canonical `just lint` and `just lint-windows` pass both feature configurations again.
The no-default-feature checks include the production app target and qualify the corrected notice callers.
Both opt-in offscreen Metal pixel cases also pass against the combined tree.
Logs: `/tmp/spaceport-9s44-new-main-{host-lint,windows-lint,metal-pixels}.log`.
These current results supersede the older checkpoint-only qualification for the combined source.
The existing debug-test linker and dependency future-compatibility notices persist.

## Source and destination

The tested source remains `.worktrees/review-squad-fixes` on `fix/review-squad-findings`.
Its base is `723be121f3d6850968865d1735569fb7b0514892`.
Its fixes are uncommitted.

The integration checkout is `.worktrees/review-squad-integration` on `fix/review-squad-integration`.
Its pinned main base is `91bda83a77e7e02bbc6d9ba02887cb95dc3876b9`.
Main contains 113 commits since the source base.
The candidate overlaps main in 35 files.

At handoff, remote main advanced to `099e380ff90d72a63c32dc19868ae89c81c32c74`.
PRs 119 and 122 merged after the candidate's pinned base.
Their delta changes 24 paths, including 11 candidate overlaps.
This verified checkpoint does not include that later delta.
Do not claim its checks qualify the newer combined tree.

Success requires preserving main's changes and the reviewed fixes together.
Run focused checks against the combined source.
Keep native and platform qualification explicit.
Keep the original checkout available for comparison.

## Transfer and conflict decisions

Applied the tracked binary patch with `git apply --3way --index`.
Copied 389 new files without overwriting existing paths.
Excluded two private operational notes from the public candidate.
The original notes remain in the source checkout.

Resolved 11 conflicted files.
Kept main's reader wakeups with the extracted connection reader.
Wake after releasing the inbox guard and after publishing reader completion.
Kept shared Unix and Windows lifecycle transports.
Unix accepts coordinate directly with their worker.
Windows accepts retain bounded nonblocking waits.

Preserved main's D71 Core graduation decision.
Assigned the workspace-close decision D72 and updated its evidence reference.
Regenerated the specification views from the registry.
Aligned D70's retry wording with the authorized Kata finding: require explicit Try again.

Combined the CLI help corrections with precise targeting and creation receipts.
Retained main's structured gotchas and added focused review lessons separately.
Updated its Windows CI lesson to match the restored jobs.

Independent review found the new Settings recipe missing from main's operations guide.
Added its guide entry; the self-enforcing recipe check now passes.
That review found no other material regression in the automatic merges.
The resolved shared contracts pass independent review.
Focused combined-source tests remain in progress.

## Test coordination correction

The first combined compile found a Unix socket-pair test under the expanded Windows parent.
Kept that upstream test Unix-only and removed its redundant Unix type qualification.

The endpoint scope then deadlocked in its contended-shutdown regression.
That test holds the inbox while waiting for an independent worker stop flag.
The inherited helper waited only for reader notifications, which require that same inbox.
Stopped only the verified owned test child and reaped its Cargo parent.
This run failed; it is not a passing endpoint result.

Kept notification wakes and added a 250-millisecond recheck in test builds only.
All four waiting helpers also enforce a 30-second failure deadline.
They still verify the actual predicate or reader count after every wake.
The existing event-count guard remains unchanged.
This deliberately qualifies main's clock-free test default for independent worker state without notifications.
It changes no production timing or connection signaling.

The exact deadlocked regression passed after the timed recheck: one test, zero failures.
Independent review approved the final deadline and notification ordering.
The final combined endpoint scope passes all 177 tests, including that regression.

The first completed endpoint run also exposed two stale expectations.
Clipboard acquisition now refuses its shared read budget before the older upload guard.
Its test retains the four-task bound and checks the typed acquisition error's localized message.
The flash cleanup test now advances to its actual expiry instead of assuming five seconds.
It still proves a fading flash disappears completely on endpoint reset.

A subsequent endpoint run caught another incomplete fixture observation.
The remote shutdown test saw a retired handle before the final ordered state was projected.
It now waits for reader completion and a drained inbox before asserting the shutdown result.
It retains the exact stopped-status and no-reconnect assertions.
The contended-reader case separately rejects retries before that projection completes.
All three corrections affect test code only.
Independent review approved them against the production callers.

## Published checkpoint verification

Passed `just format-check check-file-size check-sorted check-just-operations check-i18n check-spec`.
Passed `git diff --check` and `git diff --cached --check`.
Passed `just check-spec`: 34 tool tests and package integrity checks.
The canonical specification gate skips JSON Schema validation explicitly.

Cleaned the six changed workspace members under the shared Cargo build lock.
Kept the external dependency cache.
Used Rust 1.99.0, locked dependencies, and disabled incremental compilation.
The combined source passes 311 focused GPUI tests across these disjoint scopes:

| Scope | Passed |
| --- | ---: |
| `connection::tests` | 28 |
| `endpoint::lifecycle_tests` | 177 |
| `control::client::tests` | 23 |
| `control::server::chat` | 9 |
| `chat::tests::glide` | 8 |
| `chat::tests::recall` | 3 |
| `multiline_input::tests` | 19 |
| `state::tests::daemon_messages` | 5 |
| `window::regions::tests` | 20 |
| `smooth_scroll::tests` | 19 |

Command shape: `cargo test --locked -p sheprd-gpui --bin spaceport SCOPE -- --test-threads=1`.
All listed scopes report zero failures and zero ignored tests.
Connection results use `integration-final-connection::tests.log`.
Other scopes use `integration-verified-SCOPE.log`.

Passed canonical `just lint` in both feature configurations.
Passed canonical `just lint-windows` in both Windows GNU feature configurations.
Both commands check workspace members and all targets with warnings denied.
Windows cross-compilation does not qualify native Windows behavior.

Passed both opt-in offscreen Metal pixel cases against the combined source:
`cargo test --locked -p sheprd-gpui --features integration-test --bin spaceport metal_pixels -- --ignored --test-threads=1`.
They qualify thin image fragments and transparent fragment boundaries.
They do not qualify native keyboard input, glyphs, or interactive performance.

Two existing build notices persist from the original fixes checkout.
The large debug test executable exceeds the linker's compact-unwind section limit.
Cargo also reports future compatibility concerns in dependency `block` 0.1.6.
The retained logs show both notices before this integration.
Neither canonical Clippy command reports a new source warning.

The final privacy check inspected 906 changed paths.
The candidate excludes both private operational notes.
The focused credential and private-collaboration pattern scan found no matches.
The prior read-only privacy review inspected all 28 unchanged screenshots individually.
Its evidence remains applicable because the transfer preserved their bytes.
This check is not a claim that automated patterns can detect every secret.

Logs use `/tmp/spaceport-9s44-integration-*.log`.
Earlier evidence documents describe the original fixes checkout.
Those earlier results do not prove this combined checkout passed.

## Remaining work

Source `5f788861a` completed hosted CI with fourteen passing checks, eight failures, and one skipped target.
Both Windows native jobs, both macOS release builds, the installer, and three Linux workspace jobs passed.
The remaining failures exposed shortcut checkout conversion, accepted-batch contention, and isolated peer-state leakage.
The repairs pass 235 focused checks and both canonical lint configurations locally.
See [the CI qualification record](review-ci-qualification.md#close-batch-contention) for exact limits and remaining hosted qualification.

Published source `46738c99c` passes all required hosted platform checks on draft PR 125.
Keep the concrete PR handoff and dashboard current for Clint and CI.
PR 123's additional protocol copy is reconciled with the local safety guidance.
See [the completed reconciliation](review-jargon-main-integration.md).
Keep the draft unmerged while its hosted and native gates remain open.
CI qualifies the published workspace suites, platform jobs, application build, and CLI smoke.
The consent follow-up fails its first Windows UI Automation run.
Bounded partial capture and stage markers will locate the stall without weakening the deadline.
An independent audit also finds reduced-motion gaps in extracted slides and opted-in chat glide.
Their three regressions fail first; the corrected focused scopes pass 70 tests.
The normal-consumer test retains 150-millisecond slide timing.
See [OS accessibility evidence](review-os-accessibility.md#reorder-extraction-follow-up).

Kata now has 37 closed children and 16 open children.
Reduced motion closes after its extracted-slide and chat-glide corrections pass scoped verification.
The umbrella remains open.
Native Settings readiness is pending Doctor Biz's answer.
Both Windows native GUI jobs and the x64 accessibility tree/focus case pass on `46738c99c`.
Windows workspace path resolution and mismatched-owner rejection also pass on that source.
The full accessibility matrix and interactive release profiling remain pending.
Offscreen Metal pixel checks qualify rendering only.
