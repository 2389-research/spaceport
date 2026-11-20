<!-- ABOUTME: Qualifies fractional image scrolling without extra encoded asset retention. -->
<!-- ABOUTME: Records actual regressions, image ownership, and remaining native verification. -->

# Image-pane scrolling

This is the remaining image-bearing pane slice of `gfde`.
The earlier change already allows image-free panes beside stationary images to scroll smoothly.
The image-bearing pane now uses the same fractional rows as its cells.

## Contract

Use the same device-pixel offset and historical source bands for cells and images.
Keep stationary image fragments outside the scrolling pane fixed.
Clip moving fragments to both their source band and the stationary pane viewport.
Preserve below-text and above-text ordering and equal-z placement order.
Prepare one main image epoch before either image pass.
Do not retain textures in cached terminal scenes.

Keep historical frames as metadata only, under their existing four-frame bound.
Resolve their image keys through the presented picture's existing asset collection.
Before replacing that collection, compare image serials for the retained scrolling placements.
If a needed asset disappears, appears, or changes identity, cancel the fractional gesture.
The authoritative whole-row picture then paints normally.
This avoids additional encoded image leases or larger byte, texture, placement, or decode limits.
Preserve stale-picture fencing and cancellation redraw behavior.

## Regression baseline

Two actual-window tests use a valid raw image through the existing asynchronous decoder and atlas upload.
Both fail because the image-bearing pane returns `None` instead of submitting its fractional row request.
The log is `/tmp/spaceport-9s44-image-pane-red.log`.
The first test follows the image's shifted geometry and decoded pixels.
The second checks historical edge painting and release when the existing encoded asset disappears.
These are headless window/renderer tests; they do not qualify native raster output.

## Verification to finish

- [x] Observe the expected runtime failures before changing production code.
- [x] Pass fractional geometry and historical edge regressions.
- [x] Cover stable identities, replacement, missing assets, and cross-pane fragments.
- [x] Pass default-feature image, scroll, presentation, and endpoint cancellation consumers.
- [x] Obtain independent ownership and rendering review.
- [x] Run quick validators and applicable host/Windows cross checks.
- [ ] Qualify actual native image scrolling and release timing.

The native Settings readiness question remains pending.
No native fixture launched during this slice.

## Rendering correction

The first implementation passed both decoded-image scenarios after a fixture correction.
The fixture must move the actual decorated cell row when it changes the scroll offset.
Its first green attempt omitted that movement and correctly produced no fractional history.
That failed attempt remains `/tmp/spaceport-9s44-image-pane-first-green.log`.
The corrected two-test pass is `/tmp/spaceport-9s44-image-pane-green.log`.

Independent review then found an image mapping problem in thin bands.
Pinned GPUI rounds cropped image bounds to whole atlas pixels.
Passing a thin destination fragment as the mapping rectangle changes the sampled source coordinates.
A real paint-call regression observed a five-pixel mapping instead of the original twenty pixels.
Its contrasting two-row decoded texture and actual atlas upload passed before that assertion failed.
The valid red is `/tmp/spaceport-9s44-image-band-mapping-red.log`.

Image regions now carry a separate source grid and destination mask.
The painter preserves the original image mapping and clips through `Window::with_content_mask`.
Both mask corners use GPUI's device-pixel snapping.
Adjacent fragments share an edge instead of expanding into overlapping masks.
The probe records actual paint-call mapping and the effective content mask.
It does not expose native raster pixels or final GPU UV tiles.

The final source review found no remaining material production issue.
The reverse-direction helper initially expected an upward row request.
That test-only expectation now follows the wheel direction.
The failed intermediate fourteen-case run is `/tmp/spaceport-9s44-image-bands-green.log`.
The final fourteen-case image pass is `/tmp/spaceport-9s44-image-bands-final.log`.

## Default-feature consumers

All commands ran the actual current unit executable with `--test-threads=1`.
The following scopes overlap and must not be added together.

| Scope | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| `smooth_scroll` | 20 | 0 | 0 |
| `terminal_painter` | 50 | 0 | 0 |
| `window::regions` | 20 | 0 | 0 |
| `presentation::tests` | 6 | 0 | 0 |
| `endpoint::lifecycle_tests::image_scroll` | 3 | 0 | 0 |
| `endpoint::lifecycle_tests::wheel` | 2 | 0 | 0 |

