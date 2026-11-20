<!-- ABOUTME: Defines the real Linux app qualification pipeline and remaining matrix. -->
<!-- ABOUTME: Preserves ownership, source limits, and the next native verification step. -->

# Spaceport Linux native qualification

Goal `9s44` still covers all 53 findings.
Kata remains 49 closed and four open.
This pipeline does not close shortcut finding `3wd4` by itself.

Doctor Biz keeps Mac foreground checks pending.
Root also holds local VM and compiler work while the profiling resource question awaits a reply.
Disposable GitHub runners perform compilation and native Linux checks.
The existing scoped commit, push, and draft PR authorization applies.
No merge or release is authorized.

## Verified prerequisite

Exact source `8e43cd345405ef6e732bbe213e1c1f06cf5bc60e` passes GTK Polish input.
Run `44690512078`, job `136134910153`, reports the actual evdev text `łŃ`.
It verifies overview closure, active-window focus, key release, session closure, layout restoration, and direct-child cleanup.
The [receipt](gtk-input-8e43cd345.json) preserves the source and scope.
It starts no Spaceport process.

## Real app pipeline

- Reuse `live_gui`'s `Isolated`, `Sandbox`, and `gui_command` owners.
- Attach Spaceport explicitly to a checksum-pinned private Herdr 0.9.5 process.
- Run a Linux-only observer through the existing `--integration-test` entry.
- Share compiled cases and nonce-correlated notices from `crates/test-support/linux_shortcuts.rs`.
- Emit the mapping notice after preparation paints a real frame.
- Keep each READY notice behind exact-window activation, focus, and coherent daemon state.
- Send fixed evdev chords through the parent's persistent Mutter connection.
- After READY, observe ordinary app delivery without logical input, focus changes, or forced drawing.
- Require a real new daemon tab and actual `łŃ` in the production theme search field.
- Release uncertain keys, close the session, restore the private layout, and reap exact children.
- Verify the private daemon survives successful GUI exit.

The GNOME wrapper owns its bus, Shell, and exact test process.
The integration test owns the GUI, Herdr, settings children, and native input session.
The outer 180-second cutoff reserves time beyond internal startup, case, and cleanup bounds.
If that outer cutoff kills a hung test process, runner teardown contains remaining descendants.
Such an outcome fails qualification and never claims descendant cleanup succeeded.

## Remaining shortcut matrix

The first two cases establish the real producer and consumer route.
They do not replace these original requirements:

- Exercise Ctrl-Shift-1 through 9 and all retained punctuation aliases.
- Check exact pane, history, browser-group, Settings, shortcut-page, and window outcomes.
- Check font aliases and reset behavior through actual state.
- Receive bare F6, Shift-F6, F11, and Ctrl-PageUp/Down bytes in a real pane program.
- Exercise the revised Ctrl shortcuts and prefix h/j/k/l in real pane geometry.
- Reopen the palette with Ctrl-Shift-R while GNOME defaults remain intact.
- Keep the already qualified Windows Polish route's source limits explicit.

CI must compile the new Linux-only paths and run the shared notice tests first.
Then it runs the exact ignored native test on the owned GNOME display.
Local formatting and workflow checks do not qualify native delivery.
Release profiling, Mac Core Cmd-W, and audible screen-reader gates remain separate.

## First compilation result

