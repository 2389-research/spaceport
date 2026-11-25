# wx70: the phone's pane map and navigation predicates

Branch `feat/wx70-mobile-shared-1`, cut from `f778a8626c` (main after the
orphaned activation file was removed). Two of the survey's nearly free moves,
`pane_map` → `spaceport-panes` and the navigation predicates → one home the
phone can reach. No UniFFI signature changes: `MobilePaneMap`, `MobilePaneRect`,
`MobileView` and `MobileError` are untouched, so the generated Swift and Kotlin
are the same.

## 1. The pane map

`mobile/rust/src/projection/pane_map.rs` re-checked the daemon's pane
rectangles for overlap and bounds. `crates/spaceport-panes` derives rectangles
from the saved split tree (`Panes::rects`), so the rule that reads a *reported*
frame belongs beside it.

`crates/spaceport-panes/src/cells.rs` (91 lines) now holds:

| Item | Rule |
| --- | --- |
| `CellRect` | a pane's part of a frame, in whole cells from its top-left corner |
| `CellMap::new(columns, rows, panes)` | `Some` only when the rectangles tile the frame exactly once |
| `CellMap::columns` / `rows` / `panes` | the frame's size and each pane's cells, in the order reported |

A map is refused for a rectangle with no area or outside the frame, two panes
over one cell, a pane named twice, a cell no pane claims, or a frame with no
cells. Inside the frame and clear of each other those rectangles cover every
cell exactly once when their areas add up to the frame's, so the check needs no
frame-sized bitmap: it sums areas and compares rectangles pairwise, and the
largest allocation is the caller's own list.

