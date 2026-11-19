# Clipboard reads and async destination ownership

Verified on 2026-11-17 in `review-squad-fixes`.
The reader checkpoint below is historical; later async verification follows it.
The implementation, independent review, and scoped local gates are complete.
The complete `d378` finding remains open for native and platform qualification.
Doctor Biz requested a stopping point before MultilineInput and Core changes.

## Implemented boundary

The native reader now accepts one `Mode`: `Normal`, `Text`, or `Image`.
Text probes skip copied files and never acquire images when text is absent.
Normal paste retains file, text, then image priority.
Image paste retains image-only acquisition.
The existing Gen1 background preparation caller now selects the typed mode.
The macOS helper accepts the matching `normal`, `text`, and `image` arguments.
Linux and Windows use the same mode at acquisition boundaries.
No dependency was added.

The shared reader retains its existing text and image byte bounds.
Those bounds remain 2 MiB for text and 128 MiB for image input.
The macOS helper retains its deadline, bounded output, and exact-child cleanup.
Windows retains its native worker lease until the worker exits, including after caller timeout.
Linux retains its whole-acquisition deadline and selection policy.

## Regression evidence

Three new tests failed at runtime before the fix.
The generic text probe incorrectly returned image content.
A real private AppKit board incorrectly returned a copied file instead of its published text.
An image-only private board incorrectly returned an image for a text probe.
Red log: `/tmp/spaceport-qnz9/d378-native-mode-red.log`.

The focused reader group then passed all 33 tests.
Green summary: `/tmp/spaceport-qnz9/d378-reader-green.log`.
Full output: `/tmp/spaceport-qnz9/d378-reader/0.log`.
Build output: `/tmp/spaceport-qnz9/d378-reader-build.log`.

The group includes actual private named AppKit pasteboards and isolated helper processes.
It also covers output bounds, deadlines, malformed helper responses, and exact-child cleanup.
These checks never read the general or personal clipboard.
The native fixture emits AppKit sandbox-extension diagnostics; no test failed.
Linux helper-policy tests on macOS do not qualify a live Linux desktop.
Windows native behavior was not run for this checkpoint.

The exact check script is `/tmp/spaceport-qnz9/check-clipboard-reader.sh`.
It builds the GPUI test binary with:

```sh
cargo test -p sheprd-gpui --bin spaceport --no-run --message-format=json --locked
```

It runs that artifact with the filter `window::clipboard::`.
The throwaway runner reuses `crates/test-support/sandbox.rs` for private process state.
The runner owns and reaps only its exact child, with a bounded wait.

Every Cargo command uses `/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock`.
The environment sets `CARGO_INCREMENTAL=0` and `RUSTUP_TOOLCHAIN=1.99.0`.
`CARGO_TARGET_DIR` points to the sibling `upstream-fixes/target` directory.

## Reader checkpoint remaining work (historical)

At this checkpoint, chat and MultilineInput still used synchronous App clipboard reads.
Core paste and its plain-CtrlV probe also used synchronous App clipboard reads.
Async clipboard routing and prepared Core input slots had not been implemented.
The checkpoint did not qualify async cancellation, focus fences, ordering, or releases.
The complete approved contract and exact resume point live in `/tmp/spaceport-qnz9/d378-plan.md`.

## Checkpoint verification and review

Fresh-eyes review covered format priority, byte limits, helper ownership, and the existing Gen1 caller.
That direct review found no material reader findings.
Independent review of the completed async change remains required after implementation.

`cargo fmt --check --all` passed under the shared Cargo lock.
`git diff --check` passed.
`just check-file-size check-sorted` passed, including all ten sort-checker tests.
Structure log: `/tmp/spaceport-qnz9/d378-structure.log`.

The shared GPUI Clippy gate failed with ten test-only unwrap diagnostics in sibling scopes.
The files were `close_modal/sequence/tests.rs` and `notice/tests/history.rs`.
No clipboard diagnostic appeared in that run.
Root received the exact errors; this checkpoint does not claim the gate passed.
Command: `cargo clippy -p sheprd-gpui --bin spaceport --tests --locked -- -D warnings`.
Log: `/tmp/spaceport-qnz9/d378-reader-clippy.log`, exit 101.

Completed upload evidence is separate: [bounded SSH uploads](review-chat-uploads.md).

## Approved behavior on resume

Composer edits and IME must remain immediate; changing them must cancel a pending paste.
Owner submit, history, and completion actions must visibly refuse during preparation.
Core must reserve bounded FIFO input before the read and continue draining output.
Plain CtrlV must preserve fallback and release ordering when text is absent.
Read permits must remain held until the native worker actually exits.
Use existing private native fixtures for further end-to-end checks.

