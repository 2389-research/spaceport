# Smaller accessibility gaps

Date: 2026-11-17. Scope: `q72g` under `9s44`.
Doctor Biz authorized fixes and validation without commits or external writes.

## Behavior

Core terminal panes now expose a named Terminal node, including observing panes.
Core and Herdr use the same visible-grid transcript builder.
It preserves concealed cells, grapheme boundaries, trimmed padding, and selection text.
A visible program cursor supplies a collapsed caret when no selection exists.
A hidden cursor, or one outside the exposed pane, supplies no caret.
Changed visible text carries AccessKit's polite live-region metadata.
Core synthetic rows use the current render's actual canvas bounds.
Transcript work runs only while accessibility is active.
No input permission, daemon protocol, or runtime ownership changes.
No dependency or runtime additions.

Settings chrome follows the configured UI font size through a window-local rem basis.
The default 12-pixel setting preserves the previous sizes.
Theme card geometry and numeric field widths grow with their labels.
Actual font previews retain their own font sizes.

Workspace rows accept Alt+Up and Alt+Down while focused on the selected endpoint.
The existing drag Plan supplies group membership, sibling limits, and request selection.
Parent rows move their entire repository group.
Linked worktrees move only among their siblings.
Captured endpoint, selection epoch, generation, and boot prevent stale callbacks from moving another owner's rows.
An open dialog, disconnected transport, or active drag refuses the move.
Held keys queue no repeated reorder.
Configured commands keep priority through the existing shortcut capture.
Queue errors use the existing workspace-move notice.

Tab focus rings use the selected layout's actual active, inactive, and unselected fills.
The existing cached contrast helper supplies the color.
Other chrome rings use the existing theme ink policy.
Close buttons now expose Button roles and names.
The selected close action uses the existing active-descendant mechanism.
The existing outer menu overlay remains the dialog and owns keyboard focus.

Pane resize alternatives already existed.
Herdr's prefix-R resize mode sends pane.resize through the endpoint API.
Core's directional Resize commands change its pane splits within existing bounds.
These routes were verified and documented without adding global bindings.

## Regression evidence

The original node tests failed for missing Core Terminal role and missing unselected caret.
See `/tmp/spaceport-qnz9/q72g-node-caret-red.log`.
The initial nested-dialog assertion was discarded after inspecting the existing outer dialog owner.

The Settings regression doubled UI size while its row title remained 18 pixels high.
See `/tmp/spaceport-qnz9/secondary-text-consumers/2.log`.
The passing regression checks growth, row containment, and restoration to the default size.

The raw tab accent failed contrast on the actual selected tab fill.
See `/tmp/spaceport-qnz9/q72g-focus-red.log`.
Testing ordinary chrome ink also caught Nord's accent-wash fill.
The final test covers every builtin theme, layout, selected state, contrast mode, and a blending custom accent.

A controlled baseline disables Alt row handling under the shared Cargo lock.
The real peer then receives no reorder before its bounded read expires.
Removing the close button role also fails with None instead of Button.
Both expected runtime failures appear in `/tmp/spaceport-qnz9/q72g-baseline-*.log`.
The script restores the exact source hunks before releasing the lock.
Ownership tests invoke the captured sender before a redraw can replace its generation.
GPUI's keystroke helper may otherwise redraw and capture a fresh callback.
The peer tests use real sockets and the production client worker with simulated daemon frames.
They are integration tests, rather than live Herdr end-to-end qualification.

## Verification

All Cargo commands use the shared lock, CARGO_INCREMENTAL=0, and the shared target directory.
The final scoped batch cleans only sheprd-gpui build artifacts before compiling.

- Accessibility scope: 32 passed.
- Settings scope: 251 passed.
- Reorder scope: 24 passed, including configured-shortcut priority.
- Focus ring scope: 1 passed across the theme/layout matrix.
- Close modal scope: 16 passed.
- Core view scope: 2 passed.
- Existing Herdr resize-wire contract: 1 passed.
- Existing Core resize-bounds contract: 1 passed.

Logs: `/tmp/spaceport-qnz9/q72g-final-0.log` through `q72g-final-7.log`.
The final reorder log is `/tmp/spaceport-qnz9/q72g-reorder-reviewed-green.log`.
The narrow 640-pixel font regression also passes.
The final close control scope also passes 16 tests.
Windows all-target/all-feature GPUI Clippy passes.
Host all-target/all-feature GPUI Clippy passes with the validated Sparkle SDK.
Logs: `/tmp/spaceport-qnz9/q72g-host-clippy.log` and `q72g-windows-clippy.log`.
Format, file-size, sorted-region, localization, and diff checks pass.
The localization checker reports 2143 valid, used messages.
The existing large debug-link unwind warning and block 0.1.6 future-compatibility notice remain unchanged.
No new Rust or Clippy warnings remain.
In-flight sibling shortcut tests temporarily blocked their compilation; no failure was hidden.
Independent root review approved the final source and regression evidence without material findings.
The review covered transcript bounds, Core input ownership, row fences, contrast fills, Settings scaling, and dialog ownership.

