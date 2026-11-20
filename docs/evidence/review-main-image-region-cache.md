<!-- ABOUTME: Plans stationary image painting beside cached terminal cell layers. -->
<!-- ABOUTME: Preserves presented asset ownership, paint order, and existing resource bounds. -->

# Main-grid image region cache implementation plan

Use the existing TDD and subagent-driven workflow for this bounded `gfde` slice.

**Goal:** Reuse unchanged terminal cell regions while drawing main-grid images every frame.

**Architecture:** Interleave uncached image passes with the existing cached cell layers.
Prepare one main-target batch and share it between its below-text and above-text passes.
Keep each cached RegionView free of image sprites.

**Tech stack:** Pinned Rust, GPUI, existing terminal painter, and private synthetic image fixtures.

## Authority and limits

Doctor Biz authorized the existing performance fixes and routine internal design decisions.
Repository authority supersedes the skills' routine approval and commit gates.
No commit, push, PR, or release is authorized.
Native visual and timing checks remain pending.
Do not edit Herdr or change protocol contracts.
Preserve existing texture, decoded-byte, entry, placement, and decode-worker bounds.
This initial stationary-image slice kept the smooth-scroll image guard.
The later [image-pane scrolling slice](review-image-pane-scroll.md) supersedes that guard.
Historical frames retain metadata only; asset changes cancel motion instead of adding encoded-image leases.

## Chosen route

Keeping the main-image guard preserves the reported cache cost.
Caching image sprites would need atlas leases that GPUI does not expose here.
Choose uncached image passes around cached cells.

Preserve this order:

1. Cached main backgrounds.
2. Uncached main images below text.
3. Cached main text.
4. Cached main decorations and cursor.
5. Uncached main images above text.
6. Existing link, popup, composition, and input canvas.

Use the presented frame and its paired SurfaceImages from the same Picture.
Never independently read newer live image state for an older presented frame.
Keep equal-z placement order stable.
Do not add image passes when the main target has no placements.
Prepare the main target once, even though two passes consume its batch.
Preserve the existing popup target, ordering, and preparation path.

Each target currently advances the image eviction epoch.
The proposed split must not introduce another main-target epoch.
The existing main-plus-popup atlas-pressure question is unverified.
Do not change its policy without a real regression.
Record any reproduced ownership failure before fixing it.

## Files and ownership

Popup owns this implementation; Root reviews and integrates it.
Root separately owns chat parsing.
Coordinate every Cargo build through Root.

- `window/regions.rs`: expose or construct globally ordered cell-layer groups.
- `window/render.rs`: compose uncached image passes with those groups.
- `terminal_painter/images.rs`: prepare one opaque, frame-local batch.
- `terminal_painter.rs`: retain narrow wiring and stay below 1,000 lines.
- A focused image-paint module may hold extracted existing draw helpers.
- `window/regions/tests/images.rs`: exercise real window reuse and image rendering.
- Topic tests under `terminal_painter/` verify actual image paint and atlas behavior.

## Execution and acceptance

- [x] Add a real main-image window regression before changing the guard.
- [x] Prove current code discards otherwise unchanged region IDs.
- [x] Require actual decode and image paint, beyond entity-ID assertions.
- [x] Capture actual background, image, text, decoration, and popup paint order.
- [x] Check both main source kinds, crossing-pane placements, and equal-z ordering.
- [x] Check placement changes, removal, re-addition, and presented asset pairing.
- [x] Check one main preparation and unchanged bounded eviction behavior.
- [x] Exercise atlas removal and subsequent paint with private assets.
- [x] Record unsupported headless renderer checks precisely.
- [x] Implement only after the valid runtime red and source review.
- [x] Run region, painter, popup-image, and wheel/slide consumers.
- [x] Run quick validators after source settles.
- [x] Prove unchanged images replay cached cells without invoking cell callbacks.
- [x] Run host/Windows all-feature lint after source settles.
- [x] Obtain Root's independent ownership and ordering review.
- [ ] Keep native timing and real glyph qualification pending.

Use the existing immutable-artifact builder under `/tmp/spaceport-qnz9/cargo.lock`.
Run topic filters from the pinned executable outside the Cargo lock.
The starting qualified executable is `/tmp/spaceport-9s44-gpui.HsLaYo/spaceport-tests`.
It proves the current popup-only exception and retains the main-image guard.

## Current state

Root ran the tests-only checkpoint from `/tmp/spaceport-9s44-gpui.z1NSZc/spaceport-tests`.
Both source-kind tests failed at region reuse after confirming real decode and atlas upload.
The other four cases passed.
The log is `/tmp/spaceport-9s44-main-image-baseline.log`.

The six `main_grid` cases exercise private, valid raw image assets through actual asynchronous decoding.
Both source-kind regressions require uploaded textures before asserting region reuse.
The implementation passed its first runtime qualification in the immutable debug executable
`/tmp/spaceport-9s44-gpui.ZtQ5tx/spaceport-tests`.
Each command used `--test-threads=1` outside Cargo.
These scopes overlap and must not be added together:

