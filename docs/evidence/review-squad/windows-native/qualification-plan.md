<!-- ABOUTME: Tracks the remaining Windows keyboard qualification for Kata 3wd4. -->
<!-- ABOUTME: Separates prepared native tests from actual execution evidence. -->

# Windows keyboard qualification

Base: `53de2db63b4912f8325b348fdaffa63dce06ffd6`.
Worktree: `.worktrees/review-squad-windows-native`.
Branch: `fix/review-squad-windows-native`.

Doctor Biz authorized this isolated test implementation and static verification.
The implementation owner returns a reviewable diff before publication.
Root's existing scoped commit and push authorization applies after integration review.
No additional PR, merge, or release is authorized.
Local compilers, VMs, native desktops, and the Mac foreground remain held.
The exact-branch Windows probe will supply compilation and execution evidence.

## Success criteria

- Native scan codes select tab positions 1 through 9 by exact daemon IDs.
- Reported punctuation reaches real history, pane, popup, group, and window outcomes.
- A real pane program acknowledges console attachment before capturing pane-owned keys.
- Captured F6, Shift-F6, F11, and Ctrl-PageUp/Down retain their key and modifiers.
- Revised fullscreen, tab navigation, Zen, sidebar, and status-bar defaults have real outcomes.
- Retained Windows arrow defaults focus exact panes beside real right and down splits.
- Polish AltGr-N and AltGr-Z produce their real letters without app actions.
- Every layout scope releases keys and restores the original GUI HKL and profile set.
- Fixture cleanup touches only exact children, windows, and private paths it owns.

## Full CI follow-up on `1feda67f7`

