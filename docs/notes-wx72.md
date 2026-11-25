# wx72, part 1: one wheel rule for the desktop and the phone

Crew task wx72, part 1 only. Branch `feat/wx72-wheel-rule`, cut from
`f778a8626c`. The phone (`mobile/rust`) carried its own copy of the rule the
desktop applies when the wheel lands on a pane or a popup. That rule needs no
GPUI values, so it moved whole into `spaceport-backend` and both clients now
call it. Behaviour is unchanged on both sides: the same events, with the same
kinds, positions, geometry, modifier byte and per-event line counts.

## 1. What the two copies held, and what each actually needed from GPUI

The desktop's `wheel_target` + `WheelTarget::wheel_events`
(`crates/spaceport-gpui/src/terminal.rs`, 182 lines at the base revision)
needed exactly one GPUI value of its own: `Bounds<Pixels>`, the rectangle a
mouse gesture's later moves are clamped into (`window/mouse.rs:299,316`). Two
more GPUI types appear around it but not in the rule: `Point<Pixels>` from
`popup_origin` (an `f32` once taken as `.to_f64() as f32`) and
`CellMetrics`, which is a pair of `f32` cell sizes with cell-to-pixel helpers.

The phone's copy (`mobile/rust/src/projection/wheel.rs`, 85 lines) needed no
GPUI at all: it reads the daemon's own numbers out of
`PaneSurfaceFrame`/`PaneSurfacePane`/`ClientShellPopupSurface` and emits
`ClientPaneInputEvent`. It was the same rule with the pointer replaced by the
cell a finger touched.

So the rule itself — which flags make a program read the wheel stepwise, when a
report carries a pixel extent, whether the position travels as pixels or as a
cell, and how a count becomes one event or one per step — is GPUI-free.
`spaceport-backend` already depends on `spaceport-protocol` only (plus `serde`
and `thiserror`), re-exports `ClientPaneInputEvent` and friends, and carries no
GPUI, runtime or transport, which is why it is the home.

## 2. What moved

| Was | Is | Lines |
| --- | --- | --- |
| `terminal.rs`'s `stepwise` derivation, `geometry` `then_some`, Pixels-vs-Cell choice, and the per-step fan-out | `crates/spaceport-backend/src/wheel.rs`'s `WheelGrid` + `WheelHit` | 191 (new) |
| `terminal.rs`'s `WheelTarget { position, geometry, stepwise }` | `WheelTarget { hit: WheelHit }`, plus `modifier_bits` for the byte a report carries | 692 → 625 |
| `mobile/rust/src/projection/wheel.rs`'s flag tuple, geometry, position and fan-out | `WheelGrid::popup`/`pane`, `at_cell`, `reports` | 85 → 46 |

`WheelGrid` is the daemon's description of one grid — cell and pixel extents,
`sgr_pixel_mouse`, `mouse_reporting`, and (for a pane) `alternate_screen_active`
— with `popup()` and `pane()` as its two constructors. `WheelHit` is where
inside that grid one report aims: `report()` for one event and `reports()` for
a count. `spaceport-backend` re-exports both, and with them
`ClientMouseGeometry`, so a caller can name what a hit says about the grid.

## 3. The one thing deliberately not unified: how the position is measured

`at_pixels` and `at_cell` both end in the same private `hit()`, which decides
Pixels-vs-Cell and carries the geometry. They differ in the pixels they put in
a report, because the two clients measure differently and both are right:

- A desktop pointer is a pixel position, so `at_pixels` keeps the sub-cell
  precision the old expression had — `floor(x / (cols * cell_width) * width_px)`
  — which is bit-for-bit what `terminal.rs` computed before.
- A phone knows the cell it touched and nothing inside it, so `at_cell` reports
  that cell's top-left pixel with exact integer arithmetic
  (`cell * extent / cells`), which is also exactly what the old mobile
  expression produced.

Folding one into the other would have changed a phone report's pixel by a
rounding step (the float form of `cell / cells * extent` can floor one pixel
low), so the two entry points stay, and the shared part is everything that
decides *which* of them a report is.

