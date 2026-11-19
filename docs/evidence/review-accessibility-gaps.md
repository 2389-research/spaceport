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
| Keyboard reorder and resize | Workspace reorder passes on `679cd74b8`; both terminal kinds still need native resize qualification. |
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

## Native Herdr resize candidate

The Windows suite also includes `native_herdr_prefix_resize_moves_text_bounds_and_preserves_both_panes`.
Setup creates one real right split through the existing app command.
The daemon must report both stable pane IDs before native input starts.
The helper waits for the focused right pane's presented native caret-line bounds.
It sends authored output, then Ctrl-B, r, and Left through native keyboard input.
After authored output arrives, the native caret line must move left before Right may run.
Right must restore its original horizontal position within one pixel.
The outer terminal viewport must stay fixed throughout both observations.
Every observed line rectangle must remain finite, positive, and inside the native terminal.
Both daemon pane IDs must remain unchanged after resizing.

The existing terminal helper defaults to its unchanged text-and-caret action.
The resize action reuses its exact foreground, document-focus, and modifier-release checks.
It creates no live-event observer and changes no clipboard data.
The shared daemon query preserves the held-key fixture's flags, deadline, and bounded capture.
Independent review finds no concrete issue in the native route or ownership checks.
The targeted Windows native-harness lint passes.
All four canonical host and Windows GNU lint configurations pass.
Formatting, size, sorting, operations, localization, commit, and diff checks pass.
The final five-file review finds no further material issue.
Logs use `/tmp/spaceport-9s44-pane-keys-*.log`.
Actual Windows execution remains pending; this does not qualify Core resizing or close `q72g`.

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

## Native Herdr text and line qualification on `fe4b3d7e4`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44565690663/job/135707084161) passes the corrected terminal case.
Its receipt occurs at `2026-11-19T16:35:43.9022389Z`.
Actual native input produces the expected private terminal output.
The provider exposes that text, a collapsed caret, and visible enclosing-line rectangles within the document.
The native TextChanged subscription observes the output and removes its listener.
The existing warning/toast live-region case also passes.

This qualifies Herdr text, caret semantics, line geometry, and the changed-output event only.
It does not prove audible speech, Core text providers, or exact collapsed-caret rectangles.
The remaining native matrix above keeps `q72g` open.
The job has eighteen passing UIA cases and one Settings observation failure; it is not green.

## Native workspace reorder candidate

The Windows suite now includes `native_workspace_arrows_change_daemon_order_and_preserve_terminal_ids`.
It creates two named workspaces through the private app's existing dialogs.
A separate real connection observes snapshots with surface interest disabled.
It sends no observer-side mutations or terminal input.
The helper focuses one unique, visible, enabled workspace row in the owned foreground window.
It sends one Alt+Down press, then one Alt+Up press.
Each direction must produce the expected complete workspace-ID order in the daemon snapshot.
Both directions must preserve the daemon boot and every observed terminal ID.
The native row order must also follow the daemon order.

