# Review fixes on current main

Scope: Kata `9s44`, all 53 review findings.
Status: locally verified checkpoint, authorized for a public draft PR.
On 2026-11-18, Doctor Biz answered “Commit and open draft PR.”
That approval covers committing and pushing the reviewed 906-file checkpoint for Clint and CI.
It does not authorize merging or releasing it.

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

## Verification

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

Prepare a concrete handoff for the overlapping work with Clint.
Reconcile PR 123's additional protocol copy with the local error guidance.
See [the bounded comparison](review-pr123-overlap.md).
Commit and publish the reviewed checkpoint under Doctor Biz's recorded approval.
Then reconcile the newer main delta and rerun its affected checks before merging.
Preserve the tested checkpoint while resolving its lifecycle and notice overlaps.
The exact overlap list is `/tmp/spaceport-9s44-handoff-new-main-overlaps.txt`.
An isolated preflight found two textual conflicts among those 24 paths.
They affect `sheprd-client/src/forward.rs` and the Devices setup implementation.
Its inputs and proposed merges remain under `/tmp/spaceport-9s44-new-main-preflight`.
No preflight result has changed this verified checkpoint.
The full workspace test suite, hosted platform jobs, application build and CLI smoke remain deferred to CI.

Kata remains 31 closed children and 22 open children at this checkpoint.
The umbrella remains open.
Native Settings readiness is pending Doctor Biz's answer.
Native Windows, native accessibility, and interactive release profiling remain pending.
Offscreen Metal pixel checks qualify rendering only.
