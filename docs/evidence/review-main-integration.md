# Review fixes on current main

Scope: Kata `9s44`, all 53 review findings.
Status: the current candidate integrates the reviewed Linux and Windows native fixtures and exact GPUI 0.3.11 pins.
Linux source `16a928826` passes all forty-six native cases, including Cycle Windows and cleanup.
Windows source `7e2459d67` passes twenty-eight native cases but fails console preparation and Palette query setup.
Shared candidate `3a603345a` repairs native preparation and SearchInput's assistive SetValue contract.
Its Linux and Windows probes both pass on the same GPUI 0.3.11 source.
Configured CI on evidence head `1feda67f7` remains pending.
The shared framework update still needs configured platform CI on the combined source.
Prior application source `d747cdbdb` passes all eighteen configured CI jobs and its x64 native Core cases.
Those 0.3.10 results do not qualify the new framework version.
The earlier ordinary Core failure remains unexplained.
Four measured native tab contrast cases remain qualified on `b94978e3b`.
Settings glyphs and clipping are qualified on `7b7498762`; other native accessibility gates remain open.
Crew tracks 50 done children, two doing, and one in review for `9s44`.
The closed shortcut child still has an explicitly recorded native qualification gap.
Kata separately records 49 closed children and four open children.
Its shortcut finding `3wd4` remains open for the required native qualification.
The painting/polling finding `gfde` closes in Kata at 04:09 UTC on November 20, under event 9804.
The earlier Crew closure alone had not closed that Kata issue.
On 2026-11-18, Doctor Biz answered “Commit and open draft PR.”
That approval covers committing and pushing the reviewed 906-file checkpoint for Clint and CI.
It does not authorize merging the draft into main or releasing it.

