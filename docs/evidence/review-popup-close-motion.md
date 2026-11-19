<!-- ABOUTME: Records captured Close Others consent and popup departure verification. -->
<!-- ABOUTME: Preserves lifecycle boundaries, exact-target evidence, and remaining native checks. -->

# Popup departure and Close Others

Doctor Biz authorized implementation and validation of the popup/Close Others report (`ydr4`).
Doctor Biz later authorized closing fully completed Kata children.
Kata `ydr4` is closed after the scoped native qualification on `b20cca86f`.
Doctor Biz later authorized committing and publishing the reviewed checkpoint as a public draft PR.
Review and validation follow-ups stay on draft PR 125.

## Captured batch

The previous sequence detached native views immediately and prompted separately for each busy Herdr tab.
Cancelling a later prompt therefore left an already-partially-closed batch.
The mixed native/Herdr regression reproduced the first view closing before any complete consent.
Log: `/tmp/spaceport-qnz9/close-batch-red.log`.

Close Others now captures one exact batch before any effect.
The dialog lists every captured label and counts tabs, terminating Herdr panes, and client views.
Core tab closure detaches its client view through Store; it does not end Core runtime processes.
The dialog distinguishes those semantics.
It always asks, including page-only and idle batches or disabled single-tab prompts.
Cancel closes no tab and queues no close request.
The existing fresh-Enter and Cancel-first controls remain shared with other close confirmations.
The workspace-close target and group-membership implementation remain intact.

Before starting, the batch checks every target together.
It checks selected workspace/scope, selection epoch, connection generation, boot, native view identity, and Herdr pane membership.
It checks both the window's latest/pending snapshot and the connection inbox's latest/pending snapshot.
Each remaining step rechecks captured ownership.
A queue refusal, changed target, lost connection, or deadline stops the run visibly.
The status reports observed completed tabs and warns about unresolved sent closes when appropriate.
It neither replays nor rolls back a sent close.
Only one batch runs at a time.

The run retains serial terminal requests and the existing surface barrier.
Its deadline now survives observed tab removal while the following surface barrier remains unsettled.
The deterministic deadline regression failed before that correction.
Log: `/tmp/spaceport-qnz9/close-batch-barrier-red.log`.

The close-modal scope passed 19 tests after the correction.
They include actual framed-peer cancellation markers and whole-batch refusal before native-view detachment.
They cover changed generations, window-pending and inbox-pending added panes, and changed client views.
Log: `/tmp/spaceport-qnz9/close-batch-contracts-green.log`.
The group fixture's old fabricated boot was incompatible with its actual peer handshake.
Its previous fake-accept helper had hidden that mismatch.
The fixture now preserves the acknowledged boot and uses actual queued requests.
The group scope passed three cases using actual queued close requests.
The full endpoint sequence passed with both ACK/surface arrival orders.
Log: `/tmp/spaceport-qnz9/close-batch-group-wire-stable.log`.
The first retry had stopped at concurrent Settings signature edits.
Log: `/tmp/spaceport-qnz9/close-batch-group-wire-green.log`.
Aggregate formatting, file-size, sorted-region, and diff checks passed.
The first all-feature Clippy run found only missing test-scoped unwrap allowances.
Both owners corrected those allowances.
The final all-target, all-feature Clippy retry passed with `-D warnings` and the validated pinned Sparkle SDK.
Log: `/tmp/spaceport-qnz9/close-batch-clippy-final.log`.

## Departure motion

Implementation is in progress after the batch lifecycle checks.
Pinned GPUI exposes scene capture only to tests, and an AnyElement cannot be laid out twice.
Retaining live Palette or setup state would also retain reader tasks.
Departure state must therefore preserve only the data required to paint.
It must omit active reader ownership, mutation handlers, focus, and IME ownership.
The existing Devices departure path is the starting point.
Messages history requires only bounded paint data.

The reversible panel reducer covers interrupted entry, departure, and reopening.
Its cases distinguish popovers from dialogs and preserve partial visibility when motion reverses.
Palette departure now freezes visible text, visible ranked rows, filters, errors, preview data, and scroll offset.
It drops the live Palette and its readers when the dialog closes.
The shared Palette template omits activation handlers for frozen rows and filters.
The leaving search field paints plain text without a field entity or input handler.

