<!-- ABOUTME: Records chat accessibility, keyboard ownership, and permission consent checks. -->
<!-- ABOUTME: Separates headless metadata and framed behavior from native assistive-technology qualification. -->

# Chat accessibility

The `2njz` implementation and scoped local verification are complete in `review-squad-fixes`.
The finding remains open for native assistive-technology and keyboard qualification.
Doctor Biz authorized this finding and its validation within umbrella `9s44`.
No commit, push, or issue-tracker write has been performed here.

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
