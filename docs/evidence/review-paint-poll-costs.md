# Painting and polling costs

Finding: `gfde`. Verified partial work through 2026-11-19.
The finding remains open.

## Theme changes

Shaped terminal glyphs contain font geometry and neutral colour.
The painter applies the current terminal colours separately.
Colour-only theme changes now retain glyphs and measured cell width.
Font, size, and cell-height changes retain their existing invalidation paths.

The new actual painter regression failed when a theme change cleared its cached symbol.
Red: `/tmp/spaceport-9s44-theme-cache-red.log`.
The same regression now passes and observes both old and new background quads.
Green: `/tmp/spaceport-9s44-theme-cache-green.log`.
Existing cache checks still cover font family, weight, cascade, style, and storage bounds.
All 30 default-feature painter tests pass.
Log: `/tmp/spaceport-9s44-terminal-painter-green.log`.
These are headless behavior checks, not native timing measurements.
Independent review approved the neutral shaping and current-colour paint paths.
Whole GPUI host and Windows GNU all-target, all-feature Clippy now pass with warnings denied.
Logs: `/tmp/spaceport-qnz9/2njz-host-clippy-source-stable.log` and `2njz-windows-clippy-source-stable.log`.

## Cold glyph replacement

The cache now replaces glyphs that neither the current nor previous paint used.
Recent glyphs stay protected even when a screen has more symbols than the cache can hold.
Each paint inspects at most 4,096 replacement candidates across all misses.
After that search budget ends, misses remain uncached for that paint.
The cache reuses slots and clears their old ASCII or Unicode lookup first.
Unicode lookup and slot metadata share one `Rc<str>` key.
Font invalidation resets the complete cache and its replacement state.

The actual painter regression first filled the cache, then painted a different symbol twice.
The original code kept that current symbol uncached forever.
Red: `/tmp/spaceport-9s44-glyph-eviction-red.log`.
The oversized repeated-screen case already passed before the fix.
It still preserves the cached working set after the fix.

The final immutable executable is `/tmp/spaceport-9s44-gpui.gVX0UV/spaceport-tests`.

```sh
/tmp/spaceport-9s44-gpui.gVX0UV/spaceport-tests terminal_painter::tests --test-threads=1
```

All 32 default painter tests pass, with no failure or ignore.
They include both new cases and existing font, colour, image, and decoration consumers.
Green: `/tmp/spaceport-9s44-glyph-eviction-green.log`.
Independent review approved bounded searching, slot/map consistency, working-set protection, and synchronous shape ownership.
The cache source remains unchanged after that review.
Aggregate host and Windows GNU lint now pass; native timing remains pending.
These tests prove cache behavior; they do not measure a native speed improvement.

## Image cache and history

Main-grid image placements now preserve region reuse.
Cached scenes can outlive a texture's idle eviction.
Historical scroll frames retain placement metadata without retaining the corresponding encoded assets.
The later image-pane scrolling slice preserves those bounds without retaining additional encoded assets.
Popup images paint separately after the cached regions in `window/render.rs`.
A new actual-window regression first failed because popup-only placements discarded every region.
Red: `/tmp/spaceport-9s44-popup-image-region-red.log`.
The guard now reuses the painter's main-target predicate.
Popup-only placements can keep unchanged terminal regions.
The same test checks that both main-grid source kinds still require whole-frame painting.
The complete region scope now passes seven tests, with no failure or ignore.
Green: `/tmp/spaceport-9s44-popup-image-region-green.log`.
Artifact: `/tmp/spaceport-9s44-gpui.xD8o8D/spaceport-tests`.
Independent review approved target matching, paint order, and unchanged texture ownership after reading the green log.
At that earlier popup-only checkpoint, main-grid image fallback and smooth scrolling remained unchanged.
The later image-cache and scrolling sections below record their implemented replacements.

