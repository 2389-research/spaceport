# Caching a mirrored group's terminal picture

The follow-up `gg9v`'s mark-rate work left open: a parked or mirrored group's
terminal re-enters its picture canvas on every frame of the window, a moving
mark's included, while the window's own surface replays its cached regions and
paints no cells. The profile could not price that re-entry, because its fixture
gave the mirror no cells to paint (0 quads). Below is a fixture that does paint
them, what a frame costs with it, and the cut that removes it.

Every number was taken on one machine (macOS, 1920x1080 viewport, release
profile) with the same method: count what the frame painted, then time the frame.

## The fixture

`crates/sheprd-gpui/src/group_terminals/tests/mirror_canvas.rs`:

- the sidebar fixture's 40-workspace snapshot, connected;
- the window's own tab `t0` carrying output on every row of an 80x24 grid (one
  pane covering it, with a cursor), or 100x56 for a group filling half of a
  1920x1080 window;
- a split editor whose left group holds that tab and whose right group mirrors
  it — `Shown::Elsewhere("t0")`, the group picking a tab another group shows —
  so the right group's body is `render_terminal_mirror`'s picture canvas;
- the frame measured is the one a moving mark asks for: `window.draw` with
  nothing invalidated, so every cached view replays and only the elements that
  cannot replay paint.

`window.draw` without `window.refresh` is exactly the frame a mark's
invalidation draws: GPUI renders the window's tree again, and a cached view
whose entity was not notified replays its recorded scene instead of painting.

Run it in release, either test measured with `--nocapture`:

```sh
cargo test --release -p sheprd-gpui --bin spaceport -- mirror_canvas --ignored --nocapture
```

The measurement is the difference between that window with the mirror painting
its grid and the same window with the mirrored tab reduced to one cell. Both
halves are needed: a whole mark frame also draws the 40-workspace sidebar, the
tab strip, the marks and the chrome, which the mirror's share must not carry.

## Before

Paint counts, with the `integration-test` feature's `Counts`:

```
Counts { shapes: 0, quads: 24, glyphs: 1696, decorations: 1, paints: 1, paint_errors: 0, .. }
```

One entry into the picture canvas per mark frame (`paints: 1`), painting the
mirrored group's whole grid: 1696 glyphs, the 24 rows' background bands, and the
cursor. The `gg9v` figure of 1.00 entries per frame is what this reproduces.

Times, 200 mark frames after 10 warm-up frames, microseconds per frame:

| mirrored grid | mark frame with the mirror | same frame, mirror holds one cell | the mirror's share |
| --- | --- | --- | --- |
| 80x24 (1920 cells) | 503.0 us | 399.1 us | **103.9 us** |
| 100x56 (5600 cells) | 695.0 us | 399.1 us | **295.9 us** |

So a mirrored group's picture was 21-43% of the whole mark frame's CPU, and one
mirrored group cost 74% of everything else the frame did. At the mark's 20 fps
that is 2.1-5.9 ms of CPU per second: 0.2-0.6% of one core per mirrored or
parked group, every group in a split laid on top of that. The cost is linear in
cells (2.9x the cells, 2.85x the time).

`gg9v` measured the same window shape at 4.0 ms per native frame in the release
driver, which includes the platform's dispatch as well as this scene
construction; those 104-296 us are the same CPU work, bought on every frame.

## The cut

The picture is a cached view now, the same treatment `window::regions` gives the
window's own cells:

- `group_terminals/picture.rs` holds `PictureView`, a view whose canvas paints
  the picture exactly as the element built each frame did (frame, popup, images,
  content mask), and `GroupPicture`, one group's picture with the view painting
  it.
- `SheprdWindow::group_pictures` keeps one `GroupPicture` per group. `render_body`
  takes the last frame's set, each group keeps its own view out of it while the
  picture and the look are unchanged, and whatever no group takes is dropped
  with the scene it recorded.
- Unchanged means the same frame and image set (by `Arc` identity) through the
  same font, cell metrics and ground. A changed picture takes a fresh view,
  which has no scene to replay, so output still paints on the frame it arrives.
- A picture placing images is never reused, and is repainted whole instead: a
  replayed scene still samples a texture no paint looks up. `terminal_regions`
  refuses for the same reason.
- The canvas that sizes a parked connection to its box stays where it was, as a
  prepaint-only element beside the view.

## After

| mirrored grid | mark frame with the mirror | same frame, mirror holds one cell | the mirror's share |
| --- | --- | --- | --- |
| 80x24 | 424.3 us / 431.3 us | 429.4 us / 405.5 us | -5.0 us / +25.8 us |
| 100x56 | 400.7 us / 400.2 us | 429.4 us / 405.5 us | -28.6 us / -5.3 us |

Each pair is one run of the final binary, and the share is now inside the
noise: the one-cell baseline alone reads 399.1, 429.4 and 405.5 us across runs,
so a share of a few tens of microseconds is the machine, not the mirror. The
count says the same thing — `paints: 0, quads: 0, glyphs: 0` on a mark frame.
The whole frame fell by 79 us for the 80x24 group and 294 us for the 100x56 one.

## Tests

- `a_mark_frame_repaints_no_mirrored_cells` (needs `integration-test`, which is
  where `Counts` lives) fails before the cut with `paints: 1, quads: 24,
  glyphs: 1696`, and passes after it with `paints: 0`.
- `a_mirrored_group_paints_the_cells_that_changed` is the other side: one
  changed cell repaints the grid once (`paints: 1`, the glyphs it holds), and
  the frame after it paints nothing again. It also proves the view is replaced
  rather than rewritten in place.
- `a_parked_group_takes_the_terminal_padding_too` and
  `a_tab_moved_to_another_group_paints_there_at_once` keep the geometry: the
  picture still lands in the content box inside the terminal padding, and the
  mirror still gives way to the live terminal when pressed.

## What this does not cover

- No native window was measured: this is headless scene construction, the same
  measure `window::regions`' own tests use. `gg9v`'s 4.0 ms native frame for the
  same shape suggests the share is unchanged there, since the work removed is
  CPU scene construction, but that was not run here.
- A mirrored picture with kitty graphics placements still paints whole on every
  frame, as the window's own regions do; cutting that needs an image lifetime
  the replayed scene can hold.
- GPU work is unchanged: the primitives recorded are the same either way.

## Verification

```sh
cargo check -p sheprd-gpui --all-targets
cargo test -p sheprd-gpui --bin spaceport --locked -- mirror group_terminals regions
cargo test -p sheprd-gpui --features integration-test --bin spaceport --locked -- mirror
cargo test -p sheprd-gpui --bin spaceport --locked          # 3707 passed, 10 ignored, 5 pasteboard tests failing with or without this change
cargo clippy -p sheprd-gpui --all-targets
cargo fmt --all -- --check
just check-sorted
```

The five `window::clipboard::pasteboard_tests` failures are the host, not this
change: they panic on `+[NSPasteboard pasteboardWithUniqueName]` returning NULL,
and they fail the same way on the tree without it.
