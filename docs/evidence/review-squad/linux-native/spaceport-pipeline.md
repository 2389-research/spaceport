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

Source `6c2e09058` now passes its [exact-source run](https://github.com/2389-research/spaceport/actions/runs/44701934407).
Eight shared tests and two receipt tests pass.
The exact native test passes all thirty-four cases in 11.38 seconds.
All added popup, direct, and prefix outcomes pass without weakening the prior guards.
Key release, session closure, layout restoration, daemon survival, and exact process cleanup pass.
The [route receipt](spaceport-routes-6c2e09058.json) retains all parent case completions and available detailed route observations.
PR125 QA links the successful run; its application head remains `53de2db63`.
Kata remains 49 closed and four open.

## Exact resume point

The separate native branch is `fix/review-squad-linux-native`.
Source `5979a3a9d` remains the last whole-matrix success, with thirty-seven qualified cases.
Source `a72266897` passes native focus preparation but fails measured Cycle Windows.
Source `70395bd4d` passes those eight pane cases and confirms the platform activation call.
Source `16a928826` passes the complete forty-six-case native matrix with the corrected GPUI pin.
Its receipt and this note await the next integration checkpoint; do not push documentation solely to rerun the probe.
Do not publish a documentation-only checkpoint merely to start another native run.

Qualify the other configured platforms before integrating the shared GPUI change.
Windows source `c9a585ae5` exposes the PowerShell case-table nesting defect before native input.
The separate Windows correction is source `7e2459d67`; its next native probe remains pending.
Settings deliberately adds a window and changes activation; the general stable-window guard needs a scoped outcome branch.
Observe the actual Settings global handle rather than inferring its type from a window count.
Keep any observation hook behind `integration-test` and inside `settings_window`, whose window type is private.
Close only the owned Settings fixture through its normal close path before the next READY.
Wait for actual removal; do not cancel its persistence workers or force removal.
Before testing Cycle Windows, verify that new windows retain the explicit private endpoint and process ownership.
Prepare real pane additions one acknowledged creation at a time, as the tab fixture already does.
Keep measured stages free of logical input, focus changes, and forced drawing.

## Remaining Windows qualification

Read-only review identifies native coverage in the existing Windows fixture namespace.
Chrome Ctrl-F6/Ctrl-Shift-F6, held tab/window/split commands, and Polish `łŃ` already have native checks.
The Polish helper types RightAlt-L and RightAlt-Shift-N; it does not qualify lowercase AltGr-N/Z.
Settings button activation does not qualify its comma shortcut.
Logical `dispatch_keystroke` smoke checks do not qualify native Windows delivery.

Extend the existing disposable Polish scope with lowercase N/Z before claiming those collision examples pass.
Inspect real translation before choosing text assertions for B/W; do not invent Polish output.
Reuse `accessibility_windows::Running`, its Sandbox, explicit Herdr, bounded helpers, and real daemon snapshots.
Add exact positional digit targets, punctuation outcomes, and a real pane-owned console key/modifier producer.
Microsoft's [Console.ReadKey contract](https://learn.microsoft.com/en-us/dotnet/api/system.console.readkey?view=netframework-4.8.1) is a candidate producer contract.
First qualify its console attachment; UIA focus and app input counters cannot replace program receipt.
Windows reset is Ctrl-0; do not copy Linux's Ctrl-Shift-0 chord.
Windows retains its own arrow and Resume defaults, independently of Linux's GNOME exclusions.
Revised fullscreen, tab, Zen, and status-bar routes still need native outcome coverage.

Paths below are relative to `crates/sheprd-gpui`.
Relevant owners are `tests/accessibility_windows/non_us.rs`, `tests/support/native_altgr.cs`, and `tests/support/uia_altgr.ps1`.
The held native routes live in `tests/accessibility_windows/held_shortcuts.rs` and its `daemon.rs` child.
Reuse the serial Windows UIA and separate disposable-layout CI steps in `.github/workflows/ci.yml`.
Preserve ARM's explicit live-daemon limitation and keep default and layout scopes separate.
These are pending cases; the read-only review ran no compiler or native checks.

The next Linux checkpoint adds two Settings cases: shifted comma and the retained prefix-I route.
A Linux `integration-test` hook reads the actual typed Settings global and uses its ordinary close path.
Settings success permits exactly that new window while preserving the original windows and runtime target.
It requires actual activation of that Settings window.
Preparation closes the previous popup before READY, then closes the owned Settings window after its measured success.
The close request defers until the observer releases its main-view borrow; normal close may update source views.
The next case waits for removal and original-window activation before accepting more native input.
All other outcomes retain their original window guards.
These two cases remain unqualified until their exact-source native run passes.

Source `7d8bf54a2` now passes its [exact-source run](https://github.com/2389-research/spaceport/actions/runs/44703331035).
Eight shared tests and two receipt tests pass.
The exact native test passes all thirty-six cases in 13.13 seconds.
Both Settings routes activate the actual window and preserve the original windows and runtime.
Each owned Settings window closes through the normal path before the next case.
All prior cases, cleanup, layout/session restoration, direct reaping, and daemon-survival checks pass.
The [Settings receipt](spaceport-settings-7d8bf54a2.json) preserves those scoped observations.

Source review confirms the next presentation cases can reuse the private runtime.
`command_new_window` clones the first endpoint's target into deferred `open_additional_window_then` creation.
An explicit Socket remains attach-only and disables catalog loading; it cannot start Herdr.
Capture that creation callback's exact second handle, then verify its target, coherent snapshot, and daemon boot before READY.
Observe actual destination-window activation after Cycle Windows.
Defer cross-window activation and normal closure until the original view update returns.
Wait for removal of that exact owned handle before another READY.

Split Editor starts with one ordered group from `group_slots()` and `active_group()`.
It must retain the original ID left, add one distinct ID right, and activate that new group.
Require empty `group_tabs()` and `Shown::Empty` for the new group; all old tabs stay with the original.
Preserve global daemon pane IDs, tabs, runtime focus, boot, workspace, and windows.
Close only that captured empty group through `close_group`, then await the original layout.
The owning implementations are `browser/groups_view.rs`, `browser/groups.rs`, and `window/commands/handlers.rs` under the GPUI crate.

The Windows implementation owner works separately in `.worktrees/review-squad-windows-native`.
That owner may edit Windows fixtures, support, registration, and its exact-branch probe workflow.
Root owns the Linux app observer, Settings hook, and shared Linux cases.
The Windows owner must return a reviewable diff before committing or pushing.

The next Linux checkpoint adds the native Ctrl-Shift-Backslash Split Editor alias.
It records ordered group IDs, active group, group tab ownership, and global runtime pane IDs.
Success requires the unchanged original group left and exactly one distinct empty active group right.
Cleanup runs in a separate preparation stage after PASSED.
It guards the exact captured empty group, calls normal `close_group`, and waits for the original layout.
The next READY also waits for coherent runtime geometry and preserved runtime members.
Native compilation and this thirty-seventh case remain pending for the next exact-source run.

Source `5979a3a9d` now passes its [exact-source run](https://github.com/2389-research/spaceport/actions/runs/44704607543).
Eight shared tests and two receipt tests pass.
The exact native test passes all thirty-seven cases in 13.88 seconds.
The new right-hand group is empty and active; original tabs and global runtime pane IDs remain intact.
Normal guarded cleanup restores the original layout before Polish input.
The [group receipt](spaceport-groups-5979a3a9d.json) preserves actual creation and cleanup observations.
All prior cases and process, layout, session, and daemon-survival guards pass.

The next case measures native Ctrl-Shift-Grave Cycle Windows.
It opens the second window once through the existing deferred creation callback.
Preparation requires the exact private Socket target, coherent second-window state, matching boot, and unchanged runtime members.
READY requires the original window active with terminal focus and the owned target inactive.
Observation requires the exact target active and coherent, while both original windows and runtime members remain present.
Each window may retain connection-local terminal selection; this case does not compare their focused pane IDs.
The inactive original's surface geometry is irrelevant to this window-focus outcome.
All other measured cases retain their original surface-coherence guard.
The active target must pass full snapshot/surface coherence and structural validation.
Cleanup defers normal Close Window on only the captured handle and waits for its exact removal.
The next case waits for original-window activation and coherent geometry before READY.

The observer moves into its own module before adding this ownership flow.
Its prior body matches committed source exactly before the Cycle Windows branch is added.
This keeps each file under 1,000 lines without exposing baseline fields.
The thirty-eighth case remains pending its exact-source compilation and native run.

Source `bf599c076` compiles and passes eight shared tests and two receipt tests.
Its [native run fails during preparation](https://github.com/2389-research/spaceport/actions/runs/44706339419), before Cycle Windows input is sent.
Thirty-six earlier cases complete; the final Polish case does not run after the preparation failure.
The parent waits for `Ready(cycle-windows-alias)` after normal empty-group cleanup, then reaches its unchanged deadline.
The [failure receipt](spaceport-window-preparation-bf599c076.json) preserves the scope and cleanup observations.
The previously qualified `5979a3a9d` run remains separate.

The next diagnostic checkpoint reports creation callbacks and changed readiness state for each exact window.
It records activation, connection, fencing, config loading, surface sizing, and coherence as bounded booleans.
It adds no input, activation request, creation retry, deadline change, or measured-scope relaxation.
The next native result must identify the actual waiting gate before a correction is chosen.

Source `2db45a502` passes compilation, eight shared tests, and two receipt tests.
Its [diagnostic native run](https://github.com/2389-research/spaceport/actions/runs/44709971306) still stops before measured Cycle Windows input.
The [diagnostic receipt](spaceport-window-focus-2db45a502.json) identifies the gate.
The second window is created, active, connected, unfenced, config-loaded, surface-sized, and coherent.
The original is coherent but remains inactive after its activation request.
This establishes the missing preparation acknowledgement; it does not yet establish a product shortcut failure.

The correction requires the exact second window active and coherent before requesting native focus preparation.
A compiled `FocusOriginal` notice carries the exact nonce and Cycle Windows case ID.
The parser rejects that preparation notice for every other case or nonce.
The parent sends fixed Alt-Escape once, then waits for the original active/coherent and target inactive before READY.
GNOME 46 defines [Alt-Escape as direct window switching](https://github.com/GNOME/gsettings-desktop-schemas/blob/46.0/schemas/org.gnome.desktop.wm.keybindings.gschema.xml.in).
Alt-Tab switches applications, so it cannot serve as the two-window preparation contract here.
Measured Ctrl-Shift-Grave, cleanup, and existing deadlines remain unchanged.
Source `a72266897` passes nine shared tests and two receipt tests.
Its [native run](https://github.com/2389-research/spaceport/actions/runs/44712851380) now reaches Cycle Windows READY after native Alt-Escape preparation.
The exact original is active and coherent; the target is inactive.
The measured chord completes submission, but the target remains inactive until the original deadline.
The [failure receipt](spaceport-cycle-a72266897.json) preserves this distinction from the earlier preparation failures.
Command dispatch versus platform activation remains unproven.
The next checkpoint logs those existing boundaries behind the opt-in integration observer.
It adds no focus, native input, retry, deadline extension, or measured-state mutation.

The next checkpoint adds eight pane cases before Cycle Windows and final Polish input.
It prepares one two-by-two grid through three acknowledged splits and one acknowledged return to the original pane.
Each pending split or focus is recorded before submission and never replayed.
Pane cycling follows actual snapshot order within the current workspace and tab.
Back starts from an acknowledged top-left, top-right, bottom-right trail.
Forward requires the retained bottom-right history target after native Back succeeds.
Prefix h/j/k/l starts from distinct panes and derives targets from the actual visible grid.
Every measured result preserves runtime members, tabs, windows, and fullscreen state.
It requires the exact target pane, terminal focus, coherent geometry, and a newer daemon revision.
The shared test defines the physical aliases before their fixture implementation.
The local compiler hold prevents a failing-baseline run; no baseline failure or native pass is claimed.
Independent read-only review finds no concrete blocking issue.
Compilation, the new shared test, and all eight native cases remain pending CI.

Source `70395bd4d` passes ten shared tests, two receipt tests, and forty-four native case outcomes.
All eight pane additions reach their exact IDs in real two-by-two geometry with newer daemon revisions.
The [scoped receipt](spaceport-panes-70395bd4d.json) records these outcomes and the remaining failure.
The [whole run fails](https://github.com/2389-research/spaceport/actions/runs/44713905098) at Cycle Windows; final Polish input does not run.
The trace proves native submission, command dispatch, and the platform activation request all occur.
The exact target stays inactive until the unchanged deadline.

The [pinned GPUI backend](https://github.com/zed-industries/zed/blob/08cbb88a152c3751a43e853425a70e9f631a5918/crates/gpui_linux/src/linux/wayland/window.rs) uses a mouse-press serial and the destination surface for activation tokens.
[Mutter](https://github.com/GNOME/mutter/blob/46.0/src/wayland/meta-wayland-activation.c) requires the serial and surface from a valid input event.
This points to the backend's activation context, rather than shortcut routing.
Upstream [GPUI PR64058](https://github.com/zed-industries/zed/pull/64058) already corrects that context and correlates each token separately.
Published `gpui-pre-linux` 0.3.11 contains that correction at Zed source `f816e1f9263e3a8ef63c4baaa02e1441ce1c6038`.
Its license remains Apache-2.0.

The correction re-pins both GPUI roots together to exact 0.3.11.
This changes the previous pin specifically to fix the reproduced native failure.
It preserves the pinned Rust toolchain, compositor controls, input sequence, deadlines, and pass criteria.
The lockfile retains existing non-GPUI versions where the new dependency constraints permit them.
Cargo resolves metadata locally; compilation and native tests still run only on disposable CI runners.
The new source must pass the unchanged Linux matrix before this fix can qualify.
All configured platform and feature jobs must also pass before integration can qualify the dependency change.
Earlier 0.3.10 qualification does not establish 0.3.11 behavior on Windows or macOS.
Locked workspace metadata resolves without compilation.
Quick formatting, size, sorting, recipe, i18n, commit checks, receipt JSON, and whitespace checks pass.
An independent read-only review verifies the published source, checksum, license, coordinated pins, and preserved dependency versions.
The review finds no blocking issue; it does not replace any pending platform checks.

Source `16a928826` now passes the [complete native run](https://github.com/2389-research/spaceport/actions/runs/44715081137).
Ten shared contract tests, two receipt tests, and the exact native test pass.
All forty-six compiled cases complete in the native test's 16.76 seconds.
Cycle Windows activates the exact coherent target while preserving runtime members.
Its owned second window closes normally before final Polish search succeeds.
Key release, session closure, layout restoration, direct-child reaping, and daemon survival all pass.
The [complete receipt](spaceport-complete-16a928826.json) preserves source limits and the failing 0.3.10 baseline.
The test's input, deadlines, target guards, and observed success criteria remain unchanged across the pin correction.
Windows, macOS, and full configured CI still need to qualify the shared framework update.
Kata remains forty-nine closed and four open; the platform shortcut issue is not complete yet.