Root's reviewed composer bounds and shaping cache remain separate completed work.
The async clipboard issue stays open until implementation, focused tests, and independent review finish.
Native keyboard, visual, and release performance checks remain pending.

## Async routing implementation

Chat, standalone multiline fields, and Core now acquire clipboard data outside the UI context.
Production readers never call GPUI's synchronous App clipboard API.
The test-only in-memory clipboard remains separate from native acquisition fixtures.
One per-App budget permits at most four active read tasks across native paste routes.
Each permit stays inside its acquisition closure until that closure exits.
Windows additionally retains its existing single native-reader lease after caller timeout.

Composer tickets capture field revision, selection, and exact entity ownership.
Native edits, IME, selection movement, resets, and published blur events cancel the ticket.
Chat adds exact Target and cursor-epoch checks before applying text or attachments.
Ordinary sequence advancement preserves the ticket.
Send, history, and completion refuse immediately during preparation and never submit later.
Accepted replacement still passes the existing 256 KiB UTF-8 prompt limit.

Core reserves one bounded command slot before reading.
Its worker keeps draining terminal output while that slot waits.
Empty plain-CtrlV probes preserve the original key at that slot.
Conditional releases preserve both early-release and late-release ordering.
Consumed paste presses produce no terminal release.

Independent root review found that the first cancellation fence flushed unrelated accepted input.
The earlier plan had incorrectly requested dropping following input on cancellation.
That policy conflicted with the repository's cancellation invariant.
The correction gives each clipboard slot its own cancellation flag.
Cancelling a read skips that slot and preserves ordinary input, resize, and focus commands.
The deterministic real-worker regression first lost both queued `before` and `after` input.
Red log: `/tmp/spaceport-qnz9/d378-input-preservation-red.log`.

Pinned GPUI publishes blur subscriptions after drawing an active window's focus transition.
Its internal focus generation has no public getter.
The focus-return fixture activates its private TestWindow and draws the transition before returning focus.
That corrected test passes in `/tmp/spaceport-qnz9/d378-active-focus.log`.
This does not prove an away-and-back focus change that GPUI never publishes between frames.

## Corrected verification and review

The corrected Core worker scope passes all ten tests.
It covers queued input on both sides of a cancelled slot, output drain, capacity, and shutdown.
Log: `/tmp/spaceport-qnz9/d378-input-preservation-green.log`.
The Core view tests additionally exercise visible read errors and conditional key releases.
The framed chat tests exercise Send refusal, epoch replacement, target replacement, and sequence advancement.

Root independently reviewed the corrected slot, capacity, field, chat, and release ownership paths.
That review approved the correction with no further material findings.
The corrected expanded consumers pass with no failures or ignored tests:

- Native reader and admission: 34 tests.
- Multiline editing, prompt limits, and paste: 19 tests.
- Core worker lifecycle, backpressure, and preparation: 10 tests.
- Core view completion, errors, and releases: three tests.
- Core paste shortcut classification: three tests.
- Chat reducers, framed conversations, composer routing, and attachments: 531 tests.
- Existing Gen1 image preparation: ten tests.

Summary: `/tmp/spaceport-qnz9/d378-corrected-consumers.log`.
Individual scope logs live in `/tmp/spaceport-qnz9/d378-corrected-consumers/`.
The earlier expanded run passed every scope except its inactive focus-return fixture.
Those older results do not qualify the corrected worker source.

Current file-size, sorted-region, localization, and diff checks pass.
Localization validates 2162 messages in 114 files.
Log: `/tmp/spaceport-qnz9/d378-corrected-structure.log`.
The earlier whole-worktree format run found in-flight edits owned by other tasks.
Clipboard-owned files were formatted with the pinned Rust formatter.
The final aggregate formatting check passes.
Whole GPUI host and Windows GNU all-target, all-feature Clippy pass with `-D warnings`.
Logs: `/tmp/spaceport-qnz9/2njz-host-clippy-source-stable.log` and `2njz-windows-clippy-source-stable.log` in the same directory.
These gates include the corrected worker and later chat accessibility changes.
The later complete chat scope passes 544 tests in `/tmp/spaceport-qnz9/2njz-nested-final-chat.log`.
Earlier Settings lint results predate the worker cancellation correction and remain historical.

No full workspace CI run, native Windows execution, or Linux desktop acquisition is claimed.
Native keyboard, visual, accessibility, and release performance qualification remain pending.
No personal clipboard, production daemon, commit, push, or Kata write was used for this work.

## Native route research, November 19

