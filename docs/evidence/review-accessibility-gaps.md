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
| Core Terminal node | Ordinary Core text passes on `23cab5b4b`; ordinary and observing text, caret, bounds, and change events pass on `174db4857`. |
| Herdr caret and announcements | Text, caret, line bounds, and TextChanged pass; audible screen-reader announcements remain pending. |
| Settings font scaling | Enlarged native glyphs and clipping pass at 1100×780 and 680×780 on `7b7498762`; see captures below. |
| Keyboard reorder and resize | Workspace reorder and Herdr resize pass on `4194cb1ab`; Core resize and restoration pass on `a8f7bd371`. |
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

## Resize qualification and scaling diagnosis on `4194cb1ab`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44614263920/job/135878875152) passes eight GUI cases and twenty-two UIA cases.
The only UIA failure is enlarged Settings capture.
Herdr resize passes both arrows, changed and restored text bounds, and stable pane IDs.
Its native pass is recorded at `2026-11-19T21:12:26.6955235Z`.
Workspace reorder passes again.
This qualifies the Herdr keyboard divider alternative; Core resizing remains pending.

The Settings observer reaches the focused row and captures the 1100×780 viewport.
Its PNG is 127,468 bytes, below the unchanged 192 KiB cap.
Encoding finishes about 1.03 seconds after its entry marker.
The narrow viewport reaches 680×780 with the window active and the same row focused.
Its diagnostic then reports `keyboard=false`, which keeps the existing capture guard closed.
No timeout or encoder adjustment is needed to address that observed condition.
The next private fixture requests one native Tab/Shift-Tab round trip after the resized viewport loses keyboard modality.
Its controller checks the exact HWND and row before and after the keys.
The application still waits for its ordinary keyboard-focus capture guard.

The [actual wide capture](review-squad/settings-scale/ui-24-before-wide-1100.png) was decoded and inspected.
It preserves its native 1100×780 dimensions and original PNG bytes.
SHA256: `de8f33d6c5d67ea0ca86746b859769d2e9a5073e2302bc2691a868667e27089b`.
The General text scales, but the fixed navigation width clips several section labels.
That native finding adds a scoped layout correction before qualification can finish.
The default width now follows measured section labels and the configured font.
Dragged widths stay chosen, and narrow deliberate choices use ellipsis.
The existing panel-space cap and reset behavior remain.

A real-layout regression first fails because Indicators needs 144px but its text column has only 112px.
The correction passes at 1100px and 680px, retains a chosen 300px width, and resets to 184px at the normal font.
The existing thirty-two navigation, font, pseudo-locale, RTL, and keyboard cases also pass.
The initial combined run caught the new test's incorrect exact grip-edge assertion.
The final test checks the stored width exactly and the existing one-pixel inset border convention.
Logs use `/tmp/spaceport-9s44-settings-navigation-scale-{red,first-green,green}.log`.
`just lint lint-windows` passes all four workspace/all-target feature and platform configurations.
Format, file-size, sorting, operations, i18n, commit-subject, and diff checks pass.
Independent review finds no material width-policy, focus-handoff, role, or guard issue.
The private handoff and corrected layout still need execution on Windows.
Fresh native captures of the corrected layout remain pending.
Kata `q72g` remains open for the rest of its matrix.

## Enlarged Settings qualification on `7b7498762`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44624169019/job/135913747987) passes eight GUI cases and all twenty-three UIA cases.
Its separate pipe-owner case also passes.
The ARM native job passes its configured seven GUI cases and pipe-owner case.
ARM does not run the UIA suite or live-daemon input case.

The enlarged Settings case loads `[ui] size = 24` and preserves the private config bytes exactly.
Both captures retain their original encoded bytes, exact native dimensions, and unchanged 192 KiB cap.
The in-app observer requires an active window, keyboard modality, and the focused Smooth scrolling row before each capture.
The guarded native handoff runs at the wide viewport; the narrow capture then passes without another handoff.
The fixture log calls the size points, but the configuration and renderer use logical pixels.

