# Review squad fixes

Scope: Kata `9s44`, all 53 child findings from 2026-11-16.
Base: `723be121f` on `main`.
Worktree: `.worktrees/review-squad-fixes`, branch `fix/review-squad-findings`.

Integration now proceeds in `.worktrees/review-squad-integration` on `fix/review-squad-integration`.
Its pinned main base is `91bda83a77e7e02bbc6d9ba02887cb95dc3876b9`.
The original fixes checkout remains intact.
See [integration evidence](review-main-integration.md) for current checks and remaining gates.

Doctor Biz authorized the reported fixes, including their specific security changes, and validation.
On 2026-11-18, Doctor Biz authorized committing and opening a public draft PR for the reviewed checkpoint.
That approval includes its necessary push; it excludes merging, releasing, and unrelated security changes.
Keep existing task worktrees intact. Adapt useful work against this base.

On 2026-11-17, Doctor Biz authorized closing completed Kata child issues.
Close each issue after its fix, required checks, and review finish.
Keep incomplete findings and the umbrella open.
Keep required native and platform qualification pending where it blocks completion.

On 2026-11-17, Doctor Biz clarified that Herdr 1 means Herdr TUI and its support takes priority.
He answered the scope question about restoring the removed workspace-close route.
Restore that action with the explicit confirmation described in the question.
This supersedes the old route-cut default for workspace closing; it does not authorize checkout deletion.

## Acceptance

Resolve each child against its actual report, callers, and regression evidence.
Run focused checks and the repository's quick validators.
Obtain an independent review for lifecycle, security, or shared-contract changes.
Record native and platform checks separately from headless results.
Do not close the umbrella while any required finding remains unresolved.

## Earlier checks

Read the implementation checkpoint below for current owners and remaining work.

Implemented and verified `bs4s`, `hv7g`, and `pc1q` locally.
See [shutdown evidence](review-herdr-shutdown.md) and [SSH policy evidence](review-unverified-ssh.md).
CLI parser checks passed 60 unit tests and 22 process tests.
Its red logs are `/tmp/spaceport-qnz9/cli-parser-{red,boundary-red}.log`.
Root reviewed the CLI and SSH diffs without material findings.

Saved-tab recovery passed independent review after durable ID reservations and pre-spawn capacity reservations were added.
Browser checks passed 122 tests. Core checks passed 135 tests, with eight live tests ignored.
Destructive confirmation checks passed in real native windows at two widths and both themes.
Final screenshot capture now waits for the modal animation after its first draw.
Worker panic handling and Windows safety fixes passed independent root review.
Windows checks passed cross-compilation only. Native Windows and restored hosted jobs remain pending CI.
CLI targeting, transport exits, and bounded waiting capacity passed root review.
Those checks passed 125 control, 20 socket, and six executable transport tests.
Follow-up CLI reports, tunnel crash cleanup, and chat upload bounds are active.
Root is finishing keyboard recovery, missing-installation retry, hover ownership, and settings permission checks.
Those recovery, hover, and settings permission fixes now pass their focused checks.
Settings permission preservation also passed independent data-integrity review and executable checks.
The corrected `m225` regression failed without its guard, then passed with it restored.
Independent review approved the permission-click guard.
Core backpressure (`38y7`) passed 138 Core tests and independent concurrency review.
Its extra view-consumer assertion passed the final scoped run.

The original task worktrees remain untouched.
Final source-stable checks must clean changed workspace members under the shared Cargo lock.
The shared cache can otherwise reuse a sibling worktree's member artifacts.