The original `d378` scope has no release-profile prerequisite; `gg9v` owns that work.
Native clipboard qualification needs an exclusive disposable Windows session and an empty-only preflight.
Windows places the clipboard in the [window station](https://learn.microsoft.com/en-us/windows/win32/winstation/window-stations).
Private HOME or a second desktop in the same station does not isolate it.
Refuse a nonempty clipboard unchanged, without reading its data.

An owned broker can publish only authored text after a successful empty preflight under [OpenClipboard](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-openclipboard).
It needs an owned HWND and correct allocation transfer under [SetClipboardData](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setclipboarddata).
Exclude synthetic text from history and cloud sync through the documented [clipboard formats](https://learn.microsoft.com/en-gb/windows/desktop/dataxchg/clipboard-formats).
Clean only if clipboard ownership and sequence still match the fixture; preserve any foreign replacement.
The broker must outlive GUI failures and release resources before the parent propagates an assertion.

Exercise the real chat composer and a standalone MultilineInput separately.
Chat capture does not exercise the standalone field's Mode::Text acquisition path.
Verify normalized Unicode, the exact 262144-byte limit, overflow refusal, and the separate 2 MiB acquisition bound.
Generate payloads inside the fixture; log lengths, hashes, and receipts rather than clipboard text.
For cancellation, hold the already-published synthetic clipboard without mutation, observe pending acquisition, then type or blur.
Prewarm helpers and retain the production three-second deadline.
Release the lock and verify no stale insertion, then verify a subsequent paste recovers the reader lease.

Existing Core worker and view tests qualify ordering and delivery against their explicit test backend.
That backend is not a live native Core runtime.
Keep shared native-reader evidence separate from Core contract evidence unless a private runtime is explicitly selected.
Do not invent a new live-runtime prerequisite for the original issue.
No clipboard access, write, or native launch occurred during this read-only research.

## Native owner candidate

The Windows fixture now has an empty-only native publication owner and two native safety regressions.
The helper creates an owned hidden HWND and eagerly publishes only authored Unicode data.
It publishes the documented history/cloud exclusion before the text, without changing OS preferences.
Admission counts formats under the clipboard lock and refuses nonempty state unchanged.
Reads and clears require the exact HWND and publication sequence while locked.
Successful native allocation transfers belong to Windows; failed transfers retain cleanup ownership.

The first native case checks Unicode and authored 262144/2097153-byte payloads through the real clipboard APIs.
It checks nonempty refusal, a stale sequence with the same HWND, and preservation of a successor's publication.
It also checks cleanup after a deliberate managed exception.
These are helper ownership checks, not application paste or acquisition-limit results.

The second native case deliberately fails the Rust parent after authored publication.
The parent retains the owner, requests cleanup, and waits for the exact process to exit.
An independent native guardian starts before publication and prevents future publication after cancellation.
It clears only an exact-owned publication and writes a fresh safe-cleanup receipt before exit.
The native startup handshake separates compilation from the publication deadline.
A fresh empty-only admission verifies the guardian's actual removal.

Review found that hard termination could bypass managed `finally` cleanup.
It also found an unconditional guardian shutdown after failed cleanup.
Both paths now retain the HWND, lease, owner process, and private files until cleanup is proved.
Persistent external contention is reported as pending cleanup.
It cannot authorize termination or deletion without a safe receipt or exact terminal process evidence.
Independent correction review finds no remaining material ownership or privacy issue.

Windows native-harness lint and static checks pass.
C# compilation and both actual native safety regressions remain pending Windows CI.
No local clipboard access or publication occurred.
Real chat and standalone multiline paste, cancellation, and reader recovery remain the next qualification step.
Kata `d378` remains open.

## Native owner qualification and application candidate

Source `940749d29` compiles and passes both native owner safety cases on the disposable Windows runner.
Its full CI has eighteen successful jobs and one unconfigured skip.
Source `64970ccce` repeats those passes in the [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44551798252/job/135657584169).
The safe-cleanup receipt, expected guardian exit, and fresh empty-only admission qualify parent-failure cleanup.
These receipts do not qualify application paste.

The next candidate drives real native Ctrl-V through separate chat and standalone field windows.
The chat fixture retains its production capture owner and composer constructor.
The standalone fixture mounts the actual MultilineInput without chat capture.
Its headless wiring test verifies real field keyboard routing and normalized text.
That headless test uses GPUI's test clipboard and does not qualify native acquisition.

The prewarmed native helper publishes authored Unicode with CRLF and checks the normalized field value.
It accepts exactly 262144 UTF-8 bytes, including a multibyte character.
It rejects 262145-byte prompts and 2097153-byte acquisition while preserving the draft.
The prompt overflow appends one byte to the multibyte exact-limit payload, distinguishing UTF-8 from UTF-16 counts.
It checks distinct error tags after clearing each prior error through real edits.
The read-only observer classifies the field's actual error by exact localized prompt and acquisition diagnostics.
The helper logs receipts and lengths, never field or clipboard contents.

Cancellation holds the exact-owned clipboard without modifying its data.
An independent STA holder opens and closes it on that same thread.
It releases after at most two seconds even if GUI automation blocks.
No shared gate remains locked during the hold or automation callback.
An expired hold fails the responsiveness check.
Safety cases separately exercise callback failure and independent hold expiry.

The app observer only reports actual pending state, native reader lease, UTF-8 length, and error kind.
Typing and blur must cancel pending paste while the native reader remains active and the hold still exists.
After release, the helper requires reader completion, unchanged authored draft, and a successful fresh paste.
The production three-second reader deadline is unchanged.
The guardian retains the existing cleanup contract, with a bounded ninety-second application scenario.
Parent failures reuse the same owner retention and safe-cleanup receipt checks.

Thirty-two headless wiring and clipboard consumer tests pass, with none ignored.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- chat::preview::clipboard::tests:: chat::preview::tests:: window::clipboard::tests:: multiline_input::tests:: --nocapture`.
Log: `/tmp/spaceport-9s44-clipboard-native-consumers-fixed.log`.
Both host and Windows GNU lint configurations and static checks pass.
Independent review finds no material ownership, privacy, cancellation-evidence, or parent-cleanup issue.
C# compilation, native helper execution, and the application scenarios remain pending Windows CI.
No local clipboard operation ran, and `d378` remains open.

## First native application result on `47fd690d6`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44556877754/job/135675531922) passes eight GUI cases and sixteen UIA cases, with two UIA failures.
Both application cases pass real Unicode/CRLF paste and exact 262144-byte acceptance.
Both then fail the fixture's assertion that HelpText must become empty after editing.
Neither reaches prompt overflow, acquisition overflow, or cancellation qualification.
Both emit a safe clipboard cleanup receipt before reporting failure.

The native C# helper compiles and passes the strengthened ownership regressions.
Callback failure releases the holder, and a delayed callback cannot extend the independent two-second hold.
The exact-owned publication survives both cases and remains readable afterward.
Guardian cleanup and fresh empty-only admission also pass again after deliberate parent failure.

Pinned AccessKit Windows maps `UIA_HelpTextPropertyId` to the placeholder.
It maps the field's description to `UIA_FullDescriptionPropertyId` instead.
The failed reset read the normal placeholder rather than the error.
[Microsoft's property contract](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-automation-element-propids) distinguishes those properties.

The corrected fixture observes the actual field error through its existing read-only state probe.
It compares exact localized diagnostics and emits only none, prompt, acquisition, or other.
Native keys still clear each prior error, and native UIA Value still verifies preserved draft contents.
The probe adds no paste, acquisition, or state-changing path.
A headless test exercises the real prompt-error producer, typed acquisition-error boundary, and clearing.
Corrected actual native execution remains pending; `d378` stays open.
The corrected full consumer scope passes thirty-three tests, with none ignored.
Independent correction review finds no material issue in the read-only observation or retained native checks.

## Completed original scope on `1ad093a96`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44560521338/job/135688647754) passes both complete application scenarios.
Chat passes at `2026-11-19T16:06:17.1168932Z`; the standalone field passes at `16:06:20.6302003Z`.
Each passes native Unicode/CRLF, exact 262144-byte acceptance, multibyte 262145-byte prompt rejection, and 2097153-byte acquisition rejection.
Each preserves the draft on refusal, cancels typing and blur before release, rejects stale insertion, and recovers a fresh reader.
Native owner, independent hold expiry, callback-failure release, and guardian cleanup checks also pass.
Sanitized receipts remain in `/tmp/spaceport-9s44-clipboard-native-1ad-receipts.json`.

The full native job has eighteen passing UIA cases and one separate terminal-caret qualifier failure, with none ignored.
Its eight GUI cases pass; the overall job remains failed.
That unrelated fixture failure does not negate these completed clipboard cases.

The fresh original issue covers background acquisition, pre-replacement caps, and reusable composer shaping.
Current source routes chat, standalone, and Core acquisition through bounded background workers.
Core uses the same native Mode::Text acquisition now exercised by the standalone fixture.
Its separate view and worker tests cover reservations, FIFO input, output draining, cancellation, errors, and retained accepted input.
The current thirty-three consumer cases include actual shaping-count and invalidation checks.
Independent source and original-scope audit finds no missing requirement blocking `d378` closure.
A live Core clipboard session and a working-session release profile are not original prerequisites.
Native shared-reader evidence and Core contract evidence remain distinct.