The real headless regression first lost its visible rows after dismissal.
See `/tmp/spaceport-qnz9/ydr4-palette-departure-red.log`.
The same test passes with the frozen projection.
See `/tmp/spaceport-qnz9/ydr4-palette-departure-green.log`.
The complete motion scope passes 17 tests after the exact search-entity lifetime assertion.
See `/tmp/spaceport-qnz9/ydr4-palette-appear-final.log`.
The existing sidebar uses the same search debug selector, so field ownership uses a WeakEntity assertion.
That focused result does not prove every popup family preserves its closing content.
The footer and Messages now capture their rows before reset.
Incoming snapshots and new history entries cannot replace their departing content.
Their regressions failed before the captures existed.
See `/tmp/spaceport-qnz9/ydr4-footer-departure-red.log` and `ydr4-content-departure-runtime-red.log`.

The same runtime batch reproduced a Core-only footer panic at the legacy endpoint index.
The footer now keeps client commands and omits TUI connection rows when no legacy endpoint exists.
Root reviewed that endpoint guard without a material finding.

ConfirmClose now shares one prepared-content template between active and departing panels.
The snapshot holds only displayed text, checkbox/selection state, errors, and a scroll offset.
It retains no consent target, process handle, reader, or input owner.
The active consent still clears immediately on dismissal.
The departure regression first lost all confirmation controls after reset.
See `/tmp/spaceport-qnz9/ydr4-consent-departure-red.log`.
Frozen controls omit action handlers and accessibility action nodes.
The existing close/batch execution and its guards remain unchanged.
The detached-template test has no parent input shield.
It clicks stale Confirm, Cancel, and checkbox controls against replacement pane consent.
All three leave the replacement unchanged.
A real 24-target consent list scrolls before cancellation.
Its captured text and clipping bounds remain identical after active consent clears.
See `/tmp/spaceport-qnz9/ydr4-consent-isolation-runtime.log`.

The stale-handler fixture also failed with only the frozen handler guard disabled.
Stale Confirm then changed the replacement consent's error.
The throwaway guard check restored the exact owned source before releasing the Cargo lock.
See `/tmp/spaceport-qnz9/ydr4-consent-guard-red.log` and `ydr4-consent-guard-check.log`.
The restored full close-modal scope passes all 26 tests.
See `/tmp/spaceport-qnz9/ydr4-consent-final-green.log`.
Root approved the pure capture and unchanged consent lifecycle after reviewing those checks.

The scrolled Palette regression exposed a 3.5-pixel departure jump in the first frozen renderer.
Frozen row placement now uses the live list's measured height and direct row coordinates.
It retains only the visible row slice and a plain offset, without a scroll handle.
The test compares actual selected-row bounds before and after dismissal.
See `/tmp/spaceport-qnz9/ydr4-palette-scroll-artifact.log` for the failure.

The current complete motion scope passes 25 tests.
See `/tmp/spaceport-qnz9/ydr4-popup-content-retry.log`.
That includes the static-dialog owner's VersionMismatch, OpenLink, and release-note departure checks.
The same compiled artifact passed 24 close-modal, 103 Palette, two menu-reset, four feedback, and one Messages control test.
Logs use the `/tmp/spaceport-qnz9/ydr4-*-consumers-final.log` prefix.
The control log is `/tmp/spaceport-qnz9/ydr4-history-control-final.log`.
An initial guessed feedback filter selected zero tests; the corrected exact module ran all four cases.

Contextual tab/pane/workspace popovers, setup forms, DeviceTheme, Usage, and Devices forms still need complete paint-only projections.
The original generic Kept fields remain only for those unfinished families.
Their existence does not prove their departure content or input isolation works.
The Group popup's first compile stopped during the concurrent Themes refactor.
Its later runtime baseline failed because reset removed the visible rows.
See `/tmp/spaceport-qnz9/ydr4-group-departure-runtime-red.log`.
Group Paint now holds labels, icons, details, section headings, selection, and a plain offset.
It holds no Action, GroupId, TunnelKey, input owner, or reader.
The active and frozen menus share one row template.
Frozen rows receive no Action slice.
Frozen tunnel glyphs share the original glyph template without its tooltip or close handler.

All 30 Group consumers pass, including the actual departing row geometry.
A 64-port parsed fixture preserves its scroll after dismissal and new results.
Stale Split cannot affect a replacement group without a parent input shield.
The frozen close glyph leaves its exact owned tunnel child running.
The existing active close test still kills and reaps only its exact owned child.
See `/tmp/spaceport-qnz9/ydr4-group-consumers-retry.log`.
Root's independent final review approved Group and ConfirmClose.

