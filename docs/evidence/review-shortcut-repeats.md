<!-- ABOUTME: Records raw keyboard repeat routing and its regression evidence. -->
<!-- ABOUTME: Tracks existing shortcut compatibility checks and remaining native verification. -->

# Shortcut repeats

Verified on 2026-11-17 for the held-shortcut report (`agbt`).
Doctor Biz authorized implementation and validation in the aggregate worktree.
This work does not commit, publish, or update the issue tracker.

## Acceptance

Window creation, tab creation, splits, close actions, and custom commands run once per press.
A fresh press must still run the action again.
Holding the prefix must keep its chord armed.
Consumed chord keys must not leak their held tail into terminal or text input.
Terminal input, font changes, navigation, and pane resizing must retain repeats.
Menu actions must remain available independently of keyboard events.

## Root cause and routing

GPUI resolves action bindings before raw key capture.
Its keystroke interceptor does not carry the raw event's `is_held` flag.
The previous prefix interceptor could therefore consume the chord on a prefix repeat.
Catalog actions and custom commands could also run again after each daemon acknowledgement.

The main window now owns one raw shortcut scope.
Catalog bindings retain menu labels and work outside the main window's context.
The scope reads the window's own resolved keymap.
It preserves configured bindings, custom-command precedence, and shifted-key restoration.
The scope also owns prefix chords and resize mode.
One key map records consumed, repeatable, and pass-through tails until release.
It distinguishes direct shortcut ownership from explicit modal ownership.
Character-preferred events yield to text fields before direct shortcuts or captured direct repeats.
An explicit prefix or resize mode keeps ownership of its modal input.
Window deactivation and keymap replacement clear that ownership.
Programmatic and menu actions still call the existing command dispatcher.
New catalog commands default to one-shot behavior.
The integrating owner's confirmed workspace-close route therefore keeps that policy.

The lifecycle fixture intentionally omits the terminal canvas.
It now mounts the same production shortcut scope.
Its function-key, shifted-key, and custom-command scenarios explicitly focus that scope.
The tests retain their existing manual semantic input delivery after prefix pass-through.

## Regression evidence

The raw-event baseline failed three scenarios.
A held sidebar toggle toggled again.
A held prefix disarmed its chord.
A held New Window shortcut opened another real GPUI window.
The font-repeat preservation case passed before the change.
Log: `/tmp/spaceport-qnz9/shortcut-repeat-red.log`.

All nine expanded shortcut contracts passed after the change.
They include real framed tab, split, and custom-command requests.
Ordered input markers prove held requests were not queued after acknowledgement.
They also cover navigation and resize requests, terminal keys, font repeats,
menu actions, actual window creation, and chord release followed by text input.
Log: `/tmp/spaceport-qnz9/shortcut-repeat-contracts-green.log`.

The initial extended fixtures lacked active surfaces, current geometry, and response correlation IDs.
They now use existing endpoint attachment and input-flush helpers.
The rendered fixture acknowledges the viewport before dispatch.
The text-input scenario uses GPUI's public IME simulation.
No tests were ignored or retried to mask failures.

The integrating owner's broader endpoint run exposed six empty-fixture keyboard failures.
The shared scope and explicit focus updates address those callers.
The final focused run passed all four shifted-key endpoint cases and nine prefix cases.
It also passed ten default-build shortcut tests and thirteen integration-feature shortcut tests.
The function-key endpoint scope passed two tests.
The keyboard endpoint scope passed three tests.
The window key-action scope passed two tests.
Log: `/tmp/spaceport-qnz9/shortcut-final-green.log`.

Independent review found a missing character-preference guard in the raw route.
Two real focused-field regressions failed before that fix.
They cover both a fresh direct shortcut and a captured direct repeat.
The explicit-prefix preservation case passed before and after the fix.
Log: `/tmp/spaceport-qnz9/shortcut-text-preference-red.log`.
All three focused text cases now pass.
The integrating owner approved the direct/modal ownership fix.

The old isolated fixtures had assumed catalog shortcuts required global GPUI registration.
The raw owner now follows the window's actual resolved keymap.
The chord-only fixture therefore uses a key without a catalog default.
The shifted named-key test now asserts the catalog effect and absence of custom wire requests.
It retains pane-key forwarding and custom-command precedence cases.

The broader endpoint batch also exposed unrelated retry and remote-drop fixture failures.
Their owners retain attribution and correction of those cases.
Final full endpoint verification remains pending.
Log: `/tmp/spaceport-qnz9/shortcut-existing-contracts.log`.

File-size, sorted-region, and diff checks passed before the final batch.
The modal-priority regression then reproduced a captured font repeat acting during resize mode.
Log: `/tmp/spaceport-qnz9/shortcut-modal-priority-red.log`.
The raw route now checks active modal ownership before recorded direct repeats.
It still swallows a consumed mode-trigger tail before that trigger can end its mode.
The integrating owner independently approved this final ordering.

All-target, all-feature Clippy passed after the final change with the validated pinned Sparkle SDK.
The final batch exited successfully.
Formatting, sorted-region, file-size, and diff checks passed for the shortcut changes.
The aggregate formatting check later found only concurrent workspace-close test edits.
Their integrating owner retains that formatting work.
Full CI and actual native keyboard/performance checks remain deferred.
Headless dispatch and scripted peers do not prove OS key-repeat delivery.

## Platform mapper follow-up

Source review found that raw direct matching bypassed GPUI's platform keyboard mapper.
Windows maps shifted digit bindings even when key equivalents are disabled.
The new raw route had compared only canonical key names.
Two regressions failed before the correction.
A representative mapped shifted digit missed its command.
A configured shifted digit also lost its key to a daemon custom symbol.
The canonical named-symbol precedence case already passed.
Log: `/tmp/spaceport-qnz9/shortcut-platform-mapper-red.log`.

Direct matching now delegates to GPUI's public `KeybindingKeystroke::new_with_mapper`.
It uses the same mapper and key-equivalents setting as the existing action bindings.
The match retains typed catalog or configured ownership from the existing shortcut entry.
A configured mapped key therefore keeps its command before custom-command lookup.
A catalog default can still yield to a daemon custom command.
Canonical named bindings retain priority over another binding's mapped alias.
No default shortcuts changed.
All three focused mapper regressions passed.
Log: `/tmp/spaceport-qnz9/shortcut-platform-mapper-green.log`.
The fixture exercises GPUI's real matching types with a representative mapper function.
It does not establish native Windows keyboard behavior.
The final compatibility batch exited successfully.
It passed 58 keymap tests, 12 prefix tests, 13 integration repeat tests, and four shifted-key endpoint tests.
All-target, all-feature GPUI Clippy passed with `-D warnings` and the validated pinned Sparkle SDK.
Log: `/tmp/spaceport-qnz9/shortcut-platform-mapper-final.log`.
Focused rustfmt, sorted-region, file-size, and diff checks passed.
The integrating owner independently approved the mapper correction after inspecting its cached Windows implementation.
Windows mapping performs cached hash lookups and string clones per candidate.
Native translation populates those caches when the platform constructs its mapper.
The raw mapping lookup therefore does not invoke `ToUnicode` or mutate dead-key state.
