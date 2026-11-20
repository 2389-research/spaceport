<!-- ABOUTME: Records chat accessibility, keyboard ownership, and permission consent checks. -->
<!-- ABOUTME: Separates headless metadata and framed behavior from native assistive-technology qualification. -->

# Chat accessibility

The `2njz` implementation and scoped local verification are complete in `review-squad-fixes`.
Kata `2njz` closes after native qualification on `99e318867`.

## Native fixture parity, November 19

The original finding requires readable content, named controls, and keyboard permission answers.
It does not require audible speech, release profiling, or an announcement for every streamed message.

The rich-chat preview bypasses normal chat polling.
Its visual preparation does not initialize the normal composer or conversation focus owner.
Consequently, the previous preview omitted the composer's accessible name and Tab stop.
It also omitted the conversation focus owner outside the selection sample.
That preview could not qualify the production keyboard route.

Production polling and previews now share the composer and conversation-focus constructors.
The composer constructor owns its Tab stop and accessible name.
Normal polling retains locale refresh, appearance, completion, and placeholder updates.
The preview retains its synthetic draft and disconnected-agent placeholder.

Both rendered regressions fail first on the previous preview.
The actual field exposes no name, and Tab cannot leave and return to it.
Log: `/tmp/spaceport-9s44-chat-preview-parity-red.log`.
After the correction, all eleven preview and accessibility consumer tests pass without ignored cases.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- chat::preview::tests:: chat::tests::accessibility:: --nocapture`.
Log: `/tmp/spaceport-9s44-chat-preview-parity-green.log`.
Independent review accepts the actual rendered traversal, unchanged draft, and prepaint name assertions.
Both host and Windows GNU lint configurations pass, with and without default features.
Log: `/tmp/spaceport-9s44-chat-preview-parity-lint.log`.
Formatting, file size, sorting, operations, localization, commit, and diff checks pass.
Log: `/tmp/spaceport-9s44-chat-preview-parity-static.log`.
The owner review finds no remaining material issue in this parity correction.

This establishes headless fixture parity, not native qualification of `2njz`.
The next native dataset needs messages, a completed tool, plan steps, permission options, and Agent settings.
Inspect those real rendered controls through Windows UIA and activate permission answers through native keys.
Preview has no linked transport.
Its Session can prove queued Answer intent, while existing framed-wire tests supply separate delivery evidence.
Do not describe a preview answer as a connected-agent delivery receipt.

## Native chat candidate

The `accessibility` preview case installs one embedded protocol fixture and its preserved draft.
The fixture contains readable user/agent messages, a completed tool, three plan states, and two distinct permission requests.
An Other-category setting exposes the real Agent settings button.
The preview uses the shared composer and conversation-focus constructors.

The dataset regression fails before installation uses the fixture.
Log: `/tmp/spaceport-9s44-chat-accessibility-dataset-red.log`.
The corrected producer and existing chat consumers pass thirteen tests, with none ignored.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- chat::preview::tests:: chat::tests::accessibility:: --nocapture`.
Log: `/tmp/spaceport-9s44-chat-accessibility-consumers.log`.
The added rendered-key test reaches both real permission buttons and observes exact queued Answer commands.

The Windows scenario reads the rendered message, tool, plan, permission, and control nodes through native UIA.
It anchors the actual named composer and checks its native Value against the authored draft.
It reaches Agent settings, tool disclosure, and permission buttons through native Tab presses.
Enter opens the settings/disclosure and answers the first permission.
Space answers the second permission.
The settings and tool transitions must occur; expanded tool text must appear and disappear from UIA.

Before each key, the exact private HWND and focused-element ancestry must match.
Immediately before activation, focus must still identify the exact selected button.
An already-held Alt key refuses setup; the temporary Alt press releases in `finally`.
The scenario starts no daemon or agent and reuses exact-child helper cleanup.

The preview probe verifies each real Session queue entry's exact permission/option IDs and unchanged draft.
It locally applies ordinary ACK and PermissionResolved messages to advance the authored sample.
No linked agent receives those commands.
The native receipts therefore prove queued intent; framed-wire tests retain separate delivery evidence.

The scenario also retains a PNG of its owned initial scene through the existing native preview capture path.
Keyboard actions wait for the post-capture readiness receipt.
The parent validates PNG framing and limits the emitted receipt to 512 KiB before publication.
The image contains only the authored private-window scene, not a desktop capture.

