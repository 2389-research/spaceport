//! A mirrored group's terminal picture on a frame nothing in it changed.
//!
//! A moving mark asks for a frame while an agent works, and that frame draws
//! the whole window again. The main surface's cells are cached views, which
//! replay their scene, so such a frame paints no cells. This fixture gives a
//! mirrored group real output to paint, so what its own canvas costs on those
//! frames can be counted and timed; see `docs/notes-mirror-canvas-cache.md`.

use super::*;
use crate::sidebar::layout_tests::full_draw;
use gpui::TestAppContext;
use spaceport_client::protocol::{
    CellData, CursorState, PaneSurfaceFrame, PaneSurfacePane, SurfaceRect,
};

/// A grid half the width of a laptop window's group, in cells.
const GRID: (u16, u16) = (80, 24);
/// A large shared group: more cells than a real half-window group holds at
/// the app's cell metrics, as an upper bound.
const SHARED_GROUP: (u16, u16) = (100, 56);

/// Output on every row of `grid`, as the daemon projects a pane running a
/// command.
fn output(snapshot: &ClientShellSnapshot, pane: &str, grid: (u16, u16)) -> Arc<PaneSurfaceFrame> {
    let lines = [
        "$ cargo test --workspace --locked",
        "   Compiling spaceport-gpui v0.1.0 (/Users/clint/code/spaceport)",
        "    Finished `test` profile [optimized] target(s) in 41.28s",
        "     Running unittests src/main.rs (target/release/deps/spaceport)",
        "test terminal::tests::wire_colors_are_not_argb ... ok",
    ];
    let width = usize::from(grid.0);
    let cells = (0..width * usize::from(grid.1))
        .map(|index| {
            let line = lines[(index / width) % lines.len()].as_bytes();
            CellData {
                symbol: char::from(line[index % line.len()]).to_string(),
                fg: 0,
                bg: 0,
                modifier: 0,
                skip: false,
                hyperlink: None,
            }
        })
        .collect();
    let rect = SurfaceRect {
        x: 0,
        y: 0,
        width: grid.0,
        height: grid.1,
    };
    Arc::new(PaneSurfaceFrame {
        boot_id: snapshot.boot_id.clone(),
        projection_revision: snapshot.revision,
        surface_revision: 1,
        frame: FrameData {
            width: grid.0,
            height: grid.1,
            cells,
            cursor: Some(CursorState {
                x: 0,
                y: 5.min(grid.1 - 1),
                visible: true,
                shape: 0,
            }),
            hyperlinks: vec![],
            graphics: vec![],
        },
        panes: vec![PaneSurfacePane {
            pane_id: pane.into(),
            content_revision: 1,
            rect,
            inner_rect: rect,
            scrollbar_rect: None,
            scroll: None,
            focused: true,
            mouse_reporting: false,
            sgr_pixel_mouse: false,
            alternate_screen_active: false,
            pixel_width: u32::from(grid.0) * 8,
            pixel_height: u32::from(grid.1) * 17,
        }],
        splits: vec![],
        popup: None,
        graphics: Default::default(),
    })
}

impl SpaceportWindow {
    /// Gives the window's connection the frame of `grid` cells `pane` holds.
    fn show_grid(&mut self, grid: (u16, u16)) {
        let snapshot = self.live.snapshot.clone().unwrap();
        self.live.surface = Some(output(&snapshot, "p0", grid));
        *self.endpoints[self.selected_endpoint].live_mut() = self.live.clone();
    }
}

/// A split editor whose left group holds the window's own tab, `t0`, with
/// output on it, and whose right group mirrors that tab: the group picking a
/// tab another group shows, as an editor shows one file in two groups.
fn split_window_mirroring_t0(
    cx: &mut TestAppContext,
) -> (Entity<SpaceportWindow>, &mut VisualTestContext) {
    let (view, cx) = fixture_window_on_t0(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.show_grid(GRID);
            view.command(Command::SplitEditor, window, cx);
            let slots = view.group_slots();
            let (left, right) = (slots[0].id, slots[1].id);
            let layout = view.ensure_layout().unwrap();
            layout.show(right, Pick::Herdr("t0".into()));
            layout.activate(left);
            view.set_primary_group(left);
        });
        full_draw(window, cx).clear(cx);
    });
    let slots = view.read_with(cx, |view, _| view.group_slots());
    let shown = view.read_with(cx, |view, cx| view.group_shown(slots[1].id, cx));
    assert_eq!(shown, Shown::Elsewhere("t0".into()));
    (view, cx)
}