`bwm8` passed root state/data-integrity review and focused path, diagnostics, and executable checks.
Its corrected preview fixture reads the request before replying; the full executable CLI batch passed 43 tests.
`fn64` now uses a shared cancellable pipe capture with bounded retention and one operation deadline.
Real pipe tests exposed both the original join and a blocked-reader leak in the first fix.
The client suite passed 409 tests, with four opt-in tests ignored.
Native and Windows cross-target client Clippy passed, as did the retained GPUI Clippy run.
That log did not establish GPUI all-feature coverage.
A fresh all-target, all-feature check passed with the validated pinned Sparkle SDK.
Final isolated device tests passed, and independent lifecycle review found no material defects.
See [pipe deadline evidence](review-pipe-deadlines.md).
`05be` now distinguishes Blocked squares from Done circles in the shared default marker renderer.
Its red/green regression and six agent presentation tests passed, including actual headless painted shapes.
See [status shape evidence](review-status-shapes.md).
Doctor Biz requested that native keyboard and performance checks remain pending.
`nz6w` now checks status versions before schemas, bounds terminal hello retries, and retains Unix socket ownership.
Its real socket regressions pass; all 80 control-protocol tests and Windows cross-target Clippy pass.
The Core late-answer fix already existed in base `69eda3b53`; its ten lifecycle tests pass.
Independent socket lifecycle review found no material defects.
See [protocol edge evidence](review-protocol-edges.md).
`j2p2` passed receipt, CLI, and true all-feature Clippy checks.
Root approved the follow-up typed Core timeout and lost-outcome mapping after nine chat tests passed.
`n6a1` passed root review after preserving accepted local history writes through Settings close.
Its final Settings, persistence, history, path, and prefix checks passed.
See [settings ownership evidence](review-settings-ownership.md).
`dzyx` passed final shutdown and observer-pruning tests after root's lifecycle review.
Its final all-feature Clippy passed with the validated Sparkle SDK.
`agbt` now belongs to cli_contracts after its verified repeat regressions.
Current agents own repeat-safe shortcuts (cli_contracts), OS accessibility preferences (chat_uploads),
and Windows coverage plus workspace-close review (tunnel_cleanup).
Root prioritizes confirmed Herdr TUI workspace closing under Doctor Biz's clarification.
`knqh` now passes independent destructive-action review and the complete scoped check script.
It passed 172 endpoint tests, 183 workspace tests, CLI discovery, and true all-feature Clippy.
An explicitly selected real Herdr closed only its fixture workspace and preserved files and other sessions.
See [Herdr TUI workspace evidence](review-herdr-tui-workspaces.md) for the backend race limitation.
`krqj` passed OS policy, motion, Settings, picker, Logs, and real portal-wire checks.
Root approved its async ownership and palette consumers; native OS qualification remains pending.
`jemh` clarifies the two Core settings and hidden-tab recovery without changing D69 behavior.
`agbt` also restores the public Windows keyboard mapper and configured-key ownership.
Root approved the final mapper review after checking its cached lookup implementation.
Its keymap, prefix, repeat, shifted-key, and true all-feature Clippy checks passed.
`3wd4` passed its platform-default, custom-precedence, real-wire, and all-feature lint checks.
Root approved the implementation; native Linux and Windows keyboard qualification remains pending.
Root completed and independently reviewed `d378` composer bounds and prepared-layout caching.
Three native editing regressions failed before the prompt-size guard, then all 11 input tests passed.
All 13 input and seven chat-composer integration tests now pass.
The clipboard owner now owns the shared reader, field paste hooks, chat route, and Core preparation envelope.
The asynchronous clipboard part remains open; no complete `d378` result is claimed.

Kata now records 31 closed children and 22 open children, verified by a fresh read on 2026-11-18.
The umbrella remains open.
The closure [ledger](review-squad/kata-closures.jsonl) retains issue and operation IDs.
Local checkboxes below record implemented fixes; they do not imply every native gate passed.
Open findings retain their individual implementation and qualification gates.
Image scrolling (`gfde`) still needs its aggregate and native acceptance evidence.
Working-session redraws (`gg9v`) need a foreground release profile before further changes.
Error language (`cx2r`) also needs reconciliation with PR 123's additional protocol copy.