Both host and Windows GNU lint configurations pass with and without default features.
The final Windows harness also passes focused lint after its readiness and capture additions.
Logs: `/tmp/spaceport-9s44-chat-accessibility-lint.log`, `/tmp/spaceport-9s44-chat-accessibility-final-windows-lint.log`, and `/tmp/spaceport-9s44-chat-accessibility-harness-final-lint.log`.
Static, commit, and diff checks pass.
Independent review accepts the Session ownership, native contract assertions, isolation, and corrected key guards.
The owner review finds no remaining material issue.
The first actual Windows run fails as recorded below; `2njz` stays open.
Doctor Biz authorized this finding and its validation within umbrella `9s44`.
Doctor Biz also authorized reviewed checkpoint publication in draft PR 125.

## First native execution and diagnostic checkpoint

Source `82eaf8cb9` runs in [CI 44515264034](https://github.com/2389-research/spaceport/actions/runs/44515264034).
The x64 native job passes eight GUI cases, then fails the new chat case.
Its UIA scope reports seven passing cases and one failure, with none ignored.
The actual initial tree contains the required messages, controls, tool, plan, and permission nodes.
The first keyboard action refuses because focused-element ancestry leaves the expected process.
That refusal occurs about sixty-two seconds after the test starts.
The log lacks action-stage timing, the unexpected process ID, and the fixture's own log.
It does not establish the cause or prove a fixture timeout.
No native chat completion or inspected image is claimed.

The diagnostic checkpoint retains action-stage timestamps and process/depth facts on refusal.
It captures bounded fixture output and exact child status before cleanup.
It emits the already-captured, bounded owned-window PNG before keyboard actions.
The exact foreground, ancestry, target, draft, and answer assertions remain unchanged.
The fixture's sixty-second hold cap remains unchanged.
No production behavior changes in this checkpoint.

The ARM all-feature job also times out in the existing activity-visit merge test.
Its five-second disk poll reports `merged visits were not saved`.
The test now drains its existing owned writer outside the view update before reading.
It still requires the stored visit, current visit, and saved width in the real file.
Draining cannot manufacture the merged save if the production resave is missing.
This removes the test's scheduling deadline; it does not establish a production persistence defect.
All three activity-visit tests pass locally with none ignored.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- preferences::tests::activity_last_opened:: --nocapture`.
Log: `/tmp/spaceport-9s44-chat-ci-preferences.log`.
Independent review finds no material issue in the diagnostics or retained persistence assertions.
Both host and Windows GNU lint configurations pass, with and without default features.
Command: `just lint lint-windows`.
Log: `/tmp/spaceport-9s44-chat-ci-diagnostics-lint.log`.
Formatting, size, sorting, operations, localization, commit, and diff checks pass.
Localization validates 2174 messages in 114 files.
Log: `/tmp/spaceport-9s44-chat-ci-diagnostics-static.log`.
The owner review finds no remaining issue in this diagnostic checkpoint.
The next native CI execution remains required.

## Native lifetime correction

Source `98b4063cc` runs in [CI 44518097705](https://github.com/2389-research/spaceport/actions/runs/44518097705).
The Windows ARM all-feature job passes the corrected activity-visit test and its complete scope.
The x64 native chat case reaches Settings activation, Escape dismissal, and native tool expansion.
Its expanded tool appears in the real UIA tree.
Readiness is recorded at `11:58:29.212 UTC`; traversal fails at `11:59:29.501 UTC`.
The failure changes to `E_UNEXPECTED` while traversing the expanded tool scene.

The preview starts its sixty-second hold after readiness.
At that deadline, it requires both permission answers and exits.
The multi-action native scenario has not yet reached those answers.
Source and exact timing strongly indicate fixture expiry caused the provider loss.
This read-path failure did not retain the producer log.
The original focus refusal alone could not establish this cause.

Only the interactive `accessibility` case now accepts a hold up to 180 seconds.
Its native parent requests that budget for the existing sequential helpers.
Other preview cases retain their sixty-second cap and two-second default.
The exact focus guards, per-helper deadlines, answer assertions, and owned-child cleanup remain unchanged.
The native case must still pass; the longer fixture budget is not a qualification receipt.

The retained initial PNG is complete and contains only the authored scene.
The owner inspects its readable messages, tool row, plan states, permission prompts, draft, and controls.
Its size is 94,059 bytes.
SHA-256: `2900be0d1c88701940306f21e8322af96b490d7393e8bc841d66822e2977dca4`.
Private review files: `/tmp/spaceport-9s44-native-chat-98/initial.png` and `manifest.json`.
Permission keyboard completion remains pending, so `2njz` stays open.

The combined chat, preview, and repeat consumer scope passes 28 tests, with none ignored.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- chat::preview::tests:: chat::tests::accessibility:: window::tests::shortcut_repeat:: --nocapture`.
Log: `/tmp/spaceport-9s44-chat-lifetime-repeat-consumers.log`.
Both host and Windows GNU lint configurations pass, with and without default features.
Log: `/tmp/spaceport-9s44-chat-lifetime-window-repeat-lint.log`.
Formatting, size, sorting, operations, localization, commit, and diff checks pass.
Log: `/tmp/spaceport-9s44-chat-lifetime-window-repeat-static.log`.
Independent review accepts the scoped budget correction and keeps native completion pending.
The owner review finds no remaining material issue in this correction.

## Qualified native result on `99e318867`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44520974760/job/135549493119) executes the complete chat scenario successfully.
The exact pass receipt is recorded at `2026-11-19T12:18:27.4335207Z`.
The native tree exposes the authored messages, tool, plan states, permission prompts, and named controls.
Native Tab reaches Agent settings, the disclosure, and both permission buttons.
Settings opens and dismisses; tool output expands and folds through the actual controls.
Enter and Space produce the two exact queued answers and preserve the draft.
The native tree then removes both resolved prompts.
The probe verifies the real Session queue; it does not claim a linked agent received those commands.
Existing framed-wire tests retain the separate delivery evidence described above.

Command: `cargo test --locked -p sheprd-gpui --features integration-test --test accessibility_windows -- --ignored --nocapture --test-threads=1`.
Its UIA scope reports eight passing cases and one failure, with none ignored.
The later New Window qualifier fails on its incorrect CLI argument order before sending shortcut keys.
That unrelated failure keeps the PR's CI red; it does not invalidate this chat case's pass.

The exact-head initial PNG is complete and inspected by the owner and independent reviewer.
It contains the authored messages, tool row, plan, prompts, draft, and controls without clipping those areas.
Its size is 94,052 bytes.
SHA-256: `713d862ac79ddb59efff7475a528a7754b4311b5612bb7f30a844f0a319dc556`.
Private review files: `/tmp/spaceport-9s44-native-chat-99/initial.png` and `manifest.json`.
Independent review accepts closure against the original issue at revision one.
No audible-speech, native connected-agent, or performance qualification is claimed.

Kata closure: event `9789`, UID `01M7SWNA1CGD5BH6X4MM7YTAS6`, at `2026-11-19T12:31:47.244Z`.
Fresh issue and parent reads confirm closure, with 44 children closed and nine open.
The parent remains open.
The closure ledger retains one unique row for this finding.

## Scope

Expose named header and composer controls.
Expose readable message, tool, plan, and permission semantics.
Reach permission options with Tab and activate them with deliberate keyboard events.
Preserve composer edits, IME, pending-paste refusal, and exact permission/session ownership.
Keep the existing double-click guard and deny-by-default behavior.

## Verified regressions

The real framed-chat fixture could not reach an answer with Tab.
Rendering permission ID `u64::MAX` overflowed the previous control-ID multiplication.
The actual button, entry, and plan view producers exposed no semantic role.
All five baseline cases failed at runtime for those expected reasons.
Log: `/tmp/spaceport-qnz9/2njz-runtime-red.log`.
The initial corrected scope passed two framed keyboard cases and three metadata cases.
Log: `/tmp/spaceport-qnz9/2njz-first-consumers.log`.

A further actual-key test held Enter across a new session with reused permission IDs.
Releasing that old key answered the replacement session without a fresh press.
Log: `/tmp/spaceport-qnz9/2njz-session-consent-red.log`.
Control identity now includes exact Target, session epoch, and connection generation.
The corrected held-key case passes in `/tmp/spaceport-qnz9/2njz-controls-red2.log`.
That run also passes the pointer-paste and IME controls.

## Current implementation

Mounted entries expose semantic article nodes with their existing prepared text segments.
The accessibility callback shares the existing entry projection through `Rc`.
It does not parse the transcript again or store another transcript model.
Tool names include the existing derived tool state.
Plan steps expose their text and state as list items.
Permission groups have names and polite announcements.
Answer controls retain the first-click guard and check their captured Target and session epoch.

Chat controls use GPUI's existing keyboard-click implementation.
Tab traversal remains inside the conversation and preserves plain-Tab slash completion.
Composer-specific Enter, history, and completion routes require actual composer focus.
Chat root ownership contains focused descendants so their keys cannot reach hidden terminals.
The shared focus-ring builder now serves chrome, Settings, and chat controls.
Settings retains its disabled-control and swatch-background contrast policies.

## Final local verification

The named composer and keyboard disclosure first fail in `/tmp/spaceport-qnz9/2njz-controls-red2.log`.
The earlier composer metadata fixture rendered outside GPUI's drawing phase and was invalid.
The corrected fixture inspects the actual field during a private canvas prepaint.
The field name now uses its existing accessibility label property.
Disclosures now share keyboard clicks and screen-reader Expand/Collapse actions.
The pointer Send fixture already preserved pending-paste refusal; no pointer workaround was required.

The expanded corrected scopes pass: chat542, multiline19, Settings267, chrome4, and Core pane-owner12.
Log: `/tmp/spaceport-qnz9/2njz-complete-consumers.log`.
The chat scope includes prepared text-node projection, folding, shared ownership, and consent checks.
Root independently approved the current permission, focus, and projection safety delta.

Root selected the existing Agent Settings gear for a named Tab stop.
Its existing menu remains unchanged, with keyboard activation and Escape coverage.
The gear first fails to receive keyboard focus in `/tmp/spaceport-qnz9/2njz-gear-red.log`.
Its corrected scope passes all eight behavior cases in `/tmp/spaceport-qnz9/2njz-gear-green.log`.
The final expanded chat scope passes 543 tests in `/tmp/spaceport-qnz9/2njz-final-consumers.log`.

Direct review found nested sub-agent step rows outside the shared entry renderer.
Their visible title and state therefore also needed semantic nodes.
The actual step-row producer first exposes no role in `/tmp/spaceport-qnz9/2njz-nested-step-red.log`.
The rows now form a named list with named state/title items.
The corrected full chat run passes 544 tests with no failures or ignores.
Log: `/tmp/spaceport-qnz9/2njz-nested-final-chat.log`.

Formatting, file-size, sorting, localization, and diff checks pass.
Localization validates 2174 messages in 114 files.
Whole GPUI host and Windows GNU all-target, all-feature Clippy pass with `-D warnings`.
Logs: `/tmp/spaceport-qnz9/2njz-host-clippy-source-stable.log` and `2njz-windows-clippy-source-stable.log` in the same directory.
Those gates include the nested-step annotation and the completed DeviceTheme source boundary.
The earlier failed host gate stopped on a concurrent DeviceTheme test-fixture export.
No lint allowance was added for that fixture.
Root independently approved the final nested-step metadata and the complete consent/keyboard/IME safety delta.
The review found no material remaining defect in those paths.
The final format and diff checks also pass after the required source comments.

## Exact aggregate lint commands

Both commands ran from `/Users/harper/Public/src/2389/spaceport/.worktrees/review-squad-fixes`.
Both used the shared target, disabled incremental compilation, and selected Rust 1.99.0.
The host selected the validated pinned Sparkle SDK at `/private/tmp/spaceport-cli-sdk.7S7nty/sdk`.
The commands and log redirections below are the exact successful invocations.
The recorded tool sessions both returned exit code `0`.
The successful Clippy checks were not rerun for this evidence update.

Host command, exit `0`, finished in 25.90 seconds:

```sh
/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock env CARGO_INCREMENTAL=0 RUSTUP_TOOLCHAIN=1.99.0 CARGO_TARGET_DIR=/Users/harper/Public/src/2389/spaceport/.worktrees/upstream-fixes/target SPARKLE_FRAMEWORK_PATH=/private/tmp/spaceport-cli-sdk.7S7nty/sdk cargo clippy -p sheprd-gpui --all-targets --all-features --locked -- -D warnings > /tmp/spaceport-qnz9/2njz-host-clippy-source-stable.log 2>&1
```

Windows GNU cross-check, exit `0`, finished in 21.81 seconds:

```sh
/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock env CARGO_INCREMENTAL=0 RUSTUP_TOOLCHAIN=1.99.0 CARGO_TARGET_DIR=/Users/harper/Public/src/2389/spaceport/.worktrees/upstream-fixes/target CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar cargo clippy -p sheprd-gpui --all-targets --all-features --target x86_64-pc-windows-gnu --locked -- -D warnings > /tmp/spaceport-qnz9/2njz-windows-clippy-source-stable.log 2>&1
```

The host reports the existing `block 0.1.6` future-compatibility notice.
The Windows command compiles the GNU target; it does not execute native Windows checks.

## Native traversal correction on November 19

Source `8a4beb1ec` passes all eight x64 native GUI cases, including the separate palette qualifier.
Its later UIA scope passes eleven cases and fails the chat Allow action at its 20-second parent deadline.
The last receipt shows nine forward Tab steps after anchoring the actual composer.
This is not the earlier preview-lifetime expiry; the owned preview remains alive.

The captured native tree places tool and permission buttons before the composer.
Agent settings follows the composer.
The native script now traverses backward to the preceding controls and forward to Agent settings.
The correction preserves exact-window guards, deliberate activation, the iteration bound, and the existing deadline.
It adds no retry or synthetic permission answer.
Actual native execution of the shorter traversal remains pending.
Independent review confirms direction against the actual render order and finds no material issue.
The focused Windows native harness lint, static checks, and diff check pass.
Logs: `/tmp/spaceport-9s44-native-traversal-windows-lint.log` and `native-traversal-static.log` under the same prefix.
Earlier complete chat qualification on `99e318867` and `5c696d3c8` remains recorded separately.
The shorter route passes natively on `7607bde87` at `2026-11-19T14:03:33.1593775Z`.
Its [x64 job](https://github.com/2389-research/spaceport/actions/runs/44537828031/job/135608225364) later fails the separate Settings log-sharing check.
The UIA scope records twelve passing cases, one failure, and none ignored.

## Native qualification still pending

Pinned GPUI's TestPlatform cannot activate the complete accessibility tree through its public API.
Metadata tests therefore inspect actual view producers and synthetic-node producers directly.
Headless key tests use private framed fixtures and actual GPUI controls.
No personal transcript, clipboard, or daemon supplies test data.
Native screen readers, physical keyboard delivery, platform visuals, and performance remain pending.

## Redundant action inspection on `3634e5039`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44671802770/job/136074462029) times out during its Settings action.
The GUI remains alive; the capture reaches the first Tab stage near the helper's twenty-second parent deadline.
The action starts around 04:12:32 UTC and reaches its initial draft check at 04:12:48.666 UTC.
Its first Tab stage begins at 04:12:50.327 UTC; the parent reports timeout at 04:12:51.983 UTC.
These timings establish preparation overhead, without proving which individual UIA call caused the delay.

Each action previously repeated the full diagnostic tree walk already verified by its parent.
The action now requests discovery without that redundant dump.
It still finds the exact owned window, composer, draft, target, and native focus ancestry.
The parent retains complete tree assertions before input and after each relevant scene change.
Native Tab, Enter, Space, deliberate activation, twenty-second deadlines, and all action assertions remain unchanged.
Independent review finds no material loss of targeting or verification.
This candidate initially lacks Windows qualification; the following run records that result.

## Actual chat-action rerun on `3a0bcb00d`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44677192575/job/136091920328) passes all 25 ordinary UIA cases.
The chat case retains named content, actual Tab/Enter/Space input, deliberate permission answers, and the preserved draft.
All five action invocations finish within their original twenty-second deadlines.
The receipt appears at `2026-11-20T05:02:18.5851651Z`; the Rust test passes afterward.
See the [scoped receipt](review-squad/windows-native/qualification-3a0bcb00d.json).
The redundant diagnostic-walk correction now has actual Windows qualification.
This establishes fixture completion, without claiming a production chat-performance improvement.
The same job separately fails Core resize geometry; its overall result remains red.