## 4. What stayed where

- The desktop keeps its hit testing: which pane's `inner_rect` holds the
  pointer, `Bounds<Pixels>` from `CellMetrics`, and `popup_origin` centring a
  popup in the frame. `wheel_target` builds a `WheelGrid` once it knows the
  pane or popup, then asks it for the hit.
- The phone keeps its own bounds on the request: `column`/`row` inside the grid
  and `lines.unsigned_abs() > 128` are `MobileError::InvalidScroll`, a UniFFI
  boundary check the desktop makes elsewhere. **No UniFFI change**: `wheel()`
  keeps its name, arguments and return type, so the generated Swift and Kotlin
  are unchanged.
- `WheelSteps`, `WheelAccumulator`, `wheel_rows`, `PLATFORM_NOTCH_LINES` and
  the two-axis composition stay in the app: they are GPUI `ScrollEvent` and
  `Modifiers` work, and the phone has neither axes nor modifiers to composite.

`mobile/rust` gained one path dependency, `spaceport-backend`, beside the
`spaceport-client` and `spaceport-truncate` it already took from the desktop
workspace; its own `[workspace]` resolves the crate's real workspace root for
inherited metadata, as it already did for those two. No new crate, and nothing
in `Cargo.toml` at the root.

## 5. Tests

- `crates/spaceport-backend/src/wheel/tests.rs` (new, 180 lines): a program
  reading the wheel gets one single-step event per step and the scrollback one
  event carrying the count (also through an alternate-screen pane and a popup);
  a pixel program is told the pointer's own place and the rest the cell; an
  unmeasured grid reports cells; a report carries the geometry and modifiers.
- The desktop's fixtures built `WheelTarget` field by field; they now build the
  shared hit (`WheelGrid::pane(&wheel_pane(..)).at_cell(2, 3)`), with the pane
  itself as one shared fixture in `terminal/tests.rs`. Every existing assertion
  is unchanged except that it reads `target.hit.position()` /
  `target.hit.geometry()`.
- The phone's `tests::wheel_targets_the_displayed_program_and_bounds_a_single_gesture`
  passes unchanged, including the pixel report `{ x: 8, y: 16 }` for cell
  `(1, 1)`.

## 6. Validation

Run on this branch on macOS (`CARGO_PROFILE_DEV_DEBUG=0
CARGO_INCREMENTAL=0` throughout; `SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk`
from `python3 scripts/release/sparkle_framework.py acquire`):

| Command | Result |
| --- | --- |
| `cargo test -p spaceport-backend` | 7 passed (3 that were there, 4 new) |
| `cargo clippy -p spaceport-backend --all-targets -- -D warnings` | clean |
| `SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk cargo check -p spaceport-gpui --all-targets --features integration-test` | clean |
| `SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk cargo clippy -p spaceport-gpui --all-targets --features integration-test -- -D warnings` | clean |
| `cargo test -p spaceport-gpui` | 3400 passed, 10 ignored; `terminal::tests::wheel*` 9 passed |
| `cd mobile/rust && cargo check` | clean |
| `cd mobile/rust && cargo clippy --all-targets` | clean |
| `cd mobile/rust && cargo test` | 55 passed, 2 ignored (opt-in integration) |
| `cargo fmt --all`, `cd mobile/rust && cargo fmt`, `git diff --check` | clean |
| `just check-file-size`, `just check-sorted` | clean |

Not run: `just ci`, the Windows and Linux lint legs, `just test-gui`, any
native window check, and the iOS/Android app builds. Nothing here changes a
pixel, a word or a wire frame.

## 7. Left for the rest of wx72

Part 1 was the wheel rule only. The rest of the survey's duplication map
(`pane_map` → `spaceport-panes`, the navigation predicates, the activation
fence, `drain`, `history` pacing, `images` staging) is untouched, and so are
`spaceport-panes`, `spaceport-live` and the navigation files wx70 is working
in.