[UIA SetFocus](https://learn.microsoft.com/en-us/dotnet/api/system.windows.automation.automationelement.setfocus) alone does not prove foreground ownership.
The helper separately requires the exact owned HWND and actual row focus before sending input.
It checks released modifiers before and after each workspace key.
Independent review found an unsafe fresh global focus read after input.
The correction captures focus once and checks its PID before describing that captured element.
Workspace receipts additionally require the original HWND and exact row identity after input.
Expected app exit still bypasses any post-exit focus query.
The focused review confirms the correction without further material findings.

The targeted Windows all-feature native-harness lint and all four canonical lint configurations pass.
Formatting, size, sorting, operations, localization, commit, and diff checks pass.
Logs use `/tmp/spaceport-9s44-workspace-keys-*.log`.
Actual PowerShell execution and native reorder qualification remain pending Windows CI.
No Mac fixture launched, and no personal daemon or clipboard was inspected.
This candidate does not qualify pane resizing, Core access, or audible screen-reader output.
Kata `q72g` remains open.

## Native enlarged Settings candidate

The Windows qualifier loads `[ui] size = 24` from its private config file.
It opens the real Settings window and reaches Smooth scrolling through native keyboard focus.
The existing integration-only observer requires that row's active keyboard context before capturing.
It resizes only that window to 1100 by 780, then 680 by 780 logical pixels.
The narrow width respects Settings' actual 680-pixel minimum.
Both captures must report the exact viewport and a contained focused row.
They use the existing native GPU readback and four-million-pixel bound.
PNG encoding runs in the background and allows at most 192 KiB per image.
The harness requires one image per width and unchanged private config bytes.

The ordinary focus-ring observer and keyboard actions retain their prior behavior.
The new scale observer stays behind the Windows integration-test gate.
Independent review finds no material issue in focus, capture bounds, ownership, or evidence scope.
Targeted Windows binary and native-harness lint passes with all features and without default features.
Formatting, size, sorting, operations, localization, commit, and diff checks pass.
The final seven-file review finds no further material issue.
Full workspace gates remain assigned to CI.
Logs use `/tmp/spaceport-9s44-settings-scale-*.log`.
Actual Windows execution and inspection of both PNGs remain pending.
Row containment alone does not establish glyph readability or text clipping.
Kata `q72g` remains open.

## First keyboard qualifier result on `021fd38a8`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44595959691/job/135814538707) passes eight GUI cases.
Its UIA suite passes twenty existing cases and fails the two new keyboard qualifiers.
The ARM native job passes; its existing policy excludes this UIA suite.
This checkpoint is not green.

Workspace reorder stops before key delivery because the row is not keyboard focusable.
The existing chrome contract enables those stops only inside the sidebar keyboard region.
The corrected scenario enters that region with two native prefix-F6 commands first.
It requires actual Workspaces focus before targeting the named row.
The original focusability and ownership guards remain unchanged.

Pane resize reaches its geometry wait but returns a generic boundary timeout.
The existing receipt cannot distinguish missing authored output from unchanged left or right geometry.
The next helper records fixed stage names, an authored-output boolean, and the last rectangle coordinates.
It records no terminal text and changes no key, geometry assertion, or deadline.
This diagnoses the failure; it does not claim a resize fix.
Independent review confirms both narrow changes without material findings.
Both corrected native scenarios still need execution.

## Keyboard and scaling result on `679cd74b8`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44599468067/job/135826942474) passes eight GUI cases and twenty-one UIA cases.
The two remaining UIA failures belong to resize lookup and Settings PNG encoding.
This checkpoint is not green.

Workspace reorder passes at `2026-11-19T19:47:36.1177302Z`.
Actual Alt+Down and Alt+Up change both daemon and native row order.
The scenario preserves every workspace ID, every observed pane ID, and the daemon boot.
This qualifies the workspace keyboard alternative on Windows; pane resizing remains open.

Resize passes its presented-right-pane guard and observes authored output.
Its `authored_output` stage returns `output=True` without a text range.
Pinned `accesskit_windows` 0.34.0 leaves `FindText` unimplemented in `src/text.rs:406`.
The corrected helper uses `GetSelection` and expands its unique collapsed caret to a visible line.
The existing terminal qualifier already exercises those supported APIs.
Authored output remains a barrier before resizing.
Focus, viewport, direction, restoration, and daemon pane-ID assertions remain unchanged.
Actual native resizing still needs the corrected scenario to pass.

Settings loads the 24px configuration but emits no image before hitting the 192 KiB PNG limit.
The encoder correction keeps that limit and both original image dimensions.
A new regression uses two existing full native Settings captures.
Fast/Sub encoding fails first with a 247,400-byte PNG.
Adaptive filtering passes while preserving every decoded RGBA byte.
Best compression adds headroom for the larger native scene.
Final encoded sizes are 120,905 and 102,400 bytes, with all pixels and dimensions unchanged.
The complete two-image regression takes 0.70 seconds in the local debug test executable.
Encoding remains background work behind the Windows integration-test gate.
The helper is additionally compiled for its unit test on other hosts.
No ordinary production build includes it.
Logs use `/tmp/spaceport-9s44-native-png-{red,green,final}.log`.
Current Windows GNU all-target Clippy checks pass with all features and without default features.
Formatting, file-size, sorting, operations, i18n, commit subjects, and diff checks pass.
The corrected native rerun and actual enlarged-glyph inspection remain pending.
Kata `q72g` stays open.

## Native result after Settings extraction on `63a885b15`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44610744275/job/135866672998) passes eight GUI cases and twenty-one UIA cases.
Workspace reorder passes again at `2026-11-19T20:50:25.4203838Z`.
The ARM native job also passes its configured checks.
Resize and enlarged Settings captures remain the two UIA failures.

Resize observes authored output and actual Left movement from 1103 to 1054 pixels.
The next observation receives no `DocumentRange` and dereferences that missing range.
The corrected observation waits for a document and caret range within its existing five-second deadline.
It still requires one collapsed caret, finite contained geometry, both directions, restoration, and unchanged pane IDs.
It sends no additional keys while waiting.
Full native resize qualification remains pending.

The Settings keyboard, saved-preference, and painted-ring case passes.
The enlarged-text case reaches its native row but emits neither a PNG nor an encoder error.
Its PowerShell observer times out waiting for the completion marker.
That failure does not identify compression, focus, or resizing as the cause.
The next candidate logs initial focus, resize requests, capture readiness, and encoder entry and exit.
Readiness diagnostics run at most three times per wait.
They contain only viewport dimensions and private fixture state flags.
No image limit, native guard, encoder, or deadline changes with those diagnostics.
Windows GNU GPUI all-target/all-feature Clippy passes with warnings denied.
Actual enlarged-glyph inspection remains pending.