`spaceport-panes` had only `serde`, `spaceport-backend` and `thiserror`, so
`mobile/rust` takes it and `spaceport-backend` (for `TerminalId`, which is the
crate's pane identity) as path dependencies; neither adds a crate the phone did
not already compile through `spaceport-client`.

`pane_map` keeps what is the phone's: the focused tab's pane list, the
"a single pane needs no map" policy, the frame's cell count against its size,
and the two UniFFI records. It maps the daemon's `SurfaceRect`s into a
`CellMap` and reads the records back out of it, so the rectangles the phone
sends Swift are the ones the shared rule accepted.

### The one behaviour tightening

The phone used to accept a frame whose rectangles neither overlapped nor left
the frame, even if they left cells to no pane — the file's own comment claimed
it "omits partial or ambiguous maps", which it did not. `CellMap` requires exact
coverage, so a gapped frame now yields no map and the phone falls back to pane
list navigation, exactly what a partial map was already meant to do. The
`tests/pane_map.rs` fixture pinned a gapped 8×3 layout; it now pins a tiling
one (`w1:p1` 0,0,5,3 and `w1:p2` 5,0,3,3) and one more invalid edit covers a
rectangle that leaves a column to no pane. Reviewers who want the old tolerance
back should say so: it is one condition in `CellMap::new` and one fixture line.

## 2. The navigation predicate

Three homes answered "does this destination still exist": the phone's
`projection.rs` `Navigation::{matches,present}`, `spaceport-gpui`'s
`palette::go_to::destination_exists`, and — for the focused check alone —
`spaceport-live`'s `activation::Fence::ready`. The obvious home is
`spaceport-live::navigation`, but the phone cannot take `spaceport-live`: it
depends on `spaceport-sound`, which brings `rodio` and an audio stack onto the
phone (the hazard wx71 records). So the set and its rules went to
`crates/spaceport-client/src/navigation.rs` (109 lines), one layer below
`spaceport-live`, which the phone already depends on.

| Rule | Who reads it | What it answers |
| --- | --- | --- |
| `NavigationTarget::exists_in(snapshot, boot)` | the app's Go To revalidation | the snapshot is this boot's, and still lists the destination — else `Missing::{Session,Workspace,Tab,Pane}` |
| `NavigationTarget::is_focused_in(snapshot)` | the activation fence, the phone's focus fence | the snapshot has already focused this destination |
| `NavigationTarget::reachable_in(snapshot)` | the phone's staged route | the destination sits in the focused workspace and tab, so focusing one level per request reaches it |

`NavigationTarget<T>`, `OwnedNavigationTarget`, `as_deref` and the owning
`From<&NavigationTarget<T>>` moved with them and are unchanged. Every existing
path still resolves: `spaceport-live`'s `navigation` module is now a re-export,
so `spaceport_live::NavigationTarget`, `spaceport_palette::NavigationTarget` and
the app's `crate::navigation::{NavigationTarget, OwnedNavigationTarget}` façade
are the same names for the same type. `spaceport-gpui`'s
`palette::go_to::destination_exists` keeps its signature and its `Error`
variants and is now a match over `Missing`; its test is unchanged and passes.
The fence's inline match became `target.is_focused_in(snapshot)`.

The phone's `Navigation` enum (which it never shared, being `pub(crate)`) is
gone; `projection.rs` re-exports the shared `NavigationTarget` for the
projection, the route and the tests, so the phone spells the concept once.
`matches` is `is_focused_in` and `present` is `reachable_in`, same answers: the
`present` it replaced also required a tab to sit in the focused workspace and a
pane in the focused tab, which is what `reachable_in` says and what the
`a_single_pane_focus_rejects_a_pane_outside_the_selected_tab` test pins.
`Route::present` (the whole workspace → tab → pane chain, plus the expected
agent) stayed: it is the route's own identity, not a destination predicate.

`crates/spaceport-palette/src/navigation.rs` was a fourth, dead copy — no
`mod navigation;` declared it, and the crate re-exported the live one. It is
deleted, as `f778a8626c` deleted the activation file that move orphaned.

## 3. Tests

| Added or re-pointed | Where |
| --- | --- |
| `a_destination_exists_until_its_snapshot_boot_changes`, `a_missing_destination_is_named_by_its_own_level`, `a_tab_or_pane_is_reachable_only_in_the_focused_chain`, `a_target_is_focused_only_at_its_own_level` | `crates/spaceport-client/src/tests/navigation.rs` (new, 67 lines) |
| `a_frame_maps_each_pane_to_its_cells`, `a_frame_map_keeps_only_layouts_that_claim_every_cell_once`, `a_frame_map_refuses_rectangles_outside_it_or_with_no_area`, `a_frame_map_names_each_pane_once`, `a_frame_map_needs_a_pane_and_a_cell` | `crates/spaceport-panes/src/tests/cells.rs` (new, 91 lines) |
| `tests/pane_map.rs` fixture tiled, gap edit added | `mobile/rust/src/tests/pane_map.rs` |
| `Navigation::` → `NavigationTarget::`, `present` → `reachable_in` | `mobile/rust/src/tests/{go_to,mod,resume}.rs` |

## 4. What ran

```sh
cargo fmt --all -- --check                                    # clean
cargo clippy -p spaceport-client -p spaceport-live -p spaceport-panes -p spaceport-palette --all-targets --locked
cargo clippy -p spaceport-gpui --all-targets --features integration-test --locked
cargo test   -p spaceport-panes                               # 14 passed
cargo test   -p spaceport-live                                # 5 passed
cargo test   -p spaceport-client --lib navigation             # 5 passed
cargo test   -p spaceport-client --lib focus_barrier          # 15 passed
cargo test   -p spaceport-gpui --features integration-test palette::   # 85 passed
cargo test   -p spaceport-gpui --features integration-test activation  # 21 passed
cd mobile/rust && cargo check && cargo fmt -- --check
cd mobile/rust && cargo clippy --all-targets                  # clean
cd mobile/rust && cargo test                                  # 55 passed, 2 live ignored
```

`CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0` throughout, and
`SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk` for the app. Not run locally:
the full workspace suite, `just ci`, the Windows/Linux lint legs, the mobile
Xcode and Gradle suites, and native pane-map QA — deferred to CI. No UniFFI
bindings were regenerated, which is safe because no record or error signature
changed.