| Native capture | Dimensions | PNG bytes | SHA256 |
| --- | --- | ---: | --- |
| [Wide](review-squad/settings-scale/ui-24-after-wide-1100.png) | 1100×780 | 129267 | `bb1da0709815a0ac0a2c4ae9f5fa023728c7802ff68998fdd01846954c163b4a` |
| [Narrow](review-squad/settings-scale/ui-24-after-narrow-680.png) | 680×780 | 103064 | `8d701d1f33932a0dc4d9c78bbedd4318a8c04d6ccc4a1069996009e659830ab2` |

Root decodes and inspects both native PNGs.
Tabs & panes, Notifications, Configuration, and the search placeholder now fit the navigation.
General body text remains readable and wraps inside the narrow window.
The focused Smooth scrolling control and footer remain inside both viewports.
This qualifies the original enlarged Settings glyph and clipping requirement for these authored native scenes.
The [before-image](review-squad/settings-scale/ui-24-before-wide-1100.png) records the original navigation clipping.
The [receipt](review-squad/settings-scale/qualification-7b7498762.json) records source, job, dimensions, and hashes without machine-local paths.

The same native job also passes revised Ctrl-F6 region navigation and held Ctrl-Shift-N window creation.
Herdr text/caret/change events, workspace reorder, pane resize/restoration, and stable daemon identities pass again.
Core text/caret/observing panes, audible announcements, Core resize, and selected-tab backdrop contrast remain pending.
Kata `q72g` stays open; these images do not qualify those separate requirements.

## Real Core Windows qualification plan