Port scanning passes 56 checks, including shared consumers, exact-host bounds, cleanup, reconnect, and owned process attribution.
Eight ignored subprocess entry points run through their nonignored private parents.
The five control-socket port consumers also pass.
Root approved the scanner's independent lifecycle and data-integrity review.
Linux native qualification and release timing remain pending.
See [listening-port evidence](review-listening-port-scan-costs.md).
Sidebar qualification passes 399 sidebar, 124 Palette, and 60 state checks.
The index borrows records from the exact source snapshot and keeps only weak source ownership.
First-match lookup, plugin order, source release, and same-revision allocation changes have coverage.
Root approved the independent index and state review after reading the actual consumer results.
The final count is 896 actual row visits for 640 source rows, against the original 49,408 visits.
See [sidebar evidence](review-sidebar-lookup-costs.md).

The qualified aggregate logs are `/tmp/spaceport-9s44-host-clippy-qualified.log` and `spaceport-9s44-windows-clippy-qualified.log` in `/tmp`.
Both checks cover every target and feature with warnings denied.
The host emits the existing external `block` future-incompatibility notice.

Streaming chat already skips unchanged entry revisions in `chat/prepared.rs::Prepared::update`.
Changed agent entries still run summary preparation and then Markdown preparation when no summary matches.
The message byte and line bounds remain enforced.
Incremental parsing has not been implemented or qualified.
Any change must preserve backward references, code fences, math, embedded content, and selection coordinates.
The read-only parser investigation found a conservative candidate.
Fragments without `$` or backslash cannot require math delimiter rewriting.
They can keep the final CommonMark parser while avoiding its redundant protection parse and two text copies.
Test-only work instrumentation proved 13 unnecessary protection-parser bytes before the fix.
Root recorded a release baseline before adding the no-marker fast path.
The paired synthetic workload improved ordinary preparation by 17.5 percent.
The math control stayed effectively unchanged.
All 545 debug chat consumers pass, with the ignored benchmark run separately in release.
Independent review approved semantic identity, source coordinates, limits, and benchmark validity.
See [streaming parser evidence](review-streaming-parse-costs.md).

Stationary main images now paint uncached between cached terminal cell layers.
One batch uses the exact presented Picture's paired frame and assets.
The upper pass consumes that batch before popup preparation.
Actual decode, atlas upload, paint order, and zero unchanged cell callbacks have runtime coverage.
Root independently approved ownership, geometry, ordering, and unchanged resource bounds.
See [image evidence](review-main-image-region-cache.md) for the exact consumer runs.
The existing per-target eviction epochs remain unchanged.
No atlas-pressure policy failure was reproduced or claimed.
Fractional image fragments now use the same historical bands as terminal cells.
History resolves placements through the presented picture's existing asset set.
Replacing or losing a needed asset cancels the gesture before the asset collection changes.
The image painter uses snapped destination masks instead of cropping texture coordinates.
Four-scale crossed-image tests and both historical directions pass.
See [image-pane scrolling](review-image-pane-scroll.md) for regression and review evidence.
The offscreen Metal pixel checks pass.
Foreground image mapping, clipping, scrolling, and native timing qualification remain pending.

## Original-scope audit, November 19

The independent audit accepts the theme cache, bounded glyph replacement, shared port scans, and sidebar index fixes.
Image caching and scrolling have actual decode, cache-callback, geometry, and offscreen Metal evidence.
They still need the authored foreground visual qualification described above.

Streaming preparation remains an implementation gap.
`Prepared::update` still scans each changed complete Agent prefix for summaries, then rebuilds its message fragments.
The rich parser still submits the complete changed fragment to its final CommonMark parser.
The 17.5 percent synthetic improvement removes redundant math protection work only.
It does not prove that the final full-prefix work is resolved.
The current test-only work meter observes that final parser directly through real transcript appends.
Keep `gfde` open until the remaining work is fixed and qualified.

The separate redraw finding, `gg9v`, requires a release-mode profile first.
Doctor Biz requested that native keyboard and performance checks remain pending.
The synthetic parser measurement does not qualify native frame timing or redraw costs.