Source `7ad46da551143d545d40f33c6cb4244c4a08c850` fails Linux compilation in [run `44694321745`](https://github.com/2389-research/spaceport/actions/runs/44694321745/job/136147325933).
The workspace denies unnecessary qualification in `smoke_live.rs:115`.
Its existing `use super::*` already imports the native fixture module.
The correction removes the redundant `super::` without relaxing the lint.
Shared contract tests and native execution remain pending; no cases ran on that source.

The corrected [source `b81adabe2`](https://github.com/2389-research/spaceport/actions/runs/44694961853/job/136149401775) passes compilation, three contract tests, and the exact native test.
The native test reports one pass in 1.61 seconds; this exercises the two initial cases.
Its published 30-line tail retains daemon logs but drops the case and cleanup receipts.
The next correction exports deduplicated protocol receipts separately from that tail.
It also requires the producer's exact native PASS line, so zero selected tests cannot qualify delivery.
Actual receipt-export verification and the remaining shortcut matrix stay pending.

Source `9254f9df4` passes three shared contract tests, two receipt tests, and the exact native test.
The [scoped receipt](spaceport-pipeline-9254f9df4.json) preserves the exported app-case and cleanup observations.
Ctrl-Shift-T creates and focuses a real daemon tab through the ordinary app path.
Native Polish input produces `łŃ` in the production theme search.
The private layout restores, the input session closes, and direct process cleanup passes.
The daemon survives successful GUI exit.
The remaining matrix above still blocks shortcut issue closure.

## Numbered tabs and terminal bytes

The next checkpoint prepares nine real daemon tabs, one acknowledged creation at a time.
Each Ctrl-Shift digit begins on another tab and must focus the tab at that workspace position.
The observer preserves tab order, workspace IDs, window handles, and daemon boot identity.

Each terminal-key case starts a fresh real tab and an owned shell capture script.
The script sets raw input, reads a bounded byte count, restores terminal discipline, and emits completion.
Nonce and case markers correlate the actual projected pane output.
READY requires the capture program's output before native input begins.
No logical input, focus changes, or forced drawing occur after READY.
The observer checks exact bytes, terminal focus, tab identities, and fullscreen state.

Pinned Herdr source defines the expected legacy sequences:

| Native key | Hex bytes |
| --- | --- |
| F6 | `1b 5b 31 37 7e` |
| Shift-F6 | `1b 5b 31 37 3b 32 7e` |
| F11 | `1b 5b 32 33 7e` |
| Ctrl-PageUp | `1b 5b 35 3b 35 7e` |
| Ctrl-PageDown | `1b 5b 36 3b 35 7e` |

Sources: [function keys](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/vendor/libghostty-vt/src/input/function_keys.zig#L51-L105),
[modifier substitution](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/vendor/libghostty-vt/src/input/function_keys.zig#L295-L312),
[legacy encoding selection](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/vendor/libghostty-vt/src/input/key_encode.zig#L82-L96),
and [modified page-key forwarding](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/src/server/pane_input.rs#L304-L332).
The fixture uses official Herdr 0.9.5, whose release commit is `2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01`.

Shared parser tests reject incomplete, unrelated, malformed, and incorrectly sized receipts.
The workflow selects those tests alongside the existing case and notice tests.
This expands qualification infrastructure without changing product key handling.
No failing baseline or compilation was run locally because local compiler work remains held.
Native delivery and these new parser tests remain pending until the exact-source CI run completes.

Source `22c5ace7c` now passes that [exact-source run](https://github.com/2389-research/spaceport/actions/runs/44700115942).
Eight shared contract tests and two receipt tests pass.
The exact native test passes all sixteen compiled cases in 6.30 seconds.
Every digit reaches its workspace-relative daemon tab.
All five pane-key cases produce their expected bytes without changing terminal focus or fullscreen state.
Polish search and the existing cleanup, layout restoration, and daemon-survival guards also pass.
The [source-scoped receipt](spaceport-matrix-22c5ace7c.json) retains every parent case completion and all five hex results.
The bounded GUI tail omits early tab details; parent case acknowledgements remain present for every case.

The next checkpoint adds native `+`, `_`, and `)` font aliases.
It waits for pending config loading before recording the size and cell-height baseline.
Reset begins at a different session size, then must restore the configured size.
These three cases remain pending until their own exact-source native run passes.
The earlier sixteen-case success does not qualify these additions.

Source `8a07fb51b` now passes its [exact-source run](https://github.com/2389-research/spaceport/actions/runs/44700969910).
Eight shared tests and two receipt tests pass again.
The exact native test passes all nineteen cases in 8.10 seconds.
Increase changes size 14 to 15 and cell height 20 to 21.
Decrease restores size 14 and cell height 20.
Reset changes prepared size 16 to configured size 14 and cell height 23 to 20.
The [font receipt](spaceport-fonts-8a07fb51b.json) preserves those actual observations and the cleanup results.

The next cases cover Keybindings, Palette, saved Palette resume, and the revised Ctrl routes.
Popup checks require actual search focus; Resume compares the complete saved state and query.
Tab cycling requires exact neighboring daemon IDs, wrapping within the prepared workspace.
Region cycling requires an actual focused control beyond the temporary entry focus.
Fullscreen checks require real entry and exit while preserving the focused terminal.
Prefix cases first verify Ctrl-B against the resolved private keymap.
These additions remain unqualified until their own exact-source run passes.
Settings, Cycle Windows, split groups, pane history/cycling, geometric prefix focus, and remaining Windows cases stay pending.