Pre-integration fixture checkpoint: `cd7d317f5e13ce4e319e976164c54ba47642d387`.
Current collaboration is on [Spaceport Crew](https://artifacts.folk-fujita.ts.net/p/spaceport-crew/).
The imported `9s44` and `q72g` tasks are claimed by `review-squad` and remain open.
The painting/polling finding `gfde` is closed after its five-clause audit and all four native image inspections.
See the [native image receipt](review-squad/native-images/qualification-a8f7bd371.json) and [Crew closure receipt](review-squad/crew-closures.jsonl).
The remaining children cover native Core Cmd-W, accessibility speech, and Clint's foreground release profile.
On November 20 at 02:54 UTC, Doctor Biz explicitly answered “Keep Mac checks pending.”
Do not launch Mac foreground fixtures until Doctor Biz changes that instruction.
Windows CI continues; this instruction does not pause the full review goal.
Clint confirms at 03:15 UTC that no foreground release profile has run and no measurement evidence exists.
He retains that dedicated desktop lane; the merged browser refactors remain groundwork rather than profile results.
His separate 03:17 UTC reply closes his Interrupted/copy review loop without merge or launch authorization.

## Native shortcut integration and framework correction

The topic integration retains main `eba200955` and all reviewed fixes from `53de2db63`.
It adds repeatable private GNOME input and expanded Windows keyboard fixtures.
Normal topic-branch integration has no conflicts; it does not merge the draft into main.

The [complete Linux receipt](review-squad/linux-native/spaceport-complete-16a928826.json) records exact source `16a928826`.
Ten shared tests, two receipt tests, and all forty-six native case outcomes pass.
The unchanged Cycle Windows regression fails on GPUI 0.3.10 and passes with exact 0.3.11 pins.
Native input activates the exact coherent private target, then normal cleanup restores the original window.
Final Polish search, key release, session closure, layout restoration, direct reaping, and daemon survival pass.
The upstream activation-context fix supplies valid input serial/surface context and separate pending-token ownership.
Existing non-GPUI dependency versions stay intact; the required new closure and obsolete Metal removal remain explicit.

The [Windows plan](review-squad/windows-native/qualification-plan.md) records partial results and failed attempts.
Source `c9a585ae5` passes compilation, Clippy, four Rust contracts, extended Polish AltGr, and direct chrome focus.
Its PowerShell preflight catches aggregate case-table selection before input; the console step skips.
Four expanded native tests stop at the same boundary.
Source `7e2459d67` corrects both actual JSON consumers and adds exact scalar-ID guards.
Its [native probe](https://github.com/2389-research/spaceport/actions/runs/44716174820) passes three matrix tests and twenty-eight native cases.
The [partial receipt](review-squad/windows-native/qualification-7e2459d67.json) records the console read-only error and unchanged Palette query.
That separate Windows probe still uses GPUI 0.3.10.
Combined-source Windows CI must qualify those fixtures with 0.3.11.

The first combined-source CI on `bd7ff6d26` exposes two shared-test unwrap lint failures and those same native preparation failures.
The test correction removes both unwraps without widening allowances.
SearchInput now applies assistive SetValue through its existing normalized setter and Changed notifications.
It registers no listener for masked fields and refuses stale masked-field actions too.
Terminal preparation uses tracked native Unicode input because Windows exposes its transcript value as read-only.
Its owned helper receives the exact disposable-runner marker and waits for command echo before Enter.
Independent review verifies these changes and the helper's existing cleanup ownership.
Shared candidate `3a603345a` includes both corrections and three new search editing unit cases.
The [Windows probe](https://github.com/2389-research/spaceport/actions/runs/44719150181) and [Linux probe](https://github.com/2389-research/spaceport/actions/runs/44719193679) now use that same source.
No corrected unit or native outcome is claimed before those runs finish.

Both shared-source probes now pass.
The [Linux receipt](review-squad/linux-native/spaceport-complete-3a603345a.json) records ten contracts, two receipt tests, and forty-six native case outcomes.
It verifies exact window activation, final Polish search, session/layout cleanup, direct reaping, and daemon survival.
The [Windows receipt](review-squad/windows-native/qualification-3a603345a.json) records thirty-eight native case outcomes and all seven selected native tests passing.
Four Rust contracts, three search accessibility unit cases, and both real PowerShell consumers also pass.
The private pane program receives F8, F6, Shift-F6, F11, and Control-PageUp/Down with exact key/modifier records.
Native Resume restores the assistive query, and all three font geometry cases pass.
Polish AltGr, direct chrome focus, key release, and layout restoration pass.
PR head `1feda67f7` has identical application, fixture, Cargo, and workflow files; only evidence differs.
Its [full configured CI](https://github.com/2389-research/spaceport/actions/runs/44720815181) remains the shortcut closure gate.
Its [supply-chain run](https://github.com/2389-research/spaceport/actions/runs/44720815809) passes all three jobs.
When configured CI passes, close only Kata `3wd4` with these exact-source receipts and fresh tracker verification.
Keep Core Cmd-W, audible accessibility, and release profiling open under the existing Mac hold.
Do not launch a Mac foreground fixture, merge into main, or release.

Canonical quick checks, workflow lint, offline workflow security checks, receipt parsing, and whitespace checks pass.
Locked workspace metadata resolves without compilation.
Independent read-only review finds no concrete integration regression in the actual combined diff.
No local compiler, VM, Mac foreground fixture, or release profile runs in this checkpoint.
The full configured CI matrix and shared-framework platform checks remain pending.
Kata stays forty-nine closed and four open; no incomplete child closes.
The [workspace-name correction](review-workspace-name-qualification.md) records the setup failure and successful native rerun on `1679d74f0`.
The prior work dashboard is archived; its earlier evidence and attachments remain intact.

## Navigation ownership integration from PR 136

The combined candidate incorporates main `9bb1adaf71361051384fb91cbb0d0ca59e360e21`.
Main moves terminal groups onto the window and relocates navigation and control target helpers.
The sole conflict retains endpoint-serving readiness and attach-only startup while borrowing the new window field.
Independent review finds one stale profile-helper borrow on the removed browser field.
Host all-feature compilation reproduces that error; the corrected borrow preserves paired live and presented state.
AST search finds no remaining old browser-terminal field access.

The combined group, control-target, browser, focus, tab, and Core-close scope passes 62 cases.
Its ordinary batch skips the one explicit Metal opt-in; that real raster case passes separately.
Host binary/test Clippy and both Windows binary/native-harness configurations pass with warnings denied.
Canonical quick checks and diff review pass.
Logs: `/tmp/spaceport-9s44-nav-integration-{consumers,metal,host-clippy,windows-all,windows-minimal,static}.log`.
No foreground Mac fixture, working-session profile, or corrected Windows capture runs locally.
New combined-source CI remains required before claiming current-head platform success.

## Smoke, replay, activity, and read-only link integration

The next normal merge incorporates main `61c57dee9cf9ce410ceb997ac5601f73c37ec8eb`.
It includes PRs 137, 139, 141, and 142.
The fixture registry retains the Windows popup-departure entry and one complete verdict.
Moved helpers retain typed `EmptySurface` diagnostics and footer failure state.
Windows file and URL cases remain enabled with actual native path fixtures.
Wrapped plain-text links intentionally underline without opening; local link openers retain their confirmation rules.
Replay additions test existing same-ID acknowledgement and conversation fencing without changing production replay policy.
Activity tests keep daemon snapshots as the sole status source.

Host compilation exposes a lost `Arc` import in the confirmation helper.
Windows compilation exposes a smoke-module shadow of the app's `sidebar` module.
Explicit imports and the app-module path correct both errors; the failed checks pass after correction.
Independent read-only review finds no material safety or contract regression.

The combined smoke, link, delivery, activity, focus, and tab scope passes 151 cases.
Its one Metal opt-in passes separately without opening a desktop window.
The ACP command scope passes nine cases, including acknowledgement after reconnect without repeated execution.
The client method and response scope passes sixteen cases.
Host binary/test Clippy and both Windows binary/native-harness feature configurations pass with warnings denied.
The four link-removal guards and canonical quick checks pass.
Logs: `/tmp/spaceport-9s44-wave12-{integration-consumers,integration-metal,replay,host-clippy,windows-all,windows-minimal,link-absence,static}.log`.
Fresh combined-source platform CI and corrected Windows captures remain pending.
Mac foreground checks remain pending under Doctor Biz's instruction.

The published combined source is `b94978e3bb323e7fa9d70934b63401f841bb8dd2`.
Its [CI run](https://github.com/2389-research/spaceport/actions/runs/44669528487) exposes two test portability errors and a separate native Core failure.
Default macOS Clippy lacks the Metal test's integration-feature gate; that bounded gate correction passes local default-feature Clippy.
All four Windows workspace jobs fail the same file-link assertion after PR 142's test additions.
One URL uses `RUNNER~1` while the expected URL spells the same directory `runneradmin`.
The assertion now compares canonical file identities instead of requiring one valid URL spelling.
All five related content-update cases pass locally; actual Windows execution remains pending.
Production file opening and path validation remain unchanged.

All 25 ordinary native UIA cases pass, including corrected tab captures and workspace consent.
The corrected tab bands qualify the four measured cases; see [the original after-images and ratios](review-accessibility-gaps.md#corrected-native-tab-paint-and-core-results-on-b94978e3b).
Core observing and resize pass, while ordinary Core text/events fail without a diagnosed boundary.
The next candidate adds separate native and runtime diagnostics without retries or weakened requirements.
All 19 CI jobs finish: eleven succeed, seven fail, and one unconfigured target skips.
The seven failures include the aggregate gate; the separate supply-chain run succeeds.

Initial published checkpoint: `150eed475c4097c63dff86edf8e2b215c64a1d02`.
Draft: [PR 125](https://github.com/2389-research/spaceport/pull/125).
The remote branch and draft head matched that initial commit when it was published.
The next verified draft head is `e8acf1aa6262f91d3610d5b97b5bb5a928b0ffa7`.
It includes main `099e380ff` and the combined-source checks recorded below.
Before this update, the published head was `6d89f5e42c0478aa865df329720f4a017e3764f5`.
Its first complete CI run failed; see [the qualification repairs](review-ci-qualification.md).
Main subsequently advanced to `57d6ea140722a613ce17beb9ec6d942e5d4c504f`.
The topic branch incorporates that main alongside the CI repairs.
See [the newer integration evidence](review-jargon-main-integration.md).
Main then advanced to `44c5325e8b902f70d90f26d19d9188ae87c49d78` through PR 121.
See [the reorder integration evidence](review-reorder-integration.md) for the current combined candidate.

## Native checkpoint and P2 corrections

Source `940749d29` passes full CI with eighteen successful jobs and one unconfigured skip.
Both native Windows jobs and supply-chain checks pass.
Native Settings keys, private saved preferences, and inspected focus images satisfy `d0n2`, now closed.
The two native clipboard-owner safety cases pass, including guardian cleanup after deliberate parent failure.
Actual product paste and cancellation qualification remain pending under `d378`.
The replacement Windows SSH pipe tests pass the configured default and all-feature jobs.

Doctor Biz subsequently identified popup Quit and uncertain chat-creation P2 regressions.
Both corrections pass 28 focused tests and all four host/Windows GNU lint configurations.
See [the focused P2 evidence](review-popup-quit-chat-outcomes.md).
Their new native popup Quit test passes on `64970ccce`, preserving the private daemon and tab IDs.
That source passes full CI with eighteen successful jobs and one unconfigured skip.
Both Windows native jobs and supply-chain checks pass.

Doctor Biz approved accepting Clint's separate read-only PR review on November 19.
The review targets current head `47fd690d6`, including the contracts Clint named.
It permits PR findings or a review document, with no edits, merge, push, or rebase.
The preceding P2 source `64970ccce` is fully qualified; the clipboard scenarios were still in CI at that point.

Both complete native clipboard application cases now pass on `1ad093a96`.
Their original-scope audit accepts completion; Kata `d378` is closed with event 9797.
That x64 native job has eighteen passing UIA cases and one separate terminal rectangle-fixture failure.
The managed API returns Rect objects and permits empty rectangles for a collapsed range.
The corrected qualifier checks the caret separately from its enclosing visible line geometry.
Current Linux work also removes four GNOME-conflicting defaults and moves Resume Palette to Ctrl-Shift-R.
Its original native Linux/Windows qualification remains open under `3wd4`.

## Terminal painter split integration

Streaming source `dd3495336` conflicted with main `fc9bb0c79` from PR 132.
At that checkpoint, GitHub reported `mergeable_state: dirty` and no current-commit workflow run.
The integration retains main's painter modules and the review fixes within those modules.
`surface.rs` preserves glyph shapes when only the theme changes.
`text.rs` retains bounded cold-glyph replacement.
`frame.rs` starts each cache epoch and retains the image test probes.
The shared image painter keeps destination masks, texture ownership, and layer ordering.
Both new glyph regression modules remain registered.
No dependency or wire contract changes occur in this integration.
The combined candidate passes 94 painter, region, and smooth-scroll tests.
Both ignored offscreen Metal pixel cases pass separately.
These cases do not qualify foreground native image scrolling.
All four canonical host and Windows GNU lint configurations pass.
Formatting, size, sorting, operations, localization, and diff checks pass.
Independent review confirms twelve moved methods retain the task parent's bodies.
The fresh-eyes review finds no material issue in the fourteen-path integration diff.
The known debug-link unwind and external `block` notices remain unchanged.
Streaming source matches published `dd3495336`; its prior semantic and release receipts remain applicable.
Logs use `/tmp/spaceport-9s44-painter-integration-{tests,metal,lint,static}.log`.
Published merge `a1f85f1ea` is conflict-free and matches the remote topic branch.
Its [CI run](https://github.com/2389-research/spaceport/actions/runs/44592189535) passes with eighteen successful jobs and one unconfigured skip.
Its [supply-chain run](https://github.com/2389-research/spaceport/actions/runs/44592189691) also passes.
Both Windows native jobs pass.
The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44592189535/job/135801197666) passes eight GUI and twenty UIA cases.
That includes Settings, terminal text/caret/line bounds, clipboard application cases, and popup Quit.
Its layout inventory again contains only US `00000409`; no AltGr claim follows.
This qualifies the published streaming and painter source across the configured CI platforms.
The later workspace-reorder and Herdr-resize native scenarios still need their own current-head execution.
This merges main into the draft branch; it does not merge the draft into main.

## Sound extraction integration

Main advanced to `b2a82175c3e54ecc529136d95ef23335b9ce0368` through documentation PRs 130 and 131.
The integration branch merges that three-path delta without conflicts.
It retains the sound dependency correction and the browser extraction proposal.
Rust source and dependency graphs match the validated pre-merge tree.
The browser proposal's Store path policy remains a future design decision.
The merge does not claim a link-size or startup improvement from sound extraction.

Current local validation passes forty-one command tests and seventy-three keymap, prefix, and resume cases.
Both canonical host and Windows GNU lint configurations pass.
The corrected native terminal harness also passes focused Windows lint.
Static checks pass after formatting the final Linux override assertion.
All four Linux check jobs pass on `fe4b3d7e4`.
The corrected native enclosing-line case passes, alongside eighteen passing UIA cases in total.
The Settings qualifier fails while observing Enter after one fixed 100 ms wait.
At that checkpoint, its bounded state-observation correction still needed native execution.
Source `b6214c15b` subsequently passes full CI with eighteen successful jobs and one unconfigured skip.
Its native x64 job passes all eight GUI and nineteen UIA cases, including the corrected Settings observer.
Both Windows native jobs and supply-chain checks pass.
Source `f904651ac` subsequently passes full CI and both native Windows jobs.
Its x64 job passes eight GUI and twenty UIA cases, including the loaded-layout diagnostic.
That runner has only US layout `00000409`; no Polish or AltGr delivery claim follows.
The validated streaming candidate passes 581 chat cases and the paired release preparation workload.
It reduces synthetic preparation by 34.0–91.7 percent without changing displayed or selectable results.
Current-head CI and remaining native qualification are pending; `gg9v` still requires its separate foreground profile.

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
The earlier diagnostic returns zero bytes before its assembly-loading markers.
The next plain-script probe executes in 167–180 milliseconds.
The UIA script reaches entry and completes encoding setup, then stalls at its first cmdlet.
Independent x64 and ARM probes pass startup/encoding and fail only `Write-Output`.
Restoring two documented system paths is insufficient on ARM and in native UIA.
A shared bootstrap now explicitly loads the two required system command modules.
Source `09b5ae61b` passes the unqualified cmdlet and owned-child reaping regressions on both Windows architectures.
The native x64 chrome tree and all seven keyboard-focus observations also pass.
The workspace-consent scenario reaches completed captures but loses the cached sidebar's accessible subtree.
Source `96b5295d5` bypasses that cache while accessibility is active and preserves ordinary caching.
It also resumes the close-sequence fixture after controlled inbox contention.
Both regressions fail first; the combined four-case suite and all four lint configurations then pass.
Independent review finds no remaining issue.
The [new CI run](https://github.com/2389-research/spaceport/actions/runs/44471538211) qualifies both native UIA scenarios.
Both Windows native jobs and the Linux ARM all-feature job pass.
The complete run passes all 18 executed jobs; one unconfigured target is skipped.
The separate supply-chain run also passes.
The existing UIA deadline remains unchanged.
An independent audit also finds reduced-motion gaps in extracted slides and opted-in chat glide.
Their three regressions fail first; the corrected focused scopes pass 70 tests.
The normal-consumer test retains 150-millisecond slide timing.
See [OS accessibility evidence](review-os-accessibility.md#reorder-extraction-follow-up).

Kata now has 46 closed children and seven open children.
Reduced motion closes after its extracted-slide and chat-glide corrections pass scoped verification.
Herdr TUI workspace closing closes after native creation, consent, cancellation, and preservation checks pass.
Recovery-button keyboard access closes after native focus and retry verification, with scoped headless callback evidence.
Terminology closes after the final clipboard error family passes its output, category, and localization checks.
Status shapes close after both Windows native jobs pass and all eight monochrome marker scenes are inspected.
Visible errors close after real Windows warning, toast, and Zen-warning live-region events pass native qualification.
Popup departures and Close Others close after native animation, stable-ID consent, and all nine image inspections pass.
Chat accessibility closes after native content, named controls, deliberate permission answers, and image inspection pass on `99e318867`.
Shortcut repeats close after native window, tab, split, custom-command, and failure-cleanup qualification passes on `5c696d3c8`.
Secondary text contrast closes after 24 native color measurements and inspection of all four palette PNGs on `8a4beb1ec`.
Source `b20cca86f` then passes full CI: eighteen jobs succeed and one unconfigured target skips.
Its separate supply-chain run also succeeds.
The umbrella remains open.
Native Settings readiness is pending Doctor Biz's answer.
Both Windows native GUI jobs and the x64 accessibility tree/focus case pass on `46738c99c`.
Windows workspace path resolution and mismatched-owner rejection also pass on that source.
The full accessibility matrix and interactive release profiling remain pending.
Offscreen Metal pixel checks qualify rendering only.

## Settings extraction integration, November 19

Main merges PR 133 as `2e690b2da`.
That extraction conflicts with the published `cd7d317f5` review branch.
GitHub reports the draft as dirty, and no new native workflow starts.
The integration merges main into the review branch; it does not merge the draft into main.

The settings crate owns the moved values, files, and guarded writes.
The app keeps native probes, window refreshes, keymap derivation, and its queued-save worker.
The pure accessibility cache moves below the app without creating a second policy source.
Its one snapshot serves both motion and theme readers.
Test overrides use the settings crate's existing `test-support` feature.
Native palette rendering stays in GPUI, correcting Git's suggested move into the settings crate.
The obsolete Windows contrast poller stays removed.

Both focus-follows-mouse defaults remain false.
Runtime pane-history reads remain separate from private view settings.
Typed settings errors retain their causes through the app boundary.
Theme colors retain the text contrast correction and raw palette loading.
Initial appearance and reloads retain a coherent settings/keymap pair.
The existing preference worker and native palette scene match the pre-merge task source exactly.

An existing policy test first fails with Standard contrast instead of High.
The shared-cache correction passes the same test.
The focused app consumer command then passes 291 tests with none ignored.
It covers startup, policy, keymaps, prefix routing, Settings, preferences, and affected pane confirmation helpers.
The final settings suite passes 306 tests with none ignored in 4.64 seconds.
That run explicitly includes the relocated native announcement configuration test.
Review caught its initially missing registration before publication.
Private-prefix and platform-derived view-key tests now load real fixture files before building the app keymap.
The retired chat-shortcut test also remains registered through that real loader.

The model command is `cargo test --locked -p sheprd-settings --all-features --lib -- --nocapture`.
The app command is:

```sh
cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- \
  app::tests:: accessibility::policy::tests:: keymap:: window::prefix::tests:: \
  settings_window::tests:: settings_window::host_history::tests:: preferences::tests:: \
  core_terminals::tests::pane_owner::close_confirmation:: chat::tests::env:: chat::start:: \
  terminal_painter::tests::glyph_cache:: settings_window::native_png::tests:: --nocapture
```

Logs use `/tmp/spaceport-9s44-settings-merge-{policy-red,policy-green,consumers,model-tests}.log`.
The earlier 305-case log remains at `/tmp/spaceport-9s44-settings-merge-model-before-registration.log`.
`just lint lint-windows` passes all four workspace/all-target feature and platform configurations.
The existing external `block` future-compatibility notice remains unchanged.
The debug test linker also retains its existing compact-unwind warning.
Combined native CI remains pending.
The five original findings remain open.

## Headless executable fixture race

The `63a885b15` [Linux ARM all-feature job](https://github.com/2389-research/spaceport/actions/runs/44610744275/job/135866673252) fails the trusted PATH-selection case.
The actual helper reports `Text file busy (os error 26)` for the private trusted executable.
Six other headless cases pass, and all 454 ACP unit cases pass before that failure.

The test writes executable scripts from the parallel test process.
A sibling fork can inherit that writable descriptor until its own exec.
Linux then refuses to execute the still-open script.
The existing `crates/test-support/executable.rs` writer already prevents this race in client tests.
It writes through a single-threaded shell child, then waits before returning.
The ACP PATH fixtures now use that same helper.
The production agent launcher, path-selection assertions, deadlines, and test parallelism remain unchanged.
No retry or sleep was added.

`cargo test --locked -p sheprd-acp --all-features --test headless_exit -- --nocapture` passes all seven cases locally.
The suite takes 2.35 seconds after compilation.
Its unwind fixture deliberately catches its own panic while verifying cleanup.
The log is `/tmp/spaceport-9s44-headless-executable-race-tests.log`.
That macOS pass does not qualify the Linux race correction; the next Linux CI run remains required.
ACP all-target/all-feature Clippy passes with warnings denied.
The Unix-only shared helper expects the Windows-console lint at its import, beside the existing sandbox exception.
No production lint rule changes.
Quick format, size, sorting, operations, i18n, subject, and diff checks pass.
The native result on this source is recorded in [accessibility evidence](review-accessibility-gaps.md#native-result-after-settings-extraction-on-63a885b15).

On `4194cb1ab`, the [Linux ARM all-feature job](https://github.com/2389-research/spaceport/actions/runs/44614263920/job/135878875345) passes.
Its seven headless cases pass with none ignored in 2.10 seconds.
All four Linux jobs pass on that source.
The same source also qualifies actual Herdr keyboard resizing; see [the native result](review-accessibility-gaps.md#resize-qualification-and-scaling-diagnosis-on-4194cb1ab).
Enlarged Settings captures expose a keyboard-modality wait and clipped navigation labels.
The scoped corrections preserve native guards, configured font size, and chosen navigation widths.
Fresh native qualification remains required before closing the accessibility finding.

## Shortcut and browser integration, November 19

Published `f88c8c252` fixes enlarged Settings navigation width and the guarded capture handoff.
Its navigation regression fails first, then passes while preserving chosen widths and reset behavior.
Both native after-images remain pending.

Main `e13b88c30cb31ce510825805b2be5edfd4dd4299` merges PRs 134 and 135.
The normal topic-branch merge accepts the shortcut aliases and browser ownership extraction.
Linux retains source-confirmed GNOME exclusions and its direct Ctrl-Shift-R Resume binding.
Windows retains the shared catalog, character-input protection, and keyboard-mapper correction.
Private prefix routes and their custom-chord precedence remain intact.
Native chrome and held-window fixtures follow the revised Ctrl-F6 and Ctrl-Shift-N defaults.
Their process ownership, focus guards, event counts, and cleanup remain unchanged.

Browser operations borrow endpoint, snapshot, store, and workspace state through `BrowserHosts`.
Core admission uses the frozen hub-enabled flag, preserving restart-only opt-in behavior.
Runtime pane history remains separate from private view settings.
Independent review finds no material shortcut, restoration, or browser-ownership issue.

The command suite passes 46 tests with none ignored.
The focused UI consumer run passes 334 tests with none ignored in 1.25 seconds.
Logs are `/tmp/spaceport-9s44-wave10-commands.log` and `/tmp/spaceport-9s44-wave10-consumers.log`.
The shortcut snapshot is regenerated through its existing producer and reviewed.
`just lint lint-windows` passes all four workspace/all-target feature configurations.
The existing external `block` future-compatibility notice remains unchanged.
File-size, sorting, operations, i18n, commit-subject, and diff checks pass.
The added-line credential scan finds no credential patterns; the destination remains the public Spaceport repository.
Fresh native checks and platform CI remain required on the published merge.

Crew currently records 49 of 53 children done after Clint closes the shortcut finding.
Four board-open findings remain: Core confirmation, painting/polling, accessibility, and foreground profiling.
The shortcut finding's original native keyboard requirement also remains unverified by this squad.
The coordination question is recorded on Crew as `20261119T215435870Z.30aefa`.
No draft merge or release is authorized by this checkpoint.

On `7b7498762`, both Windows native jobs pass their configured cases.
The x64 job passes eight GUI cases, one pipe-owner case, and all twenty-three UIA cases.
Revised Ctrl-F6 navigation and held Ctrl-Shift-N retain their native assertions.
The enlarged Settings case now produces both bounded PNGs and preserves private preferences.
Root inspects both native images and qualifies their enlarged glyphs, navigation, wrapping, and viewport containment.
See [the captures and remaining accessibility gates](review-accessibility-gaps.md#enlarged-settings-qualification-on-7b7498762).
The [complete CI run](https://github.com/2389-research/spaceport/actions/runs/44624169019) succeeds on that exact source.
Eighteen jobs pass and one unconfigured release target skips.
Both macOS release executable smoke checks and all Linux/Windows feature configurations pass.
The separate [supply-chain run](https://github.com/2389-research/spaceport/actions/runs/44624169010) also succeeds on `7b7498762`.
The following commits add only evidence documents, original captures, and their receipt; application code remains unchanged.

## Review follow-up and settings-message integration

Review corrections are committed in `f3dda69b2`; the image cleanup rename is in `d2de8939e`.
The native image run failed before its producer; see [the exact failure](review-image-pane-scroll.md#first-windows-execution-on-97864f275).
No image qualification or task closure follows from that run.

Main advanced to `fa63a98d8` through PR 138's settings-message correction.
Its thirteen-path delta changes diagnostic file names and corresponding tests.
The merge has no conflicts and retains both sets of settings error corrections.
The combined diagnostic consumers pass 87 cases, with none ignored, in 1.83 seconds.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- i18n::errors::tests:: config::tests::diagnostic_text:: config_diagnostic::tests::`.
Log: `/tmp/spaceport-9s44-review-p2-main-consumers.log`.
Root reviews the complete main delta and the merged settings/localization producers.
Full platform CI and the real Windows image rerun remain pending for this combined source.
The previous four lint results apply to `f3dda69b2`; they are not combined-source CI results.

## Render-helper integration, November 20

Main `eba200955f38c8b688849d027196dfda9249774b` merges PR 140's render helpers.
The normal topic merge resolves eight conflicts without replacing the reviewed behavior.
Composer, rail, tab-strip, polling, constructor, command, and window-frame helpers follow main's structure.
The prepared popup renderer and shortcut-page renderer remain unchanged from the review branch.
Those boundaries retain inert departure controls and frozen page ownership.
The sidebar toggle stays local; Core-only fallback refuses absent Herdr TUI endpoints.
Root image batches retain paired assets through cached cell replay.
One flash element follows its terminal or window owner, preserving warning and clipboard feedback.

Two compile errors exposed incorrect private flash argument types during integration.
The arguments now match the existing `Option<Div>` result without converting or copying state.
The focused consumer run passes 717 cases with none ignored.
The boundary run passes 745 cases; four opt-in or private-producer fixtures remain ignored.
The explicit offscreen Metal run passes all three image-fragment, blending, and focus-ring cases.
Those Metal checks open no desktop window.

Commands use `cargo test --locked -p sheprd-gpui --all-features --bin spaceport` with these filters:

- `-- chat::tests:: menu::tests:: window::tests:: sidebar::rail::tests:: sidebar::tests::snapshot_index custom_shortcuts`
- `-- menu:: endpoint:: window::regions:: window::flash:: window::tab_strip:: terminal_painter::tests::images`
- `-- --ignored window::regions::tests::images::metal_raster:: tab_layout::tests::focus_raster::`

Package Clippy passes all targets with all features and without default features.
Both configurations also pass against `x86_64-pc-windows-gnu`.
Each command uses `--locked -p sheprd-gpui --all-targets` and `-- -D warnings`.
The inherited external `block` future-compatibility notice remains unchanged on macOS.
Canonical format, file-size, sorting, operations, i18n, commit-subject, and diff checks pass.
Logs are `/tmp/spaceport-9s44-render-integration-{consumers,boundaries,metal,host-all,host-minimal,windows-all,windows-minimal,static}.log`.
Independent review inspects the actual merge, callers, ownership, and focused test evidence without material findings.
Root's final diff review finds no additional issue or added credential pattern.

Full workspace suites, Linux-specific compilation, and fresh Windows native qualification defer to CI.
The bounded keyboard-profile parser and chat diagnostic-walk correction still need their actual Windows reruns.
Doctor Biz keeps Mac foreground checks pending; this integration opens no Mac fixture.
Kata remains 49 closed and four open; Crew separately remains 50 done and three doing.
No native-pending finding closes, and no merge or release follows from this checkpoint.

## Native results and Core viewport correction

Actual CI on `3a0bcb00d` finishes with 16 successes, two failures, and one unconfigured skip.
All workspace jobs, configured Mac builds, Windows installer, ARM native tests, and supply-chain jobs pass.
The x64 native job passes eight GUI cases, pipe ownership, and all 25 ordinary UIA cases.
Chat actions and the Polish AltGr qualification now pass with their original ownership and cleanup assertions.
Core ordinary and observing text pass; right-side resize restoration fails native containment.
The aggregate failure follows that direct x64 failure.
See the [scoped receipt](review-squad/windows-native/qualification-3a0bcb00d.json).

The correction bounds Core accessibility text to the current canvas during a retained-grid resize.
The [accessibility evidence](review-accessibility-gaps.md#core-resize-containment-on-3a0bcb00d) records the regression and exact verification scope.
Three initial regressions fail first; the focused consumer run then passes 124 cases with two explicit Metal opt-ins ignored.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- terminal::accessibility:: core_terminals::view::tests:: core_terminals::tests:: window::accessibility:: window::tests::core_ window::regions::tests::`.
The Mac debug test linker reports its existing 16 MiB unwind-table warning.
The same warning appears in the earlier `/tmp/spaceport-9s44-render-integration-consumers.log`.
The inherited external `block` future-compatibility notice also remains.
Neither warning comes from a newly introduced diagnostic allowance or disabled check.
Package Clippy passes all targets with all features and without default features on macOS and Windows GNU.
Commands use `cargo clippy --locked -p sheprd-gpui --all-targets`, the selected feature flag, and `-- -D warnings`.
Windows runs also select `--target x86_64-pc-windows-gnu`.
Canonical quick checks, JSON receipt validation, and diff checks pass.
Independent actual-diff review finds no material regression; the following native run qualifies the correction.
Logs are `/tmp/spaceport-9s44-core-viewport-{host-all,host-minimal,windows-all,windows-minimal,static}.log`.

## Green platform checkpoint on `d747cdbdb`

The [complete CI run](https://github.com/2389-research/spaceport/actions/runs/44680270572) succeeds on exact source `d747cdbdb82e48e6f4efb14866416f789a118b8f`.
All 18 configured jobs pass; one unconfigured release target skips.
The [supply-chain run](https://github.com/2389-research/spaceport/actions/runs/44680270699) also passes all three jobs.
Both configured Mac builds, all workspace feature/platform jobs, Windows installer, and both native jobs pass.
The x64 native job passes the original strict Core resize case, ordinary/observing text, chat actions, and Polish input.
See the [qualified receipt](review-squad/windows-native/qualification-d747cdbdb.json) for exact native times, assertions, and scope.
The native release-performance step skips and supplies no working-session profile.

Following commits add only evidence documents and the viewport gotcha.
Application, fixture, build, and workflow files remain unchanged from the qualified source.
Their own CI results remain separate from this exact-source qualification.
Kata remains 49 closed and four open; no Mac, speech, Linux/GNOME, or profiling gate is waived.
The earlier intermittent ordinary Core failure still lacks an established cause, despite later native passes.