The immutable Group verification artifact is `/tmp/spaceport-ydr4-group.2qjaMm/spaceport-tests`.
Use the matching command logs when judging its tested source boundary.
Global Themes and static-dialog evidence live in their separate reports.
Keybinds passed its separate Settings, editor, and motion scopes: 15, 8, and 31 tests.
DeviceTheme passed 21 focused tests and independent lifecycle review.
See [Keybinds departure](review-keybinds-popup-departure.md) and [DeviceTheme departure](review-device-theme-departure.md).
The custom Linux/Windows application popup now uses shared reversible motion.
Thirteen focused checks and independent lifecycle review pass.
See [application-menu departure](review-application-menu-departure.md).
Native OS menus have their own platform behavior.
Pane action, rename, movement, and process departure now have a separate [evidence report](review-pane-popup-departure.md).

Tab, RenameTab, and MoveTab now share active and passive templates.
The three-page runtime baseline lost every body before the paint capture existed.
See `/tmp/spaceport-qnz9/ydr4-tab-matrix-red.log`.
The capture holds labels, draft paint, errors, selection, and a plain scroll offset.
It retains no target, field entity, move request, or worker.
The real three-page geometry regression now passes.
See `/tmp/spaceport-qnz9/ydr4-tab-current-artifact.log`.
Three detached-template cases pass without the outer input shield.
Stale Rename, Cancel, Submit, and destination controls leave replacement state unchanged.
The old rename field releases while its paint remains.
Sixty-four real destination rows retain identical scrolled row and viewport bounds after reset.
See `/tmp/spaceport-qnz9/ydr4-tab-isolation-runtime.log`.
The destination fixture changes the boot before clicking, preventing any accidental host CLI launch.
It also asserts no error changes, so an attempted callback still fails the test.
Root's independent Tab review found no material issue.
The full consumer scope found one stale raw UnsupportedMethod message expectation.
The fixture now checks the typed error and the existing localized presentation.
The refreshed full Tab consumer scope passes all 14 tests.
See `/tmp/spaceport-qnz9/ydr4-tab-final-consumers.log`.
The immutable executable is `/tmp/spaceport-9s44-gpui.Dl8yq5/spaceport-tests`.

Closing before the first popup frame exposed retained paint with no visible departure.
The deterministic reducer regression failed at the retained-data assertion.
See `/tmp/spaceport-qnz9/ydr4-invisible-paint-red.log`.
Settlement now clears captured paint whenever no active or departing panel remains visible.
The cleanup regression passes with the fix.
See `/tmp/spaceport-qnz9/ydr4-invisible-paint-green.log`.
The final shared artifact also passes the cleanup test.
See `/tmp/spaceport-qnz9/ydr4-invisible-paint-final.log`.
Root independently reviewed the change and found no material issue.
The backdrop continues fading independently after panel paint releases.

The final shared renderer also needed a missing-capture guard.
Its real runtime baseline rebuilt Settings rows from active state despite an empty capture.
See `/tmp/spaceport-qnz9/ydr4-missing-capture-red.log`.
Every Page now identifies its passive family through an exhaustive Kept match.
Missing captures render empty content instead of calling an active renderer.
The guard regression passes in `/tmp/spaceport-qnz9/ydr4-final-capture-green.log`.
The obsolete error, input, unfolded-device fields and owner swap are removed.
Departure rendering no longer restores an active page or owner.

Replacement at the same anchor before another frame also retained the old capture.
The deterministic failure is `/tmp/spaceport-qnz9/ydr4-replacement-cleanup-red.log`.
Settlement now clears any capture with no visible departure, including replacement and first-frame cancellation.
Both cleanup cases pass in `/tmp/spaceport-qnz9/ydr4-final-cleanup-green.log`.

A real closing confirmation also reclaimed Cover::All during its departing draw.
The baseline first verifies the active cover, then expects Unknown after dismissal.
See `/tmp/spaceport-qnz9/ydr4-closed-cover-red.log`.
Cover writes now require an active panel.
The same test passes in `/tmp/spaceport-qnz9/ydr4-final-cover-green.log`.

Devices/session shell geometry now uses the captured session-modal flag.
Its passive footer underlay paints before the fading backdrop and modal.
Active and passive disclosures share motion::disclosure and identical clip/opacity math.
The shared appearance scope passes 64 cases with one session-scroll failure and one ignored private subprocess fixture.
The parent test separately invokes that private fixture.
See `/tmp/spaceport-9s44-appear-combined.log`.
The session-scroll regression exposed an exact 80-pixel jump after closing a long-error form.
A dedicated session scroll handle and plain captured offset now address that gap.
Their runtime qualification remains pending at this source boundary.

