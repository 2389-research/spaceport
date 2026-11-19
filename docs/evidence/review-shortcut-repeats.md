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

## Native tab qualification candidate, November 19

The original issue has no Mac-only requirement.
The Windows candidate uses the selected private Herdr daemon and normal application.
Two fresh New Tab presses must each create exactly one tab.
Two fresh Close Tab presses must each close exactly one tab.
The fixture always retains its first tab.

Each press remains down until its first tab change appears in UI Automation.
Five additional native down events then exercise the held tail after that acknowledgement.
No intervening up event converts those repeats into fresh presses.
Each burst must preserve the acknowledged tab count.
The fixture counts only the terminal `Tabs` container, excluding sidebar activity tabs.

The helper uses Microsoft's [SendInput API](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput).
Windows exposes previous key state through [WM_KEYDOWN bit 30](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-keydown).
The pinned GPUI mapper converts that native bit into `is_held`.
A Windows integration-only diagnostic records that boolean before shortcut policy returns.
Only explicit shortcut fixtures enable its observer; other integration and performance runs omit those records.
The fixture requires five complete held-event records per burst before starting another baseline.
It records no key names, text, terminal output, or command arguments.

Before each down event, the exact private HWND must own foreground and focused-element ancestry.
Initially held keys cause refusal.
The helper tracks only successfully injected keys and releases them in reverse order.
Rust supplies its absolute deadline before launching the helper.
The helper subtracts setup time and reserves five seconds for cleanup before the parent deadline.
It refuses injection when the remaining budget cannot cover the held phase.
A C# timer releases tracked keys and exits the exact helper if UIA stalls.
Normal exceptions use the same release owner through `finally`.