Doctor Biz requested notice at a stopping point.
He has received checkpoint notices and has not requested a pause.
The goal remains active.
The closure ledger records each later closure and its freshly verified evidence.

## Current implementation checkpoint, November 18

Close Others and Core-only popup ownership pass focused checks and independent lifecycle review.
One typed menu origin separates client ownership from legacy runtime authorization.
See [popup ownership](review-core-popup-ownership.md) for the corrected target and connection fences.

Settings keyboard navigation now passes all 267 Settings tests and independent review.
Host and Windows cross-target all-feature Clippy pass for that Settings checkpoint.
Native keyboard and screen-reader qualification remain pending; `d0n2` stays open.

Async clipboard routing now passes its corrected reader, field, Core, and framed-chat checks.
Independent review caught a paste cancellation that discarded unrelated queued input.
Per-slot cancellation now preserves accepted input, resize, and focus commands in FIFO order.
Its deterministic worker regression and aggregate lint pass.
See [clipboard evidence](review-clipboard-reads.md).

Error feedback passes focused checks and independent review.
Same-owner notice updates preserve pauses only for the exact notice ID.
The common overlay keeps warnings visible in terminal, Devices, Core, and Zen views.
The named-window Messages command also passes its control-socket check.
See [error-feedback evidence](review-error-feedback.md).

The colour-only glyph-cache fix and cold-entry replacement pass all 32 painter tests.
Cold-entry replacement also passes independent review and preserves the tested oversized working set.
Other painting and polling costs remain open in [partial evidence](review-paint-poll-costs.md).

Async clipboard, error feedback, and chat accessibility now pass independent review and the aggregate lint checkpoint.
Whole GPUI host and Windows GNU all-target, all-feature Clippy pass with warnings denied.
Logs: `/tmp/spaceport-qnz9/2njz-host-clippy-source-stable.log` and `2njz-windows-clippy-source-stable.log`.
Those lint results precede the latest popup families; a fresh aggregate boundary remains pending.
The final chat scope passes 544 tests, including nested tool-step semantics.
Native input, visual, accessibility, and platform qualification remain pending for those findings.
Their Kata children remain open, as Doctor Biz requested for pending native checks.

Global and device Themes, Group, ConfirmClose, and Keybinds departure slices also pass focused tests and review.
The application popup passes thirteen cases and independent lifecycle review.
See [application-menu evidence](review-application-menu-departure.md).
The Host family passes 46 cases and independent lifecycle review.
See [Host evidence](review-host-popup-departure.md).
Workspace checks exposed shell and status scroll jumps; the fixes and final review now pass.
Its final scopes pass 4/34/10/9/74 cases in [Workspace evidence](review-workspace-popup-departure.md).
Tab passes fourteen consumers, including detached safety, field release, and exact destination scroll.
The invisible-paint cleanup also passes its actual regression and independent review.
Usage passes 22 cases and independent lifecycle review in [Usage evidence](review-usage-popup-departure.md).
Pane passes thirty consumers and the exact owned-child process check, with independent review.
Core workspace forms pass focused tests and independent lifecycle review.
Device setup passes its appearance, device, private-catalog, passive-control, and exact-scroll checks.
Devices and final shared popup lifecycle qualification now pass independent review.
The popup finding remains open for required native visual and keyboard checks.