## Remaining verification

Root and the independent lifecycle reviewer approved the captured batch and Core ownership follow-up.
See [Core popup ownership](review-core-popup-ownership.md).
The final shared popup review awaits the session-scroll qualification and aggregate gates.
Actual native visual/input checks and performance qualification remain pending at Doctor Biz's request.

## Earlier checkpoint

Doctor Biz requested a checkpoint after the current batch checks on 2026-11-17.
Hold before starting motion implementation until work resumes.
No motion source was changed.
The inspected `menu/appear.rs`, `menu/appear/kept.rs`, and menu departure rendering retain their original behavior.

The batch implementation lives in `close_modal/batch.rs`, `close_modal/sequence.rs`, and the shared close-modal dispatcher/rendering.
Its tests live in `close_modal/tests/batch.rs`, `close_modal/sequence/tests.rs`, and existing group/endpoint sequence topics.
The new Fluent messages live in `locales/en-US/close-modal.ftl`.
Root's workspace target module remains intact.
Current source compiles and the focused runtime and lint checks pass.
Independent destructive/lifecycle review remains pending at this checkpoint.

On resume, first complete independent destructive/lifecycle review of the captured batch.
Then implement popup departure from necessary paint data only.
Preserve immediate reader cancellation and omit stale mutation handlers, focus, and IME ownership.
Cover palette, dialogs, popovers, and the new bounded Messages page.
Use `crate::motion` for timing and reverse interrupted motion from its current position.
Keep the complete batch and outgoing popup tests separate from native qualification.

## Earlier resume point

The goal resumed after the earlier checkpoint while Doctor Biz's optional pause choice remained unanswered.
The active task worktree contains shared reversible motion and multiple passive paint projections.
The shared popup renderer lives in `menu/chrome/render.rs` after a responsibility-based extraction.
Do not restore the earlier three-field Kept implementation or discard other owners' changes.
Qualify the final shared lifecycle and session-scroll corrections before marking `ydr4` complete.
Keep `window/render.rs` with root's error-feedback owner.

## Qualified local checkpoint, November 18

Every popup family now captures passive display data before its active owner resets.
The shared renderer never restores an active Page or swaps live input into the menu.
An exhaustive family guard refuses missing captures.
Closed dialogs no longer reclaim native Cover state.
Invisible or replaced departures release their captured data.

The final Devices forms preserve actual shell and list offsets.
Cancellation also clears the newly owned per-form scroll handle.
The actual 80-pixel departure and cancel-to-new-form failures now pass.
The Delete fixture preserves the existing saved-profile protection.
See [Devices evidence](review-devices-picker-departure.md).

The final immutable executable is `/tmp/spaceport-9s44-gpui.HsLaYo/spaceport-tests`.
All 65 appearance tests pass; one private subprocess entry point runs through its parent.
Final scoped consumers pass 28 Devices, seven management, 32 overview, 22 Pane, 14 Tab, and three Workspace cases.
Seven Host-control tests also pass after the Windows matrix correction.
Root approved the final independent shared lifecycle and Devices reviews after reading those logs.

Host and Windows GNU all-target, all-feature Clippy pass with warnings denied.
Logs: `/tmp/spaceport-9s44-{host,windows}-clippy-qualified.log`.
Formatting, file-size, sorted-region, i18n 2174/114, spec integrity, and diff checks pass.
Native visual, keyboard, glyph, and platform qualification remain pending.
The Kata child remains open; the local implementation checkbox records completed code work only.
Do not reimplement or discard this qualified popup work on resume.

## Remaining original finding scope, November 19

The original-scope audit finds no remaining production defect in the batch or popup paths.
The existing native workspace consent check exercises workspaces, not Close Others.
The native local-controls fixture does not observe an intermediate outgoing popup frame.

Use one private Windows scenario with three owned Herdr tabs.
Open Close Others through its actual menu and verify both targets and the aggregate counts.
Cancel and require all three tabs to survive.
Reopen and confirm once; require only the selected tab to remain without another prompt.
Capture actual native outgoing frames for a popover, dialog, and palette.
Retain the broader family coverage in the existing rendered tests.