The [full run](https://github.com/2389-research/spaceport/actions/runs/44720815181) ends with 16 passing jobs, two failures, and one unconfigured skip.
The aggregate fails after the Windows x64 native job.
Both Mac builds, the Windows installer, and all platform feature checks pass.
The [supply-chain run](https://github.com/2389-research/spaceport/actions/runs/44720815809) passes all three jobs.
The complete native shortcut matrix passes again.

Ordinary UIA passes 24 cases and fails the Settings segment check.
That check reads selection immediately after `SendWait` and a fixed 100ms sleep.
[Microsoft documents](https://learn.microsoft.com/en-us/dotnet/api/system.windows.forms.sendkeys.sendwait) that the newer implementation does not await another process.
The candidate follows the existing switch qualifier's bounded acknowledgement.
It submits the original key once, then requires the same focused segment's selected state within three seconds.
It neither retries input nor changes the app's segment behavior.
The existing failed native test supplies the baseline; the candidate still needs native execution.

Core passes its observing and resizing cases.
The ordinary case reports `changed=True output=False marker_present=True` with 222 UTF-16 code units.
Its independent runtime observer also sees the marker.
These receipts do not distinguish an echoed command, padded output, or another rendering/input defect.
The diagnostic preserves the five-second deadline and exact standalone-line assertion.
Only on failure, it records the fixture's bounded, JSON-escaped private terminal text.
The terminal uses private runner state and authored input; no personal session is attached.
No Core fix or cause is claimed.

The exact-branch probe now includes this Settings case and all three Core cases.
It stages Core through the existing checksum-pinned installer.
Doctor Biz's Mac hold and Root's local compiler/VM hold remain active.
Kata remains 49 closed and four open.

### Follow-up result on `7fe7ac05c`

The [probe](https://github.com/2389-research/spaceport/actions/runs/44723868088) passes formatting, Clippy, four contract cases, and three search unit cases.
Its four shortcut matrix cases, Polish AltGr case, and direct chrome case pass.
The exact Settings case passes its keyboard, persistence, painted ring, and dropdown assertions.
This qualifies the bounded Settings acknowledgement on this source.

The console case fails before submitting Enter or any measured shortcut.
Native text shows a truncated prepared command tail after the full character burst.
Its exact owned focus and layout guards pass.
The client has a bounded 64-command queue; the helper injected the whole command without observing intermediate progress.
That permits overload, but the available log does not prove where these characters disappeared.
The correction sends at most sixteen authored characters, then observes that prefix before sending the next chunk.
It preserves the complete command-echo barrier, original overall deadline, and cleanup reserve.
It sends each character once and never retries an uncertain submission.
Intermediate prefix checks ignore only trailing spaces omitted by the existing transcript producer.
The complete final command retains its exact comparison, including any trailing spaces.

The ordinary Core case fails again; observing and resizing still pass.
Its escaped private screen text contains `SP_A11Y_7rs\\runneradmin` on one line.
The marker exists, but a stale prompt suffix prevents the required exact output line.
This establishes native accessibility text corruption.
The actual painted terminal has not been compared with that failed accessibility transcript.
The next diagnostic drains the existing read-only runtime attachment for one second after failure.
It preserves the 1 MiB capture bound.
It displays at most 8 KiB of captured bytes as escaped text and byte-exact hex.
It also records the terminal dimensions observed before attachment.
No replay, corrective terminal input, production change, or Core cause is claimed.

### Private stream capture for deterministic replay

The integrated PR head remains `287f65aa5` while its full CI runs.
A separate Windows topic candidate retains the ordinary Core capture on success too.
This avoids losing all raw evidence when the intermittent corruption does not recur.
The capture reads the same observer, preserves its 1 MiB bound, and prints at most 8 KiB.
Escaped text remains readable; hex preserves the exact bytes.
The runtime marker must still arrive from the exact terminal.
Its original ten-second success budget remains unchanged.
Failure capture retains its separate one-second budget.
No input, UIA assertion, runtime target, or production code changes.
Captured bytes need deterministic emulator replay before any Core fix is justified.

### Captured failure and Cycle source readiness on `70df119a6`

The [probe](https://github.com/2389-research/spaceport/actions/runs/44729475426) reproduces ordinary Core corruption.
It captures all 1,342 stream bytes, with runtime dimensions 112 columns by 34 rows.
The capture contains the initial screen, Win32 input mode, typed command, actual output, and following prompt.
Only the owned runner's authored input and paths appear; no personal session is attached.
The topic adds real-emulator replay and fragmentation checks using those exact captured bytes.
A bounded diagnostic reports selected resize interleavings without claiming they reproduce the native cause.
Fixed-size replay reads the emulator's test frame.
Production also uses damage publication, app frame patches, and transcript projection.
A passing replay alone cannot distinguish those later paths or establish the native cause.
No production correction follows before those results.

The same probe fails Cycle Windows before layout mutation or measured input.
Its immediate document lookup finds no unique terminal in the selected source window.
The failure diagnostic then sees one document in each of the two owned windows.
The candidate waits within the existing phase and overall deadlines for that source's unique document.
It keeps the exact HWND, chord, target, key-release, and layout-restoration assertions.
No evidence establishes a shortcut delivery regression, so `3wd4` remains closed.
The qualified PR head `287f65aa5` and its complete green CI remain unchanged.
Kata remains 50 closed and three open.

### Fixed replay result and production pipeline candidate

The [probe on `46340b12e`](https://github.com/2389-research/spaceport/actions/runs/44731650086) passes all tests.
The three captured-stream tests pass, including all fragment boundaries.
None of the selected resize interleavings reproduces the native stale prompt suffix.
All 38 native shortcut IDs pass again, including Cycle Windows with source-document readiness.
Settings and all three Core cases also pass; ordinary Core retains its complete success capture.
This narrows the parsing experiment without establishing the native cause.
The failed and successful observer captures each contain 1,342 bytes.
After replacing only the owned six-character temporary directory names, their ordered bytes are identical.
The twelve differing bytes come from those directory names in two prompts.
These captures belong to the read-only observer, not the UI's active attachment.
They do not exclude differences in that active stream, timing, damage, or native provider state.

The next test feeds the same bytes through real damage publication, `Screen::apply`, and `Transcript::read_frame`.
It compares complete app frames across chunk sizes and checks full and narrow text projections.
The captured bytes now live once in shared test support.
The UI crate adds only the existing emulator crate as a test dependency.
A test-only re-export reaches the existing app frame consumer without widening production interfaces.
No production behavior changes; the local compiler and Mac holds remain active.
The guarded workflow also runs pure emulator replay as a separate Linux job.
Its completed log is available without waiting for the native UI build.

### Private frame observer boundary

The `9de6683ff` probe passes the damage-to-transcript test and both CI jobs.
All frame chunk sizes preserve the prompt and standalone output at both canvas widths.
This still does not capture the running UI's active stream or frame.

The next diagnostic observes that actual app frame through the existing bounded log writer.
Only the private fixture's matching nonce, session, and named pipe may enable it.
A pure scope test first requires the exact target and rejects unrelated, missing, and malformed values.
The initial scope implementation denies every target; its positive test must fail before implementation.
No app observer is enabled in that baseline.
The native probe depends on successful pure tests, preserving expected baseline failure without running unrelated native cases.
Mac foreground, local compiler, and VM holds remain active.

The scope baseline run on source c753a96c8 fails exactly its positive-target assertion.
Five other pure checks pass; the dependent native job skips.
The implementation now accepts only a matching 32-hex nonce, session, and private named pipe.
Missing, malformed, and unrelated targets remain refused.

The Windows integration-only observer records the app's actual frame before accessibility projection.
It captures dimensions, queued size, canvas bounds, cell metrics, and three sampled rows.
Each row includes at most 96 characters and its sampled concealed-cell count.
The existing diagnostics writer bounds and queues records without blocking the UI thread.
The private fixture reads those records through the existing local logs command.
Success requires an observer record; failure retains the original error alongside available frame diagnostics.
This observes app state, not painted pixels, and changes no terminal input or render path.
Independent read-only review found no material issue with scope, bounds, ownership, types, or module paths.
Formatting, file-size, sorted-list, recipe, translation, commit, whitespace, and workflow checks pass locally.
Source e68a3561f passes both jobs in [run 44738518938](https://github.com/2389-research/spaceport/actions/runs/44738518938).
Linux and Windows each pass six scope and stream checks.
Windows also passes the production pipeline test, frame sampler test, and all native probe cases.
Thirteen actual app-frame records retain a 112-by-34 grid and matching queued dimensions.
Their final rows contain the full command, standalone output marker, and next prompt.
The sampled cells have no concealment flags.
This successful observation does not reproduce or fix the intermittent native text corruption.

The next probe runs six fresh private ordinary-Core fixtures after its three existing Core cases.
Every sample runs once and retains its own frame records and result.
Any failure keeps the entire step failed, even when later samples pass.
This fixed reproduction series seeks a failed app frame; it does not qualify the defect as fixed.
All original input, assertions, deadlines, target isolation, and cleanup remain unchanged.

### Failed app frames and startup height

Source 8ffbf97c2 [run 44740706737](https://github.com/2389-research/spaceport/actions/runs/44740706737) preserves two ordinary Core failures and five successes.
The first suite's ordinary case fails; focused sample four also fails.
All six focused samples execute, and the job remains failed.
The observing and resize cases pass, as do all earlier probe steps.

Both failed app-frame sequences begin at 80-by-24, then grow to 112-by-34.
Their first prompt appears on row one before any authored input.
Later absolute cursor output lands on row zero and leaves the observed prompt suffix on row one.
The five successful sequences first record 112-by-34 with the prompt on row zero.
No sampled cell has a concealment flag.
These records place the corruption before accessibility projection; painted pixels remain unverified.

Independent source review identifies a matching Alacritty mechanism, pending replay proof.
Clearing its initial blank viewport creates one blank history line.
Growing the height pulls that line into view and moves the cursor down once.
The previous replay experiments changed width but kept the height fixed at 34.
A new pure test places the observed 80-by-24-to-112-by-34 change before the captured prompt.
Its baseline must fail before any production fix.

The [2222112ab baseline](https://github.com/2389-research/spaceport/actions/runs/44744415622) fails exactly that new startup test.
Its rows reproduce the native blank prefix and stale prompt suffix verbatim.
The six earlier pure cases pass, and Windows skips behind the failed prerequisite.

The fix wraps the existing VTE handler at actual command dispatch.
It preserves the clear and removes manufactured history only when history and viewport were empty beforehand.
The cell predicate uses Alacritty's own color, decoration, wrap, and combining semantics.
All other handler methods delegate to the pinned implementation.
Both ordinary parsing and forced synchronized-update completion use that handler.
This avoids guessing whether a scanned command has executed inside a synchronized update.
New tests cover fragmentation, existing blank history, decorated cells, later output, and both synchronized completion paths.
The probe now runs the full terminal suite before its existing native cases and fixed reproduction series.
Independent read-only review verifies all 71 handler methods without missing or duplicate delegation.
It finds no material issue with history preservation, cell semantics, synchronized dispatch, or the tests.
Compilation, red-to-green proof, and native validation remain pending.

## Implementation

- [x] Add fixed native cases and pure contract tests before implementing their runner.
- [x] Reuse `accessibility_windows::Running`, the private pinned Herdr, and the existing layout scope.
- [x] Observe actual daemon snapshots and UIA state after native input.
- [x] Add the pane-local console producer and attachment prerequisite.
- [x] Extend the existing Polish text case without replacing its failure cleanup check.
- [x] Add a push-only probe guarded by branch, repository, actor, and read-only permissions.
- [x] Run scoped static checks and obtain an independent read-only review.

## Observation and ownership

The JSON table supplies fixed case IDs, scan-code chords, and expected outcomes.
Rust and PowerShell read that same table.
The helper never runs commands taken from terminal output or logs.
Native cases use `SendInput` scan codes under the verified Polish GUI-thread HKL.
Only preparation uses the production control socket or terminal accessibility insertion.

Digit tests prepare nine real tabs, then start each case on a different tab.
They compare ordered workspace positions with exact daemon IDs and native UIA selection.
Pane-cycle tests use three panes, so next and previous have different targets.
Directional tests derive targets from acknowledged right and down splits.
Each preparation request runs once and waits for its observable result.

Passive Herdr clients keep their own focus location.
Fresh passive snapshots seed from the default updated by the GUI's API request.
The tests therefore reopen read-only snapshots and do not compare their projection revisions.
See the pinned [snapshot location](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/src/server/client_shell.rs#L34-L57),
[connection seed](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/src/server/headless.rs#L1865-L1908),
and [GUI request handling](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/src/server/headless/client_views.rs#L865-L914).

The pane producer checks its console handle before publishing a private nonce-bound READY file.
An F8 sentinel must return from `Console.ReadKey(true)` before any disputed key runs.
Each later receipt reports the real console key and modifiers.
The test also requires unchanged focused IDs, ordered scene, HWNDs, fullscreen, and visible chrome.
Microsoft documents [ReadKey](https://learn.microsoft.com/en-us/dotnet/api/system.console.readkey?view=netframework-4.8.1)
and [KeyAvailable](https://learn.microsoft.com/en-us/dotnet/api/system.console.keyavailable?view=netframework-4.8.1).
Actual ConPTY attachment and decoding remain execution prerequisites, not assumed successes.

Font tests measure a nonce-bearing authored line through native UIA glyph rectangles.
The whole marker never appears in the typed command, preventing command-echo matches.
Preparation acknowledges a different rendered font size before each measured chord.
The Windows catalog retains Ctrl-0 reset and Ctrl-Alt-Shift-B status-bar policy.

The existing `Running` owner kills and waits only for its exact children.
The existing layout scope releases held keys and restores its original HKL and profile identities.
Failed key releases remain tracked for cleanup rather than becoming unsupported success receipts.
Settings closes through its exact newly created UIA window after layout restoration.
The second app window remains owned by `Running` until fixture cleanup.

## Static verification, November 20

- `rustfmt --check --edition 2024 --config skip_children=true` passed for the four touched Rust files.
- `git diff --check` passed. Every new file produced no whitespace findings under `git diff --no-index --check`.
  The no-index commands returned 1 because each file differs from `/dev/null`.
- `just check-sorted` passed its ten script tests and the tracked sorted-region scan.
- `just check-file-size` passed for tracked sources. Scoped new-source counts remain below 1,000 lines.
- `actionlint .github/workflows/windows-shortcut-probe.yml` passed after correcting YAML scalar quoting.
- `zizmor --no-online-audits .github/workflows/windows-shortcut-probe.yml` reported no findings.
- Node's JSON parser accepted the fixed table and its bounded, unique physical chords.

The independent read-only review found four material gaps in the first draft.
The correction replaced persistent focus observation, awaited deferred window mapping,
checked pane focus and chrome preservation, and added all four retained arrow cases.
The correction also uses three-pane cycling to distinguish direction.
The implementation owner reviewed those changes directly.
A second reviewer invocation hit the agent thread limit.
The integrating owner should inspect these corrected boundaries before publication.

Root inspected those corrections and the actual helper, console producer, and workflow.
The canonical formatter exposed incorrect nested module lookup under the parent's `#[path]` mount.
Explicit relative paths now wire the matrix, console, and contract-test modules.
The earlier scoped formatter skipped children and did not prove crate-root resolution.

## Verification boundary

The first [exact-branch probe](https://github.com/2389-research/spaceport/actions/runs/44710504007/job/136200677664)
ran head `850f9ae40` and failed.
Formatting, Clippy, and all four Rust contract tests passed.
The extended Polish AltGr case and existing direct chrome case passed natively.
The console fixture never observed its attachment READY receipt.
Three matrix tests reached the layout watchdog's exit 73.
The digit test missed its first prepared-tab acknowledgement.
Those failures do not qualify any of the new matrix outcomes.

### First-run corrections

The preparation client was passive, but attempted `ClientShellPaneInput` anyway.
Pinned Herdr explicitly [rejects input from inactive shell clients](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/src/server/headless.rs#L2256-L2267).
Preparation now inserts trusted text through the actual GUI's terminal ValuePattern.
It acknowledges the real command echo before sending the preparation Enter key.
Measured cases still use only fixed native scan-code chords.
The fixture also acknowledges an actual shell prompt before issuing logical preparation commands.
The first draft only observed the terminal node, which can exist before surface activation opens.

The missing console receipt discarded the only visible program error during sandbox cleanup.
Failure paths now capture bounded, read-only private terminal and log diagnostics before cleanup.
The native helper now reports case, chord, wait, held-key, and cleanup stages.
Each PowerShell observation phase times out before the independent cleanup watchdog.
The exact cause of the three exit-73 failures remains unproven until that instrumented path runs.
The watchdog and cleanup-failure assertions remain enabled.

The font path also called `FindText`, which pinned AccessKit does not implement.
It now finds the authored marker through supported line and character range movement.
This matches the existing terminal fixture's range-based observation approach.
The remote preflight also uses the real PowerShell JSON consumer and console-script parser.

The correction has not compiled or run yet.
Static checks cannot establish that these failures are fixed.
Scoped Rust formatting, whitespace checks, sorted checks, file-size checks,
`actionlint`, and offline `zizmor` passed for this correction.
An independent read-only review found no remaining material findings after checking preparation-error diagnostics.
The reviewer confirmed the supported range APIs, preparation order, PowerShell scope, and cleanup deadlines.
Doctor Biz keeps Mac foreground checks pending.
Root holds local compiler and VM work while the profiling resource question remains unanswered.
Local compiler-based red/green proof did not run; this does not waive any required qualification.
The corrected PowerShell and C# compilation remain pending on the Windows runner.
The intended matrix covers the stock Polish programmer layout on Windows x64 only.
It does not qualify ARM64, every keyboard layout, or the whole original finding before execution.
The Linux/GNOME observer and cases belong to the integrating owner in another worktree.
Do not modify Herdr or substitute mock peers for keyboard qualification.

### PowerShell case enumeration

Source `c9a585ae5` passes compilation, Clippy, and four Rust contract tests.
The [instrumented run](https://github.com/2389-research/spaceport/actions/runs/44714614632) fails its new PowerShell preflight with a chord-count error.
The console native step skips; four matrix tests fail at that same early boundary.
Extended Polish AltGr and existing direct chrome still pass.

Windows PowerShell 5.1 emits a parsed JSON array as one pipeline object.
Wrapping that pipeline in `@(...)` nests the entire case table.
The old selector compares aggregate properties and selects the table as one case.
The new chord-count guard catches that selection before input.
The [PowerShell issue](https://github.com/PowerShell/PowerShell/issues/3424) documents this enumeration behavior.

Both native sender and console producer now assign the parsed array directly.
Both require scalar case IDs; the native sender also requires exact ID equality.
The preflight exercises the actual native consumer for every case and rejects an unknown ID.
It invokes the console producer's real case selection before console attachment or receipt writes.
The producer validates sentinel order and distinct scalar IDs in that same path.
Independent review confirms collection and child-scope return behavior, with no remaining material finding.
Static checks pass; the corrected PowerShell preflight and native outcomes remain pending CI.
This fixes the identified selection defect; it does not yet prove the earlier watchdog or digit outcomes pass.

### Native preparation and assistive search editing

Source `7e2459d67` passes Clippy, four Rust contracts, and both real PowerShell consumer preflights.
The native sender selects all thirty-eight cases; the console producer selects six pane cases.
The [native run](https://github.com/2389-research/spaceport/actions/runs/44716174820) passes digits, punctuation, pane cycling, history, Settings, groups, window cycling, and retained arrows.
Twenty-eight native case receipts pass, with layout restoration.
Three matrix tests pass; the revised-default test stops at Palette preparation.
Extended Polish AltGr and direct chrome also pass.

The console preparation fails because Windows exposes the terminal's ValuePattern as read-only.
Its document value is not a writable editor contract.
Preparation now sends bounded authored ASCII through the existing owned native input scope.
It uses [KEYEVENTF_UNICODE](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput) with a zero virtual key and character scan value.
It tracks each uncertain press before submission and reuses the existing release and disposal paths.
The helper waits for the actual command echo before sending its one Enter chord.
Its exact child receives the existing disposable-runner marker; no guard is relaxed.

The Palette query failure reveals a separate app contract gap.
SearchInput exposes an editable value but registers no SetValue listener.
GPUI's built-in actions handle click, focus, and blur; they do not apply that value.
SearchInput now handles SetValue through its weak entity and existing normalized setter.
Changed notifications still drive the owning search, while duplicate text emits no Changed event.
Masked fields register no listener and reject stale value actions inside the handler too.
The native Resume fixture keeps using SetValue, so it tests the real repaired accessibility route.
Three new unit cases cover Unicode replacement, composition cleanup, empty values, masked/invalid actions, and Changed notifications.
The exact-source probe runs those cases before native execution.
The recorded native failures supply the before-fix regression evidence; new unit and native results remain pending.
Local compiler and Mac foreground checks remain held; no test waiver is inferred.