/// Draws a frame nothing in the window invalidated, as a moving mark's does.
fn mark_frame(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

/// What a frame nothing in the window invalidated painted.
#[cfg(feature = "integration-test")]
fn mark_frame_counts(cx: &mut VisualTestContext) -> crate::performance::Counts {
    cx.update(|window, cx| {
        *cx.default_global::<crate::performance::Counts>() = crate::performance::Counts::default();
        window.draw(cx).clear(cx);
        *cx.default_global::<crate::performance::Counts>()
    })
}

#[cfg(feature = "integration-test")]
#[gpui::test]
fn a_mark_frame_repaints_no_mirrored_cells(cx: &mut TestAppContext) {
    let (_view, cx) = split_window_mirroring_t0(cx);
    let counts = mark_frame_counts(cx);
    assert_eq!(counts.paints, 0, "{counts:?}");
    assert_eq!((counts.quads, counts.glyphs), (0, 0), "{counts:?}");
}

/// The other side of the cache: output in the tab a group mirrors is painted
/// at once, and the frame after it is replayed again.
#[cfg(feature = "integration-test")]
#[gpui::test]
fn a_mirrored_group_paints_the_cells_that_changed(cx: &mut TestAppContext) {
    let (view, cx) = split_window_mirroring_t0(cx);
    assert_eq!(mark_frame_counts(cx).paints, 0);
    eprintln!(
        "MIRROR-CANVAS theme background before: {:06x}",
        view.read_with(cx, |view, _| view.theme.background)
    );
    cx.update(|_, cx| {
        view.update(cx, |view, _| {
            let surface = Arc::make_mut(view.live.surface.as_mut().unwrap());
            surface.frame.cells[0].symbol = "x".into();
            let live = view.live.clone();
            *view.endpoints[view.selected_endpoint].live_mut() = live;
        });
    });
    let changed = mark_frame_counts(cx);
    assert_eq!(changed.paints, 1, "{changed:?}");
    assert!(changed.glyphs > 0, "{changed:?}");
    assert_eq!(mark_frame_counts(cx).paints, 0);
}

/// The mirror's cached look includes the terminal theme: cell colors come
/// from it, so a theme change must retire the cached view even though the
/// mirrored cells and metrics are unchanged — as the window's own regions
/// already do. The stored picture's look must carry the new theme once the
/// frame that saw the change has rendered.
#[cfg(feature = "integration-test")]
#[gpui::test]
fn a_theme_change_repaints_the_mirror(cx: &mut TestAppContext) {
    let (view, cx) = split_window_mirroring_t0(cx);
    let before = view.read_with(cx, |view, _| {
        view.group_pictures
            .iter()
            .map(|p| p.cached_look().theme.background)
            .collect::<Vec<_>>()
    });
    assert!(!before.is_empty(), "the mirror's picture is cached");
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.theme = spaceport_settings::Theme {
                background: 0x000000,
                ..view.theme.clone()
            };
            cx.notify();
        });
    });
    mark_frame(cx);
    let after = view.read_with(cx, |view, _| {
        view.group_pictures
            .iter()
            .map(|p| p.cached_look().theme.background)
            .collect::<Vec<_>>()
    });
    assert_eq!(
        after,
        vec![0x000000; after.len()],
        "the cached picture's look follows the theme"
    );
}

/// What a mark frame costs, in microseconds, once the window has settled on
/// `grid`. A mark frame is measured after a warm-up: the first frames after a
/// change paint what changed.
fn mark_frame_cost(
    view: &Entity<SpaceportWindow>,
    grid: (u16, u16),
    cx: &mut VisualTestContext,
) -> f64 {
    cx.update(|window, cx| {
        view.update(cx, |view, _| view.show_grid(grid));
        full_draw(window, cx).clear(cx);
    });
    for _ in 0..10 {
        mark_frame(cx);
    }
    let frames = 200;
    let start = Instant::now();
    for _ in 0..frames {
        mark_frame(cx);
    }
    start.elapsed().as_secs_f64() * 1e6 / f64::from(frames)
}

/// What a mark frame costs with the mirror painting a grid, against the same
/// frame with next to nothing on it. The difference is the mirror's canvas
/// re-entry. A measurement, not an assertion: run it in release with
/// `--nocapture`.
#[gpui::test]
#[ignore = "measures a release-mode frame cost; run with --release --ignored --nocapture"]
fn a_mark_frame_costs_the_mirror_its_grid(cx: &mut TestAppContext) {
    let (view, cx) = split_window_mirroring_t0(cx);
    let barest = mark_frame_cost(&view, (1, 1), cx);
    for grid in [GRID, SHARED_GROUP] {
        let painted = mark_frame_cost(&view, grid, cx);
        eprintln!(
            "MIRROR-CANVAS {}x{} cells: {:.1} us per mark frame ({painted:.1} us with the mirror, {barest:.1} us without it), viewport {:?}",
            grid.0,
            grid.1,
            painted - barest,
            cx.update(|window, _| window.viewport_size()),
        );
    }
}