Passive outgoing captures intentionally expose no accessibility nodes.
Tree disappearance alone therefore cannot prove outgoing painting.
Keep `ydr4` open for this bounded qualification.
Unrelated speech, Settings, Mac foreground, and release profiling do not gate this original finding.

## Native qualification candidate, November 19

The Windows accessibility suite adds a private three-tab Close Others scenario.
It uses the selected test daemon and the normal application.
It finds the group action beside the strip containing the selected tab.
The UIA helper restricts discovery and invocation to the exact application process.

The scenario checks both named targets and the aggregate terminal and view counts.
It presses Enter on the default Cancel action and requires all three tabs to survive.
It reopens the same batch and invokes its single confirmation.
Only the selected tab may remain, without a second prompt.
The private checkout marker and daemon must survive both paths.

A separate daemon-free native fixture captures popover, About dialog, and command palette scenes.
It uses the production renderer, real clock, and existing QA slow-motion control.
It reads the actual Windows motion preference and applies the same value to GPUI.
It never changes the OS preference or freezes time.
The authored background omits its terminal cursor to prevent unrelated blinking differences.

Animated mode requires an actual rendered departure between 25 and 80 percent visibility.
It compares that frame with the open scene and clean closed baseline.
Those comparisons prove scene changes, not outgoing text by themselves.
Inspect every emitted outgoing image for its actual popup body before closing the issue.
Backdrop-only differences cannot qualify the dialog or palette body.
The settled popup region must exactly match the baseline.
Palette dismissal must release the search entity and all outgoing paint.
Reduced-motion mode instead requires release and baseline pixels on the first dismissed draw.
A reduced-motion receipt does not qualify intermediate animated frames.

Each scene emits bounded PNG receipts for inspection.
Each image permits at most two million pixels and 128 KiB of encoded data.
Animated mode emits at most nine images; reduced-motion mode emits six.
On a comparison failure, the fixture also emits that scene's baseline before returning the error.
Encoding and pixel comparison run on the background executor.
The fixture keeps the existing native deadline.
Its read-only render metadata exists only in Windows integration builds.

The focused appearance and batch scopes pass 74 tests, with one private subprocess entry ignored directly.
The parent test invokes that subprocess entry.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- menu::appear::tests:: close_modal::tests::batch:: --nocapture`.
Log: `/tmp/spaceport-9s44-native-popup-consumers.log`.
Formatting, file size, sorting, operations, localization, and diff checks pass.
Host and Windows GNU lint pass with all features and without default features.
Log: `/tmp/spaceport-9s44-native-popup-final-lint.log`.
Independent popup review accepts the explicit manual content-inspection gate.
Independent consent and privacy review finds no remaining material issue.
The owner review confirms exact private targets, bounded captures, and unchanged operation lifetimes.
Native execution and image inspection remain pending for this candidate.
Keep `ydr4` open until those results qualify its original scope.

### First native result

Source `197d5c65c` fails the ARM native popup check.
Its popover reaches a real outgoing frame with remaining visibility `0.7685689`.
The settled region then differs from the clean baseline.
The failing comparison emitted no images on that source.
The x64 run passes reduced-motion popup checks and emits six complete PNG receipts.
The owner inspected all six images.
They show the collapsed sidebar's compact rail remains visible inside the comparison region.
The authored source supplies Working status to its workspaces, tabs, and agents.
Animated mode can therefore change unrelated status indicators between the baseline and settled frames.
The next candidate gives that authored background Idle status before drawing.
It preserves the strict comparison and actual OS motion preference.
Native verification of this fixture correction remains pending.

Comparison failures now preserve open, outgoing, settled, and baseline images before returning.
Reduced-motion comparison failures also preserve the first dismissed draw.
Normal successful animated receipts remain nine images; failure diagnostics add at most two images.
The six inspected reduced-motion images live in `/tmp/spaceport-9s44-departures-x64-reduced/`.
Their manifest records full PNG signatures, completion markers, sizes, and hashes.
Run: [44503324192](https://github.com/2389-research/spaceport/actions/runs/44503324192).
ARM job: [135488771829](https://github.com/2389-research/spaceport/actions/runs/44503324192/job/135488771829).

### Default tab labels after a close

The x64 native consent scenario preserves all targets on Cancel and reaches its final confirmation.
Its final tree contains two tabs, with automatic labels `1` and `2`.
The selected label `2` alone does not identify which stable tab survived.

The pinned Windows runtime is Herdr v0.9.5, commit `2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01`.
Its [display-name producer](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/src/workspace.rs#L442) numbers automatic labels by current position.
Its [tab producer](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/src/app/creation.rs#L202) preserves stable tab IDs and public numbers separately.
Closing the first target therefore changes the next target's automatic label.
The client incorrectly rejects that remaining target because its label differs from the captured label.

Batch capture now retains whether each label is custom.
Validation requires unchanged label ownership and exact custom names.
Automatic labels may renumber while stable IDs and every existing ownership fence remain unchanged.
Boot, connection, workspace, pane-set, and current/pending snapshot checks remain in force.

The regression first fails on default renumbering and an unchanged-text label-ownership transition.
The custom-name refusal already passes before the fix.
Log: `/tmp/spaceport-9s44-batch-labels-red.log`.
After the correction, all ten batch, label, and sequence checks pass.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- close_modal::tests::batch_labels:: close_modal::tests::batch:: close_modal::sequence::tests:: --nocapture`.
Log: `/tmp/spaceport-9s44-batch-labels-green.log`.
The existing poison fixture emits its expected caught worker panics.