Clipboard completed Devices; Popup completed shared lifecycle and sidebar indexing.
Settings completed shared listening-port scans.
Root owns integration and the remaining performance work.
The parser and image investigations were read-only through this qualified checkpoint.
The first consolidated build stopped at Devices extraction type/import errors before running any baseline.
The corrected consolidated artifact is `/tmp/spaceport-9s44-gpui.ZcCcNN/spaceport-tests`.
It proved the popup-image cache, sidebar visit budget, shared scan, and popup cleanup runtime failures.
The next artifact is `/tmp/spaceport-9s44-gpui.xD8o8D/spaceport-tests`.
The complete region scope passes seven checks and independent review.
Shared scanner checks pass 56 cases, plus five control-socket consumers, with independent lifecycle review.
The initial combined popup run exposed an 80-pixel session-modal scroll jump.
Red: `/tmp/spaceport-9s44-appear-combined.log`.
Sidebar qualification passes 399 sidebar, 124 Palette, and 60 state checks, with independent review.
The Devices owner corrected invalid catalog and unsnapped-height fixture assumptions without changing production behavior.
The final immutable artifact is `/tmp/spaceport-9s44-gpui.HsLaYo/spaceport-tests`.
The complete popup appearance scope now passes 65 cases, with one private subprocess entry point ignored directly.
Its parent invokes that subprocess explicitly.
Devices passes 28 cases, management seven, and overview 32, with zero failures or ignores.
Both actual cancel-to-Create and cancel-to-Delete transitions now start at their own top.
The final scoped reruns pass 22 Pane, 14 Tab, three Workspace, and seven Host-control tests.
Root approved final shared lifecycle, passive controls, owner release, and accepted-task preservation.
See [Devices evidence](review-devices-picker-departure.md) and [popup evidence](review-popup-close-motion.md).
Whole GPUI host and Windows GNU all-target, all-feature Clippy pass with warnings denied.
Logs: `/tmp/spaceport-9s44-host-clippy-qualified.log` and `spaceport-9s44-windows-clippy-qualified.log` in `/tmp`.
The host reports only the existing external `block` future-incompatibility notice.
Formatting, file-size, sorted-region, i18n 2174/114, and the spec integrity gate pass at this boundary.
The spec gate includes 22 tool tests; it does not establish application conformance or native qualification.
Streaming parsing and main-grid image ownership still need resolution in the performance finding.
Release-mode profiling and native qualification remain pending.

## Next performance slice

Root owns a conservative rich-parser work regression and synthetic streaming release benchmark.
The new test counts bytes entering the actual math-protection parser.
It runs real Append changes through Transcript::apply and Prepared::update.
The intended fast path applies only when input contains neither dollar nor backslash.
The actual ordinary Markdown regression failed before the fast path.
Root recorded the release baseline before implementing it.
The paired release workload improved ordinary preparation by 17.5 percent, with a flat math control.
All 545 debug chat consumers now pass; the ignored release benchmark passed separately.
Independent semantic review approved the change.
See [streaming evidence](review-streaming-parse-costs.md) for limits and the corrected release-UI verification mistake.

Popup owns tests for stationary main-image reuse under [the bounded plan](review-main-image-region-cache.md).
Use the same presented Picture's frame and images.
The first main-image implementation keeps sprites uncached and prepares one batch for its two paint passes.
Preserve existing popup preparation and ordering.
Do not alter unverified atlas-pressure policy or enable image smooth scrolling without ownership proof.
All Cargo remains serialized by Root.
All seven image cases pass from `/tmp/spaceport-9s44-gpui.EcPUAT/spaceport-tests`.
Unchanged frames invoke zero cell callbacks while retaining cached layer order and real image uploads.
Painter 50, region 13, scroll 15, presentation six, and input-settings four pass on unchanged production.
Those scopes overlap and must not be added together.
Root independently approved image ownership, ordering, geometry, and resource bounds.
Fresh host and Windows GNU all-target, all-feature Clippy pass with warnings denied.
Logs: `/tmp/spaceport-9s44-performance-{host,windows}-clippy-final.log`.
Formatting, file size, sorted regions, i18n 2174/114, and diff checks pass.
The application error wrapper's stale copy assertion was corrected; both error tests pass.
The strengthened schema fixture still proves redaction and its retained typed source chain.

This is the next local stopping point.
The native raster, keyboard, accessibility, platform, and release-profile gates remain pending.
Keep image smooth scrolling guarded because historical frames lack paired encoded-image leases.
Do not rewrite whole-window redraw scheduling without the required release profile.
Both performance findings remain open, alongside the native-held findings.
Kata remains 28 closed and 25 open; the umbrella remains open.
No commit, push, PR, hosted CI run, or new native check occurred.