The existing shortcut-repeat consumer scope passes all 15 tests.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- window::tests::shortcut_repeat:: --nocapture`.
Log: `/tmp/spaceport-9s44-held-tabs-consumers.log`.
Independent review found and corrected the broad tab count and setup-time deadline race.
The correction review finds no remaining material issue.
Native execution was pending when this candidate was prepared.
This candidate covers tab shortcuts only.
Window, split, and custom-command qualification remain pending for `agbt`.

The final candidate passes both host lint configurations and both Windows GNU configurations.
The explicit observer gate also passes the two Windows configurations after its addition.
Logs: `/tmp/spaceport-9s44-batch-labels-final-lint.log` and `/tmp/spaceport-9s44-batch-labels-observer-lint.log`.
Static, commit, and diff checks pass.
Independent review approves exact focus ownership, terminal-strip counts, and deadline-aware key release.
The owner review finds no remaining material issue.
Rust cross-checking does not execute the C# helper or qualify native held events.

## Native tab result and New Window candidate

The tab candidate passes on source `b20cca86f` in the [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44507153995/job/135501954152).
Both New bursts and both Close bursts receive five held events after their first visible tab change.
The [popup qualification](review-popup-close-motion.md#qualified-native-result-on-b20cca86f) records that complete native job.
This qualifies native tab repeats; `agbt` remains open for windows, splits, and custom commands.

The extended candidate uses the same helper and private daemon for New Window.
Two fresh presses must grow the UIA window count from one to two to three.
Each first effect precedes five additional downs of the still-held main key.
Each held tail must preserve the window count.
The real `window list --json` command separately verifies unique IDs and preservation of all previous IDs.
The new windows inherit the captured private endpoint target.

The helper adopts only the unique newly created PID-owned HWND.
It waits for that exact window's foreground and focused ancestry within its original deadline.
Window actions do not require terminal tab chrome to have loaded.
All down events retain the exact focus guard, initially-held refusal, and owned-key cleanup.

Pinned GPUI activation sends Alt down/up before requesting foreground.
That sequence can release the fixture's Alt modifier while N remains down.
After exact activation, the helper requires N still held and restores only missing, previously owned modifiers.
It checks the complete chord before every held tail event.
N receives no intervening up event.

The integration-only observer records whether the actual event resolves to New Window through the existing binding selector.
It uses the real mapper, configured/catalog precedence, and preferred-text policy without dispatching.
The native test counts only records with both `is_held` and `new_window` true.
Activation's Alt events and incorrectly modified N events cannot satisfy that proof.
The diagnostic retains only booleans, with no raw keys or command arguments.

Independent review catches and resolves activation readiness, modifier loss, and unnecessary tab-chrome dependencies.
The correction review finds no remaining material issue.
Actual Windows execution remains pending.
This candidate does not qualify split or custom-command repeats.

The current candidate passes both Windows GNU lint configurations.
The final combined chat-lifetime checkpoint also passes both host configurations and repeats both Windows configurations.
Logs: `/tmp/spaceport-9s44-native-window-repeat-final-lint.log` and `/tmp/spaceport-9s44-chat-lifetime-window-repeat-lint.log`.
The unchanged repeat consumers pass all fifteen tests with none ignored.
Log: `/tmp/spaceport-9s44-native-window-repeat-consumers.log`.
The final combined preview, chat-accessibility, and repeat scope passes 28 tests with none ignored.
Log: `/tmp/spaceport-9s44-chat-lifetime-repeat-consumers.log`.
Static, localization, commit, rename-reference, and diff checks pass.
Log: `/tmp/spaceport-9s44-chat-lifetime-window-repeat-static.log`.
The owner review finds no remaining material issue in this candidate.

## First New Window execution and daemon candidates

Source `99e318867` passes chat and the retained tab-repeat cases in the [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44520974760/job/135549493119).
The New Window case stops before its held-input helper runs.
Its initial `--json window list` call enters startup parsing and reports `Unknown option: window`.
The fixture now uses `window list --json`, matching the existing CLI wire table and transport tests.
No application CLI contract changes.
New Window execution remains pending.

The split and custom candidates use the same private daemon and exact-window input helper.
Each case requires two fresh presses, with five matching held downs after each first effect.
The helper holds the main key while the parent checks the actual daemon and app fence.
The parent owns CLI queries separately from the PowerShell helper.
CLI queries use the existing no-console flag so they cannot open a focus-stealing console window.

The app emits optional readiness receipts only for the explicit fence-observer fixture.
Readiness requires the real surface stream and existing coherent-input predicate.
The existing ordered surface barrier follows the command request in the client's FIFO queue.
The first native event, later readiness, and real first effect must all precede the parent's ACK file.
The helper then sends its held tail.
All five actual matching input records must show readiness.
A later coherent readiness receipt fences the final assertions too.
These diagnostics contain booleans, with no key names or command arguments.
The parent reads complete JSON records within the existing four-MiB diagnostic bound.
Normal integration and performance runs omit these receipts.

The split case reads the stable tab through Herdr's real `tab get` command.
It checks pane counts one, two, and three and preserves all prior pane IDs through `pane list`.
The custom case configures one shell command under the sandbox's private TEMP.
Its bounded chain creates the first available directory among sixteen atomic slots.
Each fresh press must produce exactly one real marker and one accepted invocation.
The accepted-invocation receipts prevent delayed shell children from hiding extra dispatches.
They do not claim that the client owns or reaps those daemon-owned children.
Herdr's pinned CLI, configuration, and response schemas were checked at `2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01`.
No Herdr source changes.

Independent review catches a parent-unwind cleanup gap before native execution.
A failed CLI query or assertion could previously drop Running and kill the helper before key release.
The parent now catches failures while retaining Running, requests release, and waits for the owned helper.
It preserves the original failure after finally or the independent watchdog can release the keys.
Forced reaping retains the original parent deadline and watchdog cleanup reserve.
An added native regression deliberately fails after a real first split.
It requires the original panic, a reaped helper, actual Ctrl/Shift/D key-up states, and exactly two panes.
The correction review finds no remaining material issue.

Windows GNU all-feature and no-default lint configurations pass.
Log: `/tmp/spaceport-9s44-daemon-repeat-lint-final.log`.
The first lint attempt catches an incorrect Method import in the fixture-only queue receipt.
The corrected path uses the client's actual root export.
Actual Windows execution of New Window, split, custom, and the cleanup regression remains pending.
Kata `agbt` stays open until those native requirements pass.

The combined repeat, CLI wire, preview, and chat-accessibility scope passes 29 tests, with none ignored.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- window::tests::shortcut_repeat:: control::client::tests::wire:: chat::preview::tests:: chat::tests::accessibility:: --nocapture`.
Log: `/tmp/spaceport-9s44-daemon-repeat-consumers.log`.
The CLI wire table exercises the corrected argument order through the real parser and serializer.
Both host lint configurations also pass.
Log: `/tmp/spaceport-9s44-daemon-repeat-host-lint.log`.
The native cleanup regression cannot run on this Mac; its baseline and corrected execution remain Windows CI work.
The owner review finds no remaining material issue in the reviewed candidate.
The final key-state helper uses the existing PowerShell module bootstrap and terminating errors.
Its final Windows harness lint also passes.
Log: `/tmp/spaceport-9s44-daemon-repeat-harness-final-lint.log`.
Formatting, size, sorting, operations, localization, commit, and diff checks pass.
Localization validates 2174 messages in 114 files.
Log: `/tmp/spaceport-9s44-daemon-repeat-static.log`.