The native fixture now reads the private daemon's real `herdr tab list` response.
It records the unique stable tab with public number `3` before consent.
Cancel must preserve the entire original ID set.
Confirmation must leave only that recorded ID and public number, now displayed with label `1`.
No custom names hide the automatic-renumbering regression.
Native qualification of the correction remains pending.

The corrected source passes host and Windows GNU lint, with and without default features.
Logs: `/tmp/spaceport-9s44-batch-labels-final-lint.log` and `/tmp/spaceport-9s44-batch-labels-observer-lint.log`.
Formatting, file size, sorting, operations, localization, commit, and diff checks pass.
Log: `/tmp/spaceport-9s44-batch-labels-reviewed-static.log`.
Independent review approves the actual identity correction, preserved fences, and native stable-ID assertions.
The owner reviewed all eleven changed files and found no remaining material issue.

## Qualified native result on `b20cca86f`

Both Windows native jobs pass on commit `b20cca86f7ba597e223029747ce9c0bc5798a5a4`.
The [x64 job](https://github.com/2389-research/spaceport/actions/runs/44507153995/job/135501954152) passes eight GUI and seven UIA tests, with none ignored.
The [ARM job](https://github.com/2389-research/spaceport/actions/runs/44507153995/job/135501954484) passes seven GUI tests, with none ignored.
The separate pipe-owner cases also pass.

The real private-daemon consent case emits `CLOSE_OTHERS native PASS`.
Cancel preserves every original daemon tab ID and the checkout marker.
One confirmation removes both captured targets, without another prompt.
The sole survivor retains its original stable ID and public number `3`, with automatic label `1`.
The private daemon remains running until exact-child test cleanup.

ARM reports animated mode with actual outgoing visibility values `0.774314`, `0.75152826`, and `0.69495124`.
The owner decoded and inspected all nine complete PNGs.
Every outgoing image retains its popover, About dialog, or palette body and readable text.
The dialog and palette evidence includes content beyond their fading backdrops.
All three settled images have the same complete PNG hash.
The strict baseline checks, active-owner reset, outgoing release, and weak search-entity release all pass.

The local manifest is `/tmp/spaceport-9s44-departures-arm-animated/manifest.json`.
The outgoing PNG hashes are:

| Scene | SHA-256 |
| --- | --- |
| Popover | `0397f3feb6ca889d7af932f98ca71522b9611a403d8cdd46b3bd8fb1da608205` |
| About dialog | `0c0bf701453d82f7cc6e7a06f90c63aabaf1f1505d69daa0e1eef5a0f426fbf8` |
| Palette | `d4b11927f514db8ca4d3f8b2e5c64c593590cf08de043c76d47e7733430e95d7` |

x64 separately passes reduced-motion mode and its first-dismissed-draw baseline/release checks.
These receipts complete `ydr4`'s original popup and single-batch consent scope.
Other accessibility, shortcut, foreground, and working-session profile findings remain separate.
Kata closure event `9788` records completion at `2026-11-19T10:49:26.709Z`.
Fresh issue and parent reads verify this child closed, with 43 children closed and ten open.
The complete [CI run](https://github.com/2389-research/spaceport/actions/runs/44507153995) passes all eighteen executed jobs; one unconfigured target skips.
The separate [supply-chain run](https://github.com/2389-research/spaceport/actions/runs/44507154272) also passes on the same source.