The [official versioned manifest](https://releases.herdr.dev/core/0.1.21/manifest.json) publishes generation 1, version 0.1.21, for Windows x64.
The first investigation reads that manifest directly without downloading or executing a binary.
Its archive digest is `a8984634c35b58ba82d0aec078d5a5e6004da01d5a25b3daf822ab5e31c7312b`.
The [pinned installer](https://github.com/herdrdev/herdr-core/blob/v0.1.21/distribution/install.ps1) requires the adjacent `conpty/` directory.
Use private archive staging rather than the installer, which changes user installation state.
This remains a plan; no native Core qualification follows from release availability.

- [ ] Stage the pinned, hash-verified archive only on the isolated x64 CI runner.
- [ ] Select it explicitly through `HERDR_CORE_TEST_BINARY`; retain the archive's ConPTY files.
- [ ] Reuse the cleared sandbox with a unique `HC_SESSION`, explicit `HC_SOCKET`, and private state paths.
- [ ] Own the exact `serve` child; stop only its private Core and reap that child during cleanup.
- [ ] Enable Core in the private prelaunch configuration and use the real `run new_core_terminal` route.
- [ ] Create three actual panes and read their exact IDs through the app and Core separately.
- [ ] Exercise native split, held-close, cancel-first consent, and captured-target preservation.
- [ ] Verify Core text, change events, caret lines, resize/restoration, and observe-only behavior through native UI Automation.
- [ ] Select the focused Core document by PID, HWND, ancestry, and bounds; split panes share the name Terminal.
- [ ] Keep strict pipe-owner validation; a runtime mismatch is a blocker, not permission to relax the gate.
- [ ] Review lifecycle and input cleanup independently before running the fixture.

Windows Ctrl-Shift-W does not establish macOS AppKit Cmd-W routing.
Audible screen-reader output and selected-tab backdrop contrast remain separate native gates.

The follow-up archive inspection downloads the exact versioned Windows zip into temporary storage.
Its 3,862,941 bytes match the manifest's SHA-256 above.
The archive contains `herdr-core.exe`, `conpty/conpty.dll`, and `conpty/x64/OpenConsole.exe`.
It also includes an Apache-2.0 license and third-party notices, which the private staging step must retain.
That inspection extracts or executes no archive member and installs no local runtime.
This verifies the fixture's input artifact; strict ownership and native behavior remain untested.

### Staging implementation candidate

The existing Windows staging script now accepts `--core` for this exact pinned release.
Its default CLI still stages the existing Herdr 0.9.5 fixture.
Both releases share bounded download, hash verification, Windows path checks, and extraction.
Core's executable, ConPTY files, and notices must exist before extraction starts.
Nonempty destinations are refused and preserved.
The fixture User-Agent names Spaceport; the release host refuses Python's default User-Agent with HTTP 403.
No credentials or installer are used.

The real `--core` CLI scenario fails first against the old usage contract.
All five staging tests then pass, including both actual release downloads.
Those tests extract into temporary directories and execute neither staged runtime.
They cover traversal, alternate streams, missing ConPTY, digest mismatch, and destination preservation.
Without explicit `SPACEPORT_TEST_RUNTIME_DOWNLOAD=1`, the three archive guards pass and both network cases stay skipped.
CI's existing script-check step now runs those offline guards.
Logs: `/tmp/spaceport-9s44-core-staging-{red,green,offline}.log`.
Local checks use uv with an isolated temporary project; CI retains its existing standard-library Python command.
Independent review finds no material archive trust, bounds, path, cleanup, or default-contract issue.
This implements the staging prerequisite; real Windows Core startup and native qualification remain pending.

### Real Core text candidate

The Windows harness now starts the explicitly selected Core binary in a separate private state directory.
One nonce supplies its exact pipe and session, including the prepared shutdown command.
The fixture owns its `serve` child and preserves private state if cleanup fails.
It retains strict pipe-owner checks and never uses an installer or discovers a personal daemon.

The actual app command creates one terminal, whose address must match Core's independent terminal listing.
Native UI Automation then checks text, caret-line bounds, and TextChanged delivery through the existing helper.
An observe-only runtime stream must independently contain the marker typed through that native helper.
The actual runtime must retain exactly one terminal after the check.
The fixture removes its event observer, stops its private Core, and reaps its owned child.

Both Windows GNU feature configurations pass the targeted accessibility-harness Clippy check with warnings denied.
Logs: `/tmp/spaceport-9s44-core-native-windows-{all,minimal}.log`.
Independent read-only review finds no material lifecycle, isolation, targeting, or evidence issue.
CI runs this namespace separately, after staging Core, even if another UIA case fails.
Actual Windows execution remains pending.
This first case does not qualify Core close, splits, observing-pane input refusal, resize, speech, or macOS Cmd-W.

### First Core execution on `daab7134e`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44649782332/job/136001693950) stages and starts the actual pinned Core runtime.
Connection, strict pipe ownership, generation 1, and the initially empty runtime assertions pass before the next failure.
The fixture then places `--json` before `run`, which the documented launch parser refuses.
No Core terminal is created, and native Core text remains unqualified.

All three fixture calls now put the command family before `--json`, as `docs/cli.md` requires.
The existing control-command parser scope passes all 12 cases.
Log: `/tmp/spaceport-9s44-core-cli-contract.log`.
Independent review confirms that only CLI argument order changes.
Native execution of the corrected fixture remains pending.

### Native observing-pane candidate

A second case opens the actual app mirror route for the same runtime terminal.
It finds the new tab through that exact terminal address and focuses its native document.
The existing helper registers its TextChanged observer before writing a private readiness flag.
A scoped producer waits for that flag, then writes one authored command through the exact Core attachment.
The observing view receives no authored keyboard input.

The helper requires fresh marker text, a native change event, an unselected caret, and contained line bounds.
Observer disposal must complete before the test accepts its receipt.
The test preserves the sole runtime terminal and both captured app tab IDs.
The producer joins before fixture cleanup, and helper failures retain the producer result.
The original Text action and early-return Resize action remain unchanged.

Both Windows feature configurations pass the targeted accessibility-harness Clippy check with warnings denied.
Logs: `/tmp/spaceport-9s44-core-observe-windows-{all,minimal}.log`.
Independent read-only review finds no material targeting, event-ordering, lifetime, or helper regression.
Actual Windows execution and existing native helper regressions remain pending.
This case does not qualify observing-pane input refusal, Core resize, audible speech, or macOS Cmd-W.

### Native Core text qualification on `23cab5b4b`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44652883114/job/136012085170) passes the actual Core terminal case.
The app tab matches Core's terminal address before native input starts.
The helper reads the authored output line, receives TextChanged, and exposes the unselected caret and contained line bounds.
Its observer is removed; an independent runtime stream also contains the native typed marker.
The terminal survives, and the exact private Core shuts down and reaps successfully before the test returns.
The [receipt](review-squad/core-native/qualification-23cab5b4b.json) records this scope and its remaining gates.

The same job passes eight GUI cases, pipe ownership, and 23 non-Core UIA cases.
Its separate image case fails pixel validation, so the native job remains red.
This passing Core case does not qualify observing panes, Core resize, audible speech, backdrop contrast, or macOS Cmd-W.

### Native observing-pane qualification on `174db4857`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44657637800/job/136027900757) passes both Core cases.
The observing receipt occurs at `2026-11-20T02:15:17.9733456Z`.
It confirms fresh text, TextChanged, an unselected caret, contained line bounds, and observer removal.
The case preserves the runtime terminal and both app tabs, then cleans up the exact private runtime.
The ordinary Core case also passes at `2026-11-20T02:15:24.0750787Z`.
The [receipt](review-squad/core-native/qualification-174db4857.json) records both results.

This job still fails its separate image scroll-capture wait.
Passing Core cases do not qualify resize, observing-pane input refusal, speech, backdrop contrast, or macOS Cmd-W.

### Native Core resize candidate

The new case creates two real Core terminals through the existing app split command.
It focuses the right native document and uses the private Herdr profile's stock Ctrl-B, r resize mode.
The app routes Left and Right to Core's existing split geometry commands.
Native caret-line bounds must move left and return within one pixel.
The owned window and fixed pane edges must stay fixed, and the divider gap must remain constant.
An independent runtime stream must contain the marker typed into that exact right terminal.
Both runtime terminal addresses must survive unchanged.

Both Windows feature configurations pass the targeted accessibility-harness Clippy check with warnings denied.
Logs: `/tmp/spaceport-9s44-core-resize-windows-{all,minimal}.log`.
Independent read-only review finds no material targeting, geometry, lifecycle, or existing-helper regression.
PowerShell is unavailable locally; actual native execution remains assigned to Windows CI.
This case does not qualify a Core-only default keymap, macOS Cmd-W, or audible speech.

### Native Core resize qualification on `a8f7bd371`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44661227704/job/136039952924) passes all three Core cases.
The resize receipt occurs at `2026-11-20T02:43:15.3497430Z`.
Actual prefix, Left, and Right move and restore the right pane's native caret-line bounds.
Fixed window and pane edges and the divider gap remain stable.
The independent Core stream contains the typed marker, and both terminal addresses survive unchanged.
Exact private Core shutdown and child reaping succeed before the case returns.
Ordinary and observing text cases also pass on this same source.
The [combined receipt](review-squad/core-native/qualification-a8f7bd371.json) records all three cases.
This qualifies the existing resize alternative, while speech, native tab contrast, and macOS Cmd-W remain open.

### Selected-tab native capture candidate

Use the existing isolated Windows app and real Herdr fixture.
Capture one selected tab before and after actual keyboard-region entry and native tab focus.
Cover a light theme and a dark theme with opaque chrome and a private authored image backdrop.
Require the exact foreground window, selected tab, terminal focus, released modifiers, and stable contained bounds.
Capture only the owned window, with bounded PNG output and unchanged private preferences.
The image backdrop covers the desktop; do not capture personal desktop content.
Inspect every pair and measure the painted ring against its actual neighboring fill before claiming contrast qualification.
Do not treat a configured color, focus event, or uninspected PNG as native contrast proof.

The candidate uses Classic tabs with Nord and Catppuccin Latte.
Image cases use a private two-tone PNG at the existing 0.75 image opacity.
Two separated native terminal samples must show that image before either tab capture runs.
The helper emits only the selected tab's cropped rectangle, never a full desktop screenshot.
Each PNG has a 128 KiB bound, and each capture checks exact foreground and rectangle-point ownership.
The helper restores its thread DPI context and disposes temporary drawing resources on all paths.
An explicit zero-window case must refuse capture before native input starts.
Both feature configurations pass Windows accessibility-harness Clippy with warnings denied.
Logs: `/tmp/spaceport-9s44-tab-contrast-windows-{all,minimal}.log`.
Independent read-only review finds no material source-level ownership, provenance, cleanup, or helper syntax issue.
PowerShell and C# execution remain pending Windows CI.
Actual contrast measurements, all eight image inspections, and non-Classic layout captures remain unqualified.
This candidate covers image-backed translucency, not desktop material composition.

The screen copy follows [CopyFromScreen](https://learn.microsoft.com/en-us/dotnet/api/system.drawing.graphics.copyfromscreen).
The helper restores the context returned by [SetThreadDpiAwarenessContext](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setthreaddpiawarenesscontext).

### Native tab contrast failure on `1679d74f0`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44663710320/job/136048031195) passes all 25 non-Core UIA cases.
The workspace-name acknowledgement and four selected-tab capture pairs execute successfully.
Root decodes and views all eight original crops, then verifies their dimensions and hashes.
The [receipt and crops](review-squad/tab-focus/contrast-1679d74f0.json) preserve every measurement.

| Actual selected fill | Minimum painted ring ratio |
| --- | --- |
| Dark, opaque | 3.00396 |
| Dark, image backdrop | 3.09499 |
| Light, opaque | 3.00113 |
| Light, image backdrop | **2.85149: fails the 3:1 target** |

The focus policy chooses accent ink against the layout's nominal opaque fills.
The image-backed fill differs after composition, so that policy cannot guarantee the painted ratio.
This is a real remaining defect; `q72g` stays open.

The correction uses two opaque inset bands only when the tab fill is translucent.
Opaque tabs keep their current accent ring.
Black and white bands use the two-color pattern from [W3C C40](https://www.w3.org/WAI/WCAG22/Techniques/css/C40).
That technique alone does not qualify inset indicators or image backgrounds; actual pixel checks remain required.
They add no layout width and do not change focus or input ownership.
A real offscreen Metal regression first reproduces the captured fill's ratios of 2.8514864 and 1.0.
It then checks the same focus painter against captured, black, white, grey, and accent-colored backgrounds.
The offscreen check opens no Mac desktop window and makes no native keyboard or speech claim.
Corrected Windows captures and their measurements remain pending.

### Core resize marker observation on `1679d74f0`

The same job passes ordinary and observing Core text cases.
Its Core resize case sees the authored line before input, then no longer sees it after Left.
The helper returns early on that missing marker before measuring the new caret geometry.
This result does not establish whether the native resize geometry moved.

The helper now separates initial authored-output readiness from later Core caret measurements.
The initial line and independent runtime byte stream still prove exact input delivery.
Every Herdr stage still requires its marker.
Later Core stages still require the current unselected caret, contained line bounds, fixed edges, divider gap, movement, and restoration.
Stage diagnostics report the actual marker boolean instead of printing a constant success value.
The final runtime identity and exact cleanup assertions remain unchanged.
This does not claim that old Core output stays visible or explain its redraw behavior.
Corrected native execution remains pending.

The new offscreen Metal case fails first on the captured light-image fill at ratios `[2.8514864, 1.0]`.
It passes after the two-band correction across all five captured/extreme background controls.
Logs: `/tmp/spaceport-9s44-tab-halo-{red,green}.log`.
Independent review finds no material focus, layout, fixture, or input-proof regression.
The focus/tab consumer scope passes ten cases; its separately executed Metal opt-in is skipped in that ordinary batch.
Host binary/test Clippy and both Windows binary/native-harness feature configurations pass with warnings denied.
Formatting, file-size, sorted regions, operations, localization, commit subjects, and diff checks pass.
The existing large debug-link unwind warning and external `block` future-compatibility notice remain unchanged.
Corrected native Windows pixels and Core resize execution still remain pending.

On `b94978e3b`, default-feature macOS CI finds the raster module missing its integration-feature gate.
The renderer API itself is available only with `gpui_platform/test-support`, enabled by the app's `integration-test` feature.
The module now uses the same macOS-and-integration gate as the existing image Metal tests.
Host binary/test Clippy without default features passes after correction.
Log: `/tmp/spaceport-9s44-tab-halo-default-gate.log`.
The correction does not change the tab painter or turn this opt-in raster test into native keyboard qualification.
