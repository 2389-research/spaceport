# Splitting `terminal_painter` into one file per layer

Implements [`notes-lower-shared-types-design.md`](notes-lower-shared-types-design.md)
§3 Shape A and issue #127. The module was 983 production lines in one file
against the repo's 1,000-line ceiling, with eight sub-responsibilities merged
into a single `impl TerminalPainter`. This is a pure file move: no behaviour
changed, no function was rewritten, and the only signature-level edits are
visibility widenings from private to `pub(super)` for the moved items a
neighbouring file calls.

## What moved

| New file | Moves | Lines now (design's estimate) |
| --- | --- | --- |
| `terminal_painter.rs` | `mod`/`use` wiring, `TerminalPainter`, `Default` | 82 (under 150) |
| `terminal_painter/highlight.rs` | `Tint`, `Highlight`, `SELECTION_ALPHA`, `MATCH_ALPHA`, `CURRENT_MATCH_ALPHA`, `Tint::color` | 46 (42) |
| `terminal_painter/diagnostics.rs` | `PaintTiming`, `PaintDiagnostics`, `REPORT_INTERVAL`, `SLOW_PAINT` | 62 (48) |
| `terminal_painter/backgrounds.rs` | `background_spans` with its wide-symbol and color closures | 58 (49) |
| `terminal_painter/links.rs` | `link_bounds`, `paint_link` | 87 (71) |
| `terminal_painter/composition.rs` | `composition_origin`, `byte_index`, `paint_composition`, `composition_bounds` | 97 (80) |
| `terminal_painter/surface.rs` | `set_appearance`, `reset_cache`, `verify_native_cache`, `shape`, `metrics`, `configure`, `cell_width`, `release_idle_images` | 120 (107) |
| `terminal_painter/text.rs` | `paint_glyphs`, `paint_text` | 147 (134) |
| `terminal_painter/frame.rs` | `paint_frame`, `paint_images`, `SCROLLBAR_INSET`, `SCROLLBAR_ALPHA` | 397 (369) |
| `terminal_painter/tests.rs` | the module list plus the harness's own imports | 43 (27) |

Each file carries a few lines more than the design estimated because it opens
with a `//!` doc comment and its own import block; the moved bodies are the
same lines in the same order.

`paint_frame` keeps its `#[allow(clippy::too_many_arguments)]`. It takes 12
arguments and splitting it needs an argument struct, which would change how the
`part` and `images` options drive the three paint paths; that is out of scope
here.

## Where the public surface stayed

`crate::terminal_painter::` still names the same 15 items other modules read:
`Highlight`, `Tint`, `TerminalPainter`, `Layer`, `Part`, `Span`, `cell_ranges`,
`clip`, `covers`, `decoration_spans`, `CellSeparator`, `Ground`, `ImageTarget`,
`PlacedImages`, `fit_bgra`, `texture`. The main file's `pub(crate) use` lines are
that surface, unchanged.

Visibility widened to `pub(super)` only where a sibling file calls a moved item:

| Item | From | Reason |
| --- | --- | --- |
| `Tint::color` | private | `frame.rs` tints each highlight |
| `PaintTiming` and its four fields, `PaintDiagnostics`, `new`, `record`, `take_errors` | private | `frame.rs` records a paint and formats the report |
| `REPORT_INTERVAL`, `SLOW_PAINT` | private | read by `PaintDiagnostics` and by its tests |
| `background_spans` | private | `frame.rs` paints the runs |
| `link_bounds` | private | `paint_link` in the same file, and its test |
| `composition_origin`, `byte_index` | private | `paint_composition`/`composition_bounds` and their tests |
| `shape` | private | `text.rs` and `composition.rs` shape through the cache owner |
| `metrics` | private | `frame.rs` and `links.rs` measure cells |
| `configure` | private | `frame.rs` re-configures the font before painting |
| `paint_glyphs` | private | `composition.rs` paints its shaped line with it |
| `paint_text` | private | `frame.rs` paints the text layer |

`decoration_spans` stays re-exported from the main file rather than named
through `terminal_painter::decorations`, because that module is private to
`terminal_painter` and `window/links/tests/cases.rs` reads the spans too.

## The `sheprd-cells` boundary

Five of the moved pieces name no `gpui` type, and each now says so in its
header so the boundary drawn today stays visible: `area.rs`,
`decorations.rs`, `diagnostics.rs`, `backgrounds.rs`, and `links.rs`'s
`link_bounds`. That is 291 lines of cell-rect and color algebra a future
`sheprd-cells` crate can take unchanged; §2.3 of the design keeps the crate
itself out of scope until a non-GPUI consumer appears.

## Gates

| Check | Result |
| --- | --- |
| `cargo check -p sheprd-gpui --all-targets` (before and after) | passed, no warnings from the crate |
| `cargo check -p sheprd-gpui --all-targets --features integration-test` | passed, so the `reset_cache`/`verify_native_cache`/`Counts` paths still compile |
| `cargo test -p sheprd-gpui --bin spaceport -- terminal_painter::` | 47 passed, 0 failed — the same 47 the pre-split tree ran |
| `cargo test -p sheprd-gpui --bin spaceport` | 4062 passed, 0 failed, 10 ignored |
| `cargo test -p sheprd-gpui --bin spaceport -- find::` | 25 passed, 0 failed |
| `cargo test -p sheprd-gpui --bin spaceport -- copy_mode::` | 24 passed, 0 failed |
| `cargo fmt --all -- --check` | passed |
| `cargo clippy -p sheprd-gpui --all-targets` | passed, no warning from the crate |
| `just check-file-size` | passed; the largest file in the module is `frame.rs` at 397 lines |
| `just check-sorted` | passed |
| `just check-commits` | passed |

`Cargo.lock` did not change, so no `cargo metadata` run was needed.

## Deferred

- The `windows` and `checks` CI legs still lint every target and feature; the
  local clippy pass above is the default-feature set.
- No native QA: no pixel, layout, or input path changed, so a screenshot would
  show the same window.
- Extracting `sheprd-cells` itself, which the design rejects until a consumer
  exists outside the app.
