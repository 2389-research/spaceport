<!-- ABOUTME: Records chat accessibility, keyboard ownership, and permission consent checks. -->
<!-- ABOUTME: Separates headless metadata and framed behavior from native assistive-technology qualification. -->

# Chat accessibility

The `2njz` implementation and scoped local verification are complete in `review-squad-fixes`.
The finding remains open for native assistive-technology and keyboard qualification.

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

## Native qualification still pending

Pinned GPUI's TestPlatform cannot activate the complete accessibility tree through its public API.
Metadata tests therefore inspect actual view producers and synthetic-node producers directly.
Headless key tests use private framed fixtures and actual GPUI controls.
No personal transcript, clipboard, or daemon supplies test data.
Native screen readers, physical keyboard delivery, platform visuals, and performance remain pending.