## Findings

Current November 18 checkpoint: 34 children closed, 19 open, umbrella open.
Core guidance (`jemh`) closed after copy review, native General wrapping, and three Core-enabled catalog tests.
The closure ledger retains event 9502 and its UID.
The latest keyboard fixture timed out without becoming key, including after Doctor Biz confirmed readiness.
Both exact-window attempts remain failed qualification; neither reached the keyboard assertions.
Fresh-press confirmation (`fe28`) also closed after three regression passes, native confirmation checks, and independent review.
The current release dense rendering fixture passes.
The working-session harness now passes fifteen ordinary tests, including the captured scrollbar regression.
Its corrected native rerun passed seed but could not present the inactive, occluded GUI's first frame.
No CPU interval was sampled; no working-session performance conclusion is claimed.
Canonical host lint passes both all-features and no-default-features workspace/all-target configurations.
Canonical Windows GNU cross-lint also passes all-features and no-default-features workspace/all-target configurations.
This proves cross-compilation, not native Windows execution.
Root then started a narrow image-free-pane scrolling follow-up under `gfde`.
Its tests cover endpoint input, overlap transitions, predecessor cleanup, and a neighboring decoded image draw.
The per-pane overlap fix passes its regression and existing consumer filters, including real decoded-image painting.
Review found a cancellation redraw gap; its regression failed at zero redraw notifications.
The corrected handler and notification-before-automatic-render test now pass.
The current integration build passes three image-input, sixteen scroll, and eight decoded-image cases.
Independent production review approves the overlap and cancellation changes.
Latest canonical host and Windows GNU lint pass both configurations.
Native image-scroll qualification and full image-pane asset ownership remain pending; `gfde` stays open.

The later image-bearing scrolling slice now resolves history through the picture's existing assets.
It cancels before an asset identity changes and retains no additional encoded bytes.
Cells and images share historical bands and the same device-pixel offset.
Crossed placements split into stationary and moving fragments with nonoverlapping masks.
Thin fragments preserve their original texture mapping through destination content masks.
Both historical directions, serial replacement, and four display scales pass actual decoded-image tests.
All twenty-one integration-feature region tests pass.
Default scroll, painter, presentation, endpoint image-input, and wheel consumers also pass.
Independent review approved the corrected ownership and rendering path.
Canonical host and Windows GNU lint pass both configurations with warnings denied.
See [image-pane scrolling](review-image-pane-scroll.md) for exact failures, results, and native limits.
Native raster output, real timing, and the remaining `gfde` acceptance stay open.

Doctor Biz unlocked the desktop and native qualification resumed on November 18.
See [native qualification](review-native-qualification.md) for actual passes, new regressions, and remaining gaps.
The generic shell follow-up now passes its headless regressions; final native and lint checks remain pending.