## Limits

The pinned GPUI TestPlatform does not activate accessibility through its public test API.
Headless tests inspect actual element metadata and the shared synthetic-node producer.
They do not qualify a full native accessibility tree or screen-reader speech.
Contrast checks compare the layouts' RGB fills.
Translucent or image backdrops still need native visual inspection.
Native screen-reader announcements, Settings glyph/clipping checks, and physical keyboard checks remain pending.
Doctor Biz kept native keyboard and performance checks pending for this review.
Native Windows execution and the full workspace CI gate remain pending.
No personal daemon, SSH host, agent, or production credentials were used.

## Native completion audit

An independent audit finds implementation and regression coverage for all six original requirements.
It finds no concrete remaining code defect in this scope.
The published Windows tree/focus check covers only part of the native contract.
It does not query terminal text or caret ranges.
The first consent follow-up also times out before returning a tree.
Kata `q72g` remains open for these specific checks:

| Requirement | Remaining native evidence |
| --- | --- |
| Core Terminal node | Inspect actual text, caret, bounds, and observing panes with accessibility active. |
| Herdr caret and announcements | Query caret/text ranges and observe changed output announced by a screen reader. |
| Settings font scaling | Inspect enlarged glyphs and clipping, including a narrow window. |
| Keyboard reorder and resize | Reorder workspace rows and resize both terminal kinds through native key delivery. |
| Selected-tab focus contrast | Inspect focused tabs on actual fills, including applicable translucent or image backdrops. |
| Close-button roles | Native names, Button roles, cancellation, and confirmation pass on `64970ccce`; see below. |

## Native close-button qualification

Source `64970ccce` passes the [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44551798252/job/135657584169).
The private workspace case requires native `Button 'Close workspace'` and `Button 'Cancel'` nodes.
Actual Enter cancels without losing the workspace; Tab then Enter confirms a fresh dialog.
The private Close Others case requires both native Button roles before any tab closes.
Cancellation preserves every captured ID, and one confirmation closes only the two named targets.
Those cases pass at `2026-11-19T15:21:01.2369189Z` and `15:19:18.8859389Z` respectively.
The role and keyboard-effect evidence satisfies the original close-button requirement.
It does not qualify the remaining Core, caret, scaled Settings glyph, reorder, or selected-tab image checks.
Kata `q72g` remains open.

## Native Herdr text and caret candidate

The next Windows case opens only a private Herdr terminal.
It requires the native Document's TextPattern before running its helper.
Native keys require the exact foreground HWND, the owned Terminal focus, and released modifiers.
The helper sends an authored echo command and requires its standalone output line.
It observes the native Terminal TextChanged event through the shared bounded MTA listener.
It resolves a fresh document range and reads at most 16384 characters, without logging terminal contents.

The [UIA GetSelection contract](https://learn.microsoft.com/en-us/dotnet/api/system.windows.automation.textpattern.getselection) returns a collapsed range for an unselected system caret.
Pinned AccessKit Windows 0.34 implements that path through its terminal text selection.
The fixture checks the collapsed caret separately from its enclosing visible line's finite, contained bounds.
It does not invent TextPattern2 support; the pinned provider exposes TextPattern.
Observer removal and thread join must succeed before the helper prints its removal receipt.

The shared observer now accepts only LiveRegionChanged or TextChanged.
The existing warning/toast consumer explicitly selects its original live-region event.
PID/window scoping, record bounds, registration, removal, and join contracts remain unchanged.
Independent review finds no material contract, ownership, or evidence gap.
Both host and Windows GNU lint configurations and static checks pass.
Logs: `/tmp/spaceport-9s44-terminal-native-{lint,static}.log`.
C# compilation, actual native text/caret execution, and the existing live-event regression remain pending CI.
The [TextChanged contract](https://learn.microsoft.com/en-us/dotnet/api/system.windows.automation.textpattern.textchangedevent) qualifies native text-change delivery once this case passes.
It does not prove audible screen-reader speech or Core terminal accessibility.

## First native terminal result on `1ad093a96`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44560521338/job/135688647754) passes eighteen UIA cases and fails only this new terminal qualifier.
The qualifier reaches the rectangle assertion after observing authored output, a native TextChanged event, and a collapsed caret.
Its rectangle assertion incorrectly expects four scalar coordinates from a managed API.
[GetBoundingRectangles](https://learn.microsoft.com/en-us/dotnet/api/system.windows.automation.text.textpatternrange.getboundingrectangles) returns Rect objects and permits no rectangles for a collapsed range.

The correction retains the collapsed-caret checks.
It expands a cloned range to the enclosing line and checks every visible Rect's finite, contained geometry.
It reports enclosing-line bounds, rather than claiming a caret rectangle that the API need not return.
The app's accessibility producer and listener lifecycle remain unchanged.
Corrected native execution remains pending; `q72g` stays open.
