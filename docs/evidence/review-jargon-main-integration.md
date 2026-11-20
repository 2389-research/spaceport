# Agent ownership and error-copy integration

Scope: integrate main `57d6ea140722a613ce17beb9ec6d942e5d4c504f` into draft PR 125.
This follows local CI repair checkpoint `b3be7225ed3270f7a8a866cf574ac7eed914688d`.
Main adds the ACP cleanup work and the additional error-copy work from PRs 111 and 123.
Doctor Biz's existing commit and draft-publication approval covers this necessary integration.
No main merge or release is authorized here.

## Error guidance

Resolved 21 textual conflicts while retaining the reviewed safety guidance.
Timed-out requests may have happened; the person must check before retrying.
Timed-out input may have arrived; the person must reconnect explicitly.
Unreadable saved connections retain their data instead of recommending deletion.
Version guidance retains the correct product and the named Reconnect action.

Kept main's additional protocol-error text and its localized messages.
Kept main's other compatible error wording and typed source chains.
The localization equality test caught twelve catalog messages left behind by overlapping conflict sections.
Updated those messages to match the merged Rust Display text.
The retained protocol tests also check for wire terminology in displayed messages.
Independent review found two shared size errors that incorrectly blamed inbound daemon data.
Their final text describes size bounds without assuming a message direction.
Three real codec cases cover empty encoding, oversized encoding, and empty decoding.
The oversized accessibility producer also retains its no-input assertion.
The imported copy proposal is explicitly historical and links this final guidance.

## Unix process ownership

Both candidates could signal a numeric process-group ID after reaping its leader.
Capturing that number at spawn does not prevent reuse after reaping.
A zero-signal probe cannot distinguish a replacement group with the same number.
Independent review rejected main's claim that capturing the number alone solved that hazard.
The unused reaped atomic was not retained.

The Unix child owner now observes exit with `waitid` and `NOWAIT`.
It caches the agent's status while retaining the unreaped leader.
That leader reserves the identity until group cleanup finishes.
The same mutex covers observations, group signals, and the final reap.
Cleanup retires the group before releasing the native identity.
Repeated stop calls cannot signal the retired number.
Only this owner may reap the child; arbitrary external reapers are unsupported.

The owner preserves normal exit codes and signal outcomes.
It preserves cleanup of tools surviving their agent's exit.
Interrupted observations retry within the grace period.
Other observation errors cannot skip final cleanup of the owned group.
Missing ownership forbids subsequent numeric signals.

The Unix child also cleans up on final owner drop and pipe setup errors.
A run-scope stop guard covers setup errors while a waiter still holds a shared handle.
The guard is created before connecting streams or starting the waiter.
Windows retains the existing JobObject owner and receives the same run-scope guard.

The unreaped leader can keep the group probe positive after exiting.
Cleanup therefore uses the existing bounded two-second grace before its final KILL.
That cost applies even when the direct agent has already exited.
An open session can retain one exited leader until cleanup.
This change makes no interactive performance claim.

## Verification

The native ownership regression first failed because observation had already reaped the leader.
The run-owner regression first failed with a live waiter after guard drop.
Both observation-error regressions first failed because cleanup returned early.
Their fixtures perform cleanup before asserting failures.

The completed all-feature ACP suite passes 453 unit cases.
Its branding case and seven executable headless integration cases also pass.
All ACP cases report zero failures and zero ignored cases.
Tests cover native waitability, normal and signal status, signal/reap ordering, repeated stop,
concurrent stop/wait, final drop, run-scope cleanup, and a surviving TERM-ignoring tool.
Independent review approved the completed owner and its error paths.

Client, protocol, and terminal all-feature suites pass 521 cases in total.
Ten explicit-resource cases remain ignored in those suites.
These results do not qualify a live personal daemon.

The final focused GPUI all-feature run passes 180 cases with ten private fixtures ignored directly.
The scopes include every localized error mapping, protocol terminology, connection setup,
window options, status bars, padding, and the reported macOS CI failures.
The corrected client catalog now matches Display across every tested variant.

Canonical `just lint` passes both host feature configurations with warnings denied.
Canonical `just lint-windows` passes both Windows GNU feature configurations with warnings denied.
Windows cross-compilation does not prove native Windows behavior.
Formatting, file sizes, sorted regions, operations, localization, and specification checks pass.
The specification validator explicitly skips JSON Schema validation.
The existing compact-unwind and dependency future-compatibility notices remain; no new source warning appears.