- [x] `1qx4` — Ask before Cmd-W closes a Herdr Core pane
- [x] `fzwm` — Keep saved tabs when browser-tabs.json fails to load
- [x] `hv7g` — Turn off agent forwarding for hosts that skip host-key checks
- [x] `fe28` — Require a fresh key press to confirm End processes and Remove device
- [x] `zdrf` — End port tunnels and forwards when Spaceport crashes (closed after native Windows and Linux qualification)
- [x] `t45y` — Report a lost connection when the connection thread panics
- [x] `bs4s` — Stop restarting a Herdr server the user stopped
- [x] `jwa1` — Make the reconnect and missing-Herdr card buttons keyboard-reachable (headless keyboard checks; OS routing pending)
- [x] `2qr3` — Tell first-time users to install Herdr instead of retrying forever (closed after recorded native card confirmation and real framed recovery)
- [x] `1qgs` — Note that dismissing release notes marks them read for every client
- [x] `x4yd` — Document how Spaceport's SSH differs from herdr --remote
- [ ] `gfde` — Smaller painting and polling costs
- [x] `hkn1` — Remove stray cfg lines in two test files
- [x] `1wwk` — Clean up Windows identity, state folder and test coverage (native Windows execution pending CI)
- [x] `q72g` — Close smaller accessibility gaps. Implemented and independently reviewed; native limits remain in [evidence](review-accessibility-gaps.md).
- [x] `knqh` — Add closing Herdr TUI workspaces and brief Herdr users in the docs (native visuals and keyboard pending)
- [x] `jemh` — Make the Herdr Core experiment easier to understand (copy-only; closed after native General layout and catalog checks)
- [x] `p9wk` — Rewrite confusing settings descriptions
- [x] `ydr4` — Animate popups out and confirm Close Others once (native visual and keyboard checks pending)
- [x] `cx2r` — Replace protocol jargon in errors and settle one name per thing (native layout pending; see [evidence](review-error-language.md))
- [x] `j2p2` — Return the new chat's tab and pane from chat new (native agent startup pending)
- [x] `p3ft` — Give callers in Herdr Core panes a session guard
- [x] `4a60` — Bring CLI docs and help back in step
- [x] `x3b7` — Make CLI JSON errors and list shapes consistent
- [x] `nz6w` — Tidy protocol edge cases (native Windows execution pending CI)
- [x] `tbg4` — Keep a settings file's permissions when saving it
- [x] `gnr2` — Skip relative PATH entries when finding herdr and agents
- [x] `dzyx` — Coalesce preference saves and flush them on quit
- [x] `fn64` — Bound the pipe-reader joins after SSH probes (native Windows execution pending CI)
- [x] `38y7` — Don't drop Core input and resizes silently
- [x] `agbt` — Ignore key repeat for window, tab and command shortcuts (native keyboard checks pending)
- [x] `d378` — Cap chat composer pastes and read the clipboard off the UI thread (native input and platform qualification pending)
- [x] `m225` — Ignore a double-click's second click on chat permission buttons
- [x] `bwm8` — Separate the dev build's state and socket from the installed preview (native Windows execution pending CI)
- [ ] `gg9v` — Profile and cut whole-window redraws while agents work
- [x] `2rqs` — Bound chat uploads over SSH and clean up dropped files
- [x] `rder` — Stop the whole agent process tree on Windows (closed after native x64 and ARM regressions)
- [x] `q7h6` — Check the Herdr pipe's owner before connecting on Windows (native execution pending Windows CI)
- [x] `jvcf` — Restore Windows CI and run native GUI tests somewhere (closed after actual native execution; unrelated application failures remain)
- [x] `05be` — Show agent status with more than colour (native visual qualification pending)
- [x] `d0n2` — Make the Settings window keyboard-navigable (native keyboard and assistive-technology qualification pending)
- [x] `8zvq` — Raise secondary text contrast on light themes (native screen qualification pending)
- [x] `krqj` — Honour the OS reduce-motion setting (native OS qualification pending)
- [x] `2njz` — Make chat panes usable with a screen reader and the keyboard (native assistive-technology and keyboard qualification pending)
- [x] `3wd4` — Fix default shortcuts that collide on Linux and Windows (native keyboard qualification pending)
- [x] `kjkw` — Show errors somewhere the user can see them (native announcements, input, and visual qualification pending)
- [x] `p32e` — Keep the CLI usable while agents wait
- [x] `pc1q` — Stop the argument parser eating note and config text
- [x] `494d` — Fix the spaceport agent skill so it points agents at real commands (validator and 18 installer checks pass)
- [x] `4qzb` — Give 'app gone', 'version mismatch' and 'timed out' their own exit codes
- [x] `f7r9` — Keep tab commands in the caller's session and refuse waits on unknown tabs
- [x] `yftv` — Stop hover from moving the focus other Herdr clients type into
- [x] `n6a1` — Read Herdr's own settings, or stop saying Spaceport does (native Settings qualification pending)