| Filter | Passed | Failed | Ignored | Log under `/tmp/spaceport-qnz9/` |
| --- | ---: | ---: | ---: | --- |
| `window::regions::tests::images` | 7 | 0 | 0 | `gfde-main-image-images-green.log` |
| `terminal_painter` | 50 | 0 | 0 | `gfde-main-image-painter-green.log` |
| `window::regions` | 13 | 0 | 0 | `gfde-main-image-regions-green.log` |
| `smooth_scroll` | 15 | 0 | 0 | `gfde-main-image-slides-green.log` |
| `presentation::tests::` | 6 | 0 | 0 | `gfde-main-image-presentation-exact-green.log` |
| `endpoint::lifecycle_tests::input_settings` | 4 | 0 | 0 | `gfde-main-image-input-settings-green.log` |

The broader `presentation` filter selected an unrelated error-message test.
It reported nine passes and one failure in `client_schema_presentation_remains_redacted`.
The fixture expected `invalid endpoint catalog schema`; current display returned the localized saved-file diagnostic.
Root corrected the stale test expectation and strengthened its source-redaction checks.
The correction changes no production error path.
Root's exact `error::tests::` filter passed both cases in EcPUAT.
Its log is `/tmp/spaceport-9s44-error-presentation-final-green.log`.
The broad log is `gfde-main-image-presentation-green.log`.

Only test instrumentation changed after this executable was built.
The probe now counts actual main cell-layer callbacks.
Both source-kind cases require zero callbacks on a second unchanged draw.
They also require the cached marker sequence and real image upload to remain present.
The scene-order case checks zero callbacks while retaining its strict layer ordering.
Root built `/tmp/spaceport-9s44-gpui.EcPUAT/spaceport-tests` for these strengthened assertions.
All seven image cases passed with zero failures and zero ignores.
The final log is `/tmp/spaceport-9s44-main-image-final-green.log`.
The result proves zero main cell callbacks while cached marker order and real image paint remain present.
No production image code changed after the earlier consumer qualification.

Quick formatting, size, sorted, i18n, and diff checks passed after the source freeze.
The i18n check validated 2,174 messages in 114 files.
Final host and Windows GNU all-target, all-feature Clippy pass with warnings denied.
Logs: `/tmp/spaceport-9s44-performance-{host,windows}-clippy-final.log`.
Only the existing external `block` future-incompatibility notice remains on the host.
Cross-lint does not establish native Windows execution.
Native qualification remains pending.

Root independently inspected the production source and found no material issue.
The review covered layer order, Picture pairing, uncached image ownership, batch consumption, bounds, and image-free rendering.
It also confirmed that the smooth-scroll guard remains unchanged.

`terminal_regions` now receives the presented Picture and its paired assets.
It inserts uncached image canvases after backgrounds and after decorations.
An opaque ImageBatch prepares once and splits its sorted placements by z.
The upper canvas consumes the batch before the existing popup preparation.
No texture enters a cached RegionView.
Image-free frames add no canvases and retain their existing idle release.
The smooth-scroll guard and image eviction policy remain unchanged.

The image helper extraction keeps `terminal_painter.rs` below 1,000 lines.
A new test observation records texture reference counts at popup preparation.
It requires main textures to have only cache and test-observation owners then.
This checks that the shared batch releases its texture leases after the upper pass.

The runtime cases preserve these contracts:

- Main backgrounds, below-text images, text, decorations, above-text images, then popup paint in order.
- Equal-z images retain placement order, including placements crossing both panes and their border.
- Main preparation advances one epoch, followed by the existing single popup preparation.
- Removing a real atlas entry causes an unchanged image to upload during its next paint.
- A retained Picture uses its paired assets after live image state changes.
- Replacing that Picture releases its old encoded asset set.
- Moving, removing, and re-adding placements preserves their actual geometry.

The scoped probe only exists under `cfg(test)`.
It observes real image preparation, texture uploads, and paint callbacks.
It inserts marker quads within the actual cell layers and image paint contexts.
`Window::painted_quads` then exposes marker order, including cached scene replay.
The probe neither replaces the renderer nor supplies image data.
It preserves the decode limit and uses no application mode.

The ordinary TestAppContext has no configured HeadlessRenderer.
Raster screenshots and native visual checks remain pending.

## Image-free pane scrolling follow-up

The stationary image cache slice is complete; `gfde` still includes a global smooth-scroll veto.
One placement currently blocks fractional input and render-time slides in every pane.
Root owns a narrow follow-up while native foreground qualification remains blocked.