Logs use `/tmp/spaceport-9s44-image-consumer-<scope>.log`, with colons replaced by underscores.
The image cases include both historical directions and asset removal without additional encoded leases.
They also cover replacement under an unchanged surface pointer.
Crossed transparent images retain equal-z order and nonoverlapping masks at four display scales.
The cancellation consumer now uses an image pane without scroll metrics.
Images themselves no longer force that fallback.
The test still proves notification precedes automatic redraw and sends no whole-row input.

The integration-feature region suite passes all twenty-one cases.
This includes the actual sliding-cell source-band consumer and device-pixel geometry checks.
The log is `/tmp/spaceport-9s44-image-feature-regions.log`.
The first canonical host lint failed on one test's literal boolean comparison.
That assertion now uses `assert!` with the same condition and diagnostic.
Both canonical host lint legs pass with warnings denied.
They cover workspace/all-target checks with all features and without default features.
The log is `/tmp/spaceport-9s44-image-bands-host-lint-green.log`.
The existing external `block` future-incompatibility notice remains unchanged.
Both canonical Windows GNU lint legs also pass with warnings denied.
They cover workspace/all-target checks with all features and without default features.
The log is `/tmp/spaceport-9s44-image-bands-windows-lint.log`.
This proves cross-compilation, not native Windows execution.
Formatting, file-size, sorted-region, and diff checks pass.

Kata remains at thirty closed children and twenty-three open children after a fresh parent read.
The `gfde` finding and umbrella remain open for required remaining qualification.
No commit, push, PR, native launch, release, or issue closure occurred during this slice.

## Offscreen Metal raster qualification

Two macOS opt-in tests now render the actual compositor through GPUI's supported offscreen Metal backend.
They use `HeadlessAppContext`, `current_headless_renderer`, and real screenshot pixels.
The no-op text system draws only blank terminal cells; these tests make no native glyph claim.
No AppKit application, desktop window, daemon, or user clipboard is involved.
The tests require the existing `integration-test` feature and explicit `--ignored` selection.

Use the shared Cargo lock and target prefix from the handoff, then run:

```sh
cargo test --locked -p sheprd-gpui --features integration-test \
  --bin spaceport metal_pixels -- --ignored --test-threads=1
```

The first case compares a thin historical image fragment with its original sampled pixel.
The fragment must preserve the red row of a contrasting two-row image.
It also samples inside the translated image but below the stationary pane to prove actual clipping.
The second case compares crossed transparent fragments against an original single-pass alpha reference.
Both sides of the snapped boundary must match that reference.
Additional pixels prove that only the inside fragment moves.

Initial compilation exposed two private test helpers and one redundant path qualification.
The corrected fixture uses public `NoopTextSystem` and the supported renderer factory.
No GPUI dependency or upstream source was changed.
Its first two pixel checks passed in `/tmp/spaceport-9s44-metal-image-raster-public-api.log`.

Independent review found two insufficient assertions.
The original black sample lay outside the image, and equal boundary colors could both be double-blended.
Those checks now use the clipped pane exterior and the original single-pass reference.

The earlier cropped-mapping implementation was restored temporarily for the regression baseline.
The actual Metal result was `[170, 0, 77, 255]` where the original pixel was `[247, 0, 0, 255]`.
The transparent-boundary control still passed.
The valid runtime red is `/tmp/spaceport-9s44-metal-cropped-mapping-red.log`.
The intended source-grid mapping was restored immediately and matched its saved SHA256 exactly.
Both strengthened tests then passed in `/tmp/spaceport-9s44-metal-image-raster-final.log`.
The temporary baseline's unused-field warning disappeared after restoration.
The pre-existing linker compact-unwind and external `block` notices remain unchanged.

Independent review confirmed both assertion findings resolved and found the context cleanup sound.
Canonical host lint passes both feature configurations after the new tests.
The log is `/tmp/spaceport-9s44-metal-image-host-lint.log`.
Formatting, file-size, sorted-region, and diff checks pass.
The two Metal cases are macOS-only; the preceding Windows GNU checks cover the unchanged production path.

This establishes offscreen Metal image raster correctness for the tested fragments.
It does not establish AppKit input, actual foreground presentation, native glyph behavior, or interactive timing.
The original native keyboard readiness question remains pending, and its fixture has not launched.