Logs use `/tmp/spaceport-9s44-jargon-*.log` and `/tmp/spaceport-9s44-unix-*.log`.
The run-owner baseline is `/tmp/spaceport-9s44-run-owner-red.log`.
Linux and native Windows qualification remain assigned to hosted CI.
Native Settings input, accessibility, and release profiling remain pending.
Kata closures remain unchanged while those qualification gates stay open.

## Clipboard message follow-up, November 19

The original-scope completion audit finds four remaining clipboard messages with internal terms.
The Busy message exposes a reservation; the fallback message exposes semantic-input targeting.
The timeout messages expose a skipped reservation or an unnamed reconnect instruction.
These messages reach the real `Localize` implementation and its English catalog.

The corrected messages describe clipboard pasting and name **Terminal → Reconnect** when needed.
Preparation applies to clipboard text and images, so the message covers both.
Before preparation completes, the worker still has no bytes to send.
Its timeout message can therefore say this paste was not sent.
A write timeout may follow partial input, so its message retains that uncertainty.
Error variants, source chains, retry categories, framing, and cancellation remain unchanged.

The existing error-output/category regression fails first against the old Busy message.
The corrected client scopes pass 34 cases without ignored cases.
The six GPUI client-localization cases also pass, including English Display equality and pseudo-locales.
Logs use `/tmp/spaceport-9s44-clipboard-copy-{red,client-green,localized-green}.log`.
Independent review checks the actual reservation, preparation, and incremental write producers.
Native speech and visual qualification do not establish this wording contract and remain separate work.

The final client library scope passes 33 cases after moving the separate reader-closure test into its own integration target.
All four host and Windows GNU lint configurations pass with the corrected copy.
Independent completion review finds no remaining original-scope terminology gap.
Kata `cx2r` closes on `4156467f3` with event `9785` and UID `01M7SAGQPPECJE259BM03R7Q2B`.
Fresh issue and parent reads confirm 40 closed children and 13 open children.
The umbrella remains open.

## PR 125 review follow-up, November 19

[Clint's review](https://github.com/2389-research/spaceport/pull/125#issuecomment-7024224759) found remaining visible connection jargon on `3307213db`.
The earlier completion assessment missed activation notices, batch readiness, and chat mirror validation.
Those messages now describe terminal screens, connection readiness, and chat connection identifiers.
The unsupported-switch error receives the same correction in Rust and Fluent.
Chat mirror identifiers name chat attach connections; they do not name saved SSH hosts.
Configuration keys, message IDs, typed errors, protocol values, and source chains stay intact.

Both rendered-copy regressions fail against the old messages and pass after the correction.
The config localization table now includes `InvalidChatMirror`.
English Display equality, inserted labels and reasons, and complete pseudo-localization remain covered.
The localized-error and Core-close scopes pass 62 cases, with none ignored.
Logs: `/tmp/spaceport-9s44-p2-copy-red.log` and `/tmp/spaceport-9s44-review-p2-consumers-green.log`.
Native screenshots of the updated error messages remain pending desktop fixture readiness.

The review also identifies asymmetric interruption handling in the agent exit waiter.
The waiter now retries only `Interrupted`, before sleeping or reporting exit.
Other errors retain their original OS causes.
The child lock drops between observations and before sleeping.
The regression injects one interruption, then observes an actual private child exiting with code 7.
It also verifies that the exact leader remains waitable until cleanup.
The baseline fails on the injected interruption; all 13 process cases pass after the correction.
Logs: `/tmp/spaceport-9s44-interrupted-wait-{red,green}.log`.
This proves portable retry policy; it does not reproduce native signal delivery.
Independent review finds no material ownership, cancellation, or cleanup regression.

The full TERM grace remains intentional after the leader exits.
That exit cannot establish whether descendants stopped.
The waitable leader reserves the group identity until cleanup finishes.
New comments explain this cost; signaling and timing remain unchanged.

Both canonical host lint configurations and both Windows GNU lint configurations pass.
Format, file size, sorting, operations, localization, commit subjects, and diff checks pass.
Logs: `/tmp/spaceport-9s44-review-p2-{lints,static}.log`.
Root's final review checks the changed producers, localized consumers, and both new test files.
The existing external `block` compatibility notice remains unchanged.
Full platform CI remains pending for this follow-up; no native keyboard or speech claim follows.