Keep images stationary and keep image-overlap panes on whole rows.
Remove only the window's global placement veto.
Reject overlapping main-image cell rectangles in `SmoothScroll::wheel`.
Cancel motion in `observe` when either the old or new frame overlaps the scrolling rectangle.
Use conservative half-open geometry and the existing main-target source filter.
Preserve popup, setting-off, scrollbar-drag, mouse-reporting, and alternate-screen policies.
Retain no additional image bytes or texture leases.

Regression scenarios cover fractional endpoint input beside images, cross-pane placement overlap, and image entry/removal.
An actual decoded-image draw must retain the neighboring pane's fractional offset without moving the image.
Pure scroll tests must decline image-bearing frames and release predecessors before storing unsafe history.
Root will run the regressions before changing production behavior.
Full image-pane sliding still needs bounded historical assets and matching image clipping; this slice does not enable it.

The valid baseline is `/tmp/spaceport-9s44-gpui.CCvaDT/spaceport-tests`.
Two endpoint cases and two pure scroll cases failed for the expected guard and history behavior.
Three decoded-image cases failed, while four other image cases passed.
One failure proves render discarded a neighboring pane's fractional offset after actual image upload.
Two strengthen existing cache tests with actual image-pane wheel rejection instead of the global flag.
Logs are `/tmp/spaceport-9s44-image-free-pane-{input,model,paint}-red.log`.
The earlier build failure was an unnecessary import qualification, not runtime regression evidence.

The green executable is `/tmp/spaceport-9s44-gpui.pfTfD5/spaceport-tests`.
Both endpoint cases, all sixteen pure scroll cases, and all eight image cases pass.
The actual decoded neighbor keeps its uploaded pixels and exact bounds while the other pane rests at -0.75 rows.
It still prepares main images once per draw.
Existing wheel cases pass two tests, setting-off one, input settings four, scrollbar drag one, regions fourteen, and presentation six.
These scopes overlap and must not be added together.
Logs use `/tmp/spaceport-9s44-image-free-pane-*-green.log` and `*-consumers.log`.

The existing whole-row wheel fixture now places its image inside the pane instead of on window chrome.
That preserves its intended overlap case under the narrower policy.
No image cache, decode, upload, eviction, or encoded-byte ownership code changed.
`ImageStore::receive` removes encoded assets before publishing surface metadata to the UI.
Thus retained image-free-pane history adds no image asset leases.
Independent review and latest lint results remain pending this checkpoint.

Review found a cross-pane cancellation redraw gap.
A sub-row wheel over the image pane clears the other pane's settled slide without submitting a whole line.
The existing input handler then requests no terminal redraw because smooth input returned `None`.
Input and IME geometry can reset before the old shifted picture repaints.
Clipboard owns only a new cancellation regression file; Root owns its registration and production correction.
The regression must require a redraw notification before any explicit paint can mask the missing request.
Root will run that red before changing the input handler.

The cancellation regression failed at zero redraw notifications, where one was required.
Its old fractional offset, cleared input/IME geometry, and lack of an incidental render all passed first.
The valid red is `/tmp/spaceport-9s44-image-cancel-red.log` from `q1HM73`.
The input handler now requests a surface redraw when an existing slide clears, including whole-row fallback with no input.
Independent production review approved the correction and found no remaining material issue.

The first green run exposed an invalid test assertion forbidding the intended automatic render.
That run is not passing evidence.
The test now records the render count inside the notification callback, before the automatic render.
It requires one notification, unchanged count at notification, and exactly one subsequent automatic render.
No explicit draw, refresh, or notify can rescue the cancellation path.
The framed sentinel also proves the sub-row fallback sent no whole-row input.

The current integration-feature app and unit executable compile successfully.
All three endpoint image-scroll cases, sixteen scroll cases, and eight decoded-image cases pass.
Logs are `/tmp/spaceport-9s44-image-final-{input,scroll,paint}-green.log`.
The preceding source also passed 26 wheel consumers and eight image consumers after the cancellation handler changed.
Only the test assertion changed after those consumer runs.
Latest canonical host and Windows GNU lint pass both all-features and no-default-features workspace/all-target configurations.
Logs are `/tmp/spaceport-9s44-image-final-{host,windows}-lint.log`.
Formatting, file size, sorting, and diff checks pass.
Actual native image scrolling remains unqualified while foreground presentation is unavailable.
`gfde` remains open for its remaining work and qualification.

## Later image-bearing scrolling

The image-bearing pane now slides through the same source bands as its terminal cells.
Stationary fragments outside that pane remain fixed.
History uses the picture's existing asset collection and cancels before image identities change.
No extra encoded assets or larger resource limits are retained.
The painter preserves texture mapping with snapped destination content masks.
Both thin historical directions, replacement under an unchanged surface pointer, and crossed transparent images pass.
The crossed-image test covers four display scales and equal-z order.
All twenty-one integration-feature region tests pass.
See [the follow-up evidence](review-image-pane-scroll.md) for exact failures and results.
Native raster output and timing remain unqualified.