## Remaining native route, November 19

The existing Windows `accessibility_windows` harness can qualify a real image-bearing pane without synthetic connection state.
Use `Running::prepared`, its explicit private daemon, and the normal Socket GUI target.
Keep exact-child cleanup and the sandbox's isolated home, configuration, temporary directory, and socket.

Pinned Herdr `2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01` has a real inline Kitty RGBA producer test.
Its [graphics test](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/src/server/headless/tests/pane_graphics.rs#L385) checks actual client-shell image assets.
Its [pane CLI](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/src/cli/pane.rs#L1039) runs a command in one exact pane.
An authored finite PowerShell script can use that path inside the fixture's existing private shell.
The Windows PowerShell/ConPTY producer path still needs execution; the upstream test alone does not qualify it.

The case must discover the private pane before submitting the producer.
Require coherent presented metadata, paired authored image assets, actual decoding, and actual atlas upload before capture.
Use contrasting source rows, below-text and above-text images, transparent overlap, and known scrollback lines.
Capture the stationary frame, an unchanged redraw, one fractional image-bearing slide, and the settled frame.
Check fixed pane clipping, image/text translation, source mapping, and layer order against those frames.
Require unchanged region identities and zero cell paint callbacks during the stationary redraw.

The existing test image probe paints marker quads and must stay out of this native scene.
A scoped nonvisual integration counter must observe actual cell paint callbacks instead.
Arm the in-app observer before native wheel input; the existing 48-millisecond spread can evade external polling.
Keep the real clock, smooth-scroll policy, and native input behavior unchanged.
Guard input with the exact PID, HWND, foreground, focused terminal, and released-modifier checks.
Any wheel helper must bound its own pointer movement and restore it without overriding later user movement.
Encode only bounded captures from the owned window on the background executor.

This route has been inspected, but no new native image case has run.
It qualifies the remaining image-rendering gate when executed and inspected.
It does not replace the separate foreground release profile owned by `gg9v`.

## Real-pane native candidate

The new `accessibility_windows::images` case keeps the existing normal Socket GUI and private daemon.
It discovers the actual pane, then submits one authored script through Herdr's existing `pane run` command.
The script writes four tiny inline RGBA images through the pane's existing PowerShell process.
It writes bounded scrollback and a unique fixture token before waiting on its private completion marker.
The wait is bounded, and no additional producer process starts.
The observer requires that token, exact source pixels, paired asset serials, and a coherent presented frame.
Queue acceptance alone never satisfies image delivery.

The nonvisual probe records actual region canvas callbacks and actual image paint/atlas observations.
It belongs to one window and bounds each frame's image records.
It never paints diagnostic quads or retains image textures.
An actual renderer regression fails first because the initial cells produce no observed callbacks.
After wiring the real callbacks, it passes actual decoding, upload, unchanged region identities, zero replay callbacks, and probe release.
Logs are `/tmp/spaceport-9s44-native-image-probe-{red,green}.log`.

The native observer records the completed frame through GPUI's next-frame callback.
Pinned GPUI runs that callback before drawing the following frame.
The readback therefore uses the scene associated with the recorded picture and slide.
The case captures stationary, unchanged replay, fractional, and settled frames without changing the clock or scroll policy.
Each screenshot permits at most two million pixels and a 128 KiB PNG.
Encoding runs on the background executor.
Failures retain a bounded owned-window capture before reporting the original error.

Native input requires one exact PID, HWND, focused Terminal document, and released buttons/modifiers.
The helper sends one wheel delta and never retries it.
It uses Windows' [physical cursor APIs](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setphysicalcursorpos).
Finally, cancellation, and an independent watchdog restore only the helper's unchanged pointer claim.
Later user movement or a different foreground window prevents restoration.
An intentional pointer-failure case requires restoration before the actual image gesture starts.
The parent preserves the original failure and waits for helper cleanup before dropping its owned processes.

Review finds that absence outside the pane cannot prove clipping when the edge image is also missing.
Two pixel-validator regressions fail first on missing interior cyan and an unavailable outside sample.
The validator now requires both sides and rejects an unavailable sample.
Its fractional case also requires a historical image mapping beyond the fixed pane before masking.
The stationary wire placement cannot supply that oversized extent: pinned Herdr [clips it before export](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/src/kitty_graphics.rs#L495).
The actual masked fractional paint supplies the stronger client-side evidence.
The capture also checks contrasting source rows, transparent overlap, and below/above-text glyph visibility.
Independent review confirms the original clipping finding is resolved without changing the renderer or limits.

The focused consumer run passes 97 tests, with two existing Metal opt-ins ignored.
The raster false-positive regressions pass in that run.
Both offscreen Metal pixel cases also pass explicitly after the new observation hooks, with none ignored.
Their log is `/tmp/spaceport-9s44-native-image-metal.log`.
Logs are `/tmp/spaceport-9s44-native-image-clipping-red.log` and `/tmp/spaceport-9s44-native-image-consumers.log`.
`just lint lint-windows` passes all four workspace/all-target feature configurations.
The final Windows harness cleanup follow-up also passes targeted all-feature and no-default-feature cross-lint.
Canonical formatting, size, sorting, operations, i18n, commit-subject, and diff checks pass.
The existing external `block` future-compatibility notice remains unchanged.

The PowerShell/ConPTY producer and embedded C# still require actual Windows execution.
Cross-lint checks the Rust harness but does not compile or execute those embedded scripts.
No native image qualification or `gfde` closure is claimed by this candidate.

## First Windows execution on `97864f275`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44635700648/job/135953846600) passes 23 UIA cases and fails the new image case.
Its pointer-failure helper reaches the private terminal before failing in its `finally` block.
The C# cleanup method returns Boolean, but PowerShell's dynamic call site requires Object.
The [official binder source](https://github.com/PowerShell/PowerShell/blob/master/src/System.Management.Automation/engine/runtime/Binding/Binders.cs) special-cases methods named `End` or `Process` before boxing their results.
That name-based branch incorrectly includes this unrelated `End` method.
It prevents cleanup invocation and masks the expected intentional failure.
The run therefore proves neither pointer restoration nor native image qualification.
It never reaches the image producer, so this failure says nothing about ConPTY graphics support.

The test-only method and caller now use `Finish`.
Restore logic, ownership, watchdog, deadlines, and restoration receipts remain unchanged.
The existing negative native case requires both the original intentional failure and `restored=True`.
Independent review confirms the root cause and the two-line correction.
Its actual Windows rerun remains pending; local Rust cross-lint cannot prove this fix.

## Native progress and silent producer command on `da9023c8c`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44641489126/job/135973846144) passes eight GUI cases and pipe-owner qualification.
It passes 23 UIA cases, then reports the image case's JSON parse failure.
All eight Linux/Windows workspace jobs also pass; the ARM native job passes its configured cases.
This run reaches UIA after the readiness correction; it does not establish universal startup reliability.
The [Windows x64 all-feature job](https://github.com/2389-research/spaceport/actions/runs/44641489126/job/135973845658) executes both real-pipe readiness regressions successfully.
That proves the corrected first-connection and missing-pipe behavior on Windows; the old-probe unit baseline remains unrun.

The image case advances past both intentional-pointer-failure assertions before the later panic.
Those assertions require the original error and `restored=True`.
That control flow establishes the `Finish` failure-cleanup slice; the earlier receipt was not printed separately.
The fixture now emits an explicit bounded receipt immediately after those assertions.

Pinned Herdr's [`pane_run`](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/src/cli/pane.rs#L1039-L1051) uses `send_ok_request`.
Its [success path](https://github.com/herdrdev/herdr/blob/2ecf19a92fc3a7fff75b6ce3655ef04f4625ec01/src/cli.rs#L755) returns zero without printing JSON.
The fixture incorrectly treats that silent success as a query envelope.
The producer may already have started before the parser panic; no retry is justified.

The producer now uses the shared status-only CLI path, once.
Queries still require JSON and retain operation/byte-count diagnostics for malformed output.
The same child deadline, no-console flag, capture limit, and cleanup remain intact.
Command success is not delivery; exact authored assets and all four captures remain mandatory.
Independent review finds no weaker delivery or cleanup condition.
Both Windows harness cross-lints pass: all features and integration-only without default features.
Logs: `/tmp/spaceport-9s44-image-silent-command-lint-{all,minimal}.log`.
Actual native scene, fractional motion, clipping, mapping, and layer qualification remain pending.
