// ABOUTME: Exercises real main-image uploads beside cached terminal cell regions.
// ABOUTME: Checks final layer order, asset pairing, placement changes, and atlas replacement.
use super::*;
use crate::{
    state::ConnectionStatus,
    terminal_painter::image_probe::{PaintedImage, Phase, Session},
};
use gpui::{Bounds, Window, point, px};

mod pane_scroll;
mod scrolling;

fn main_source(id: u32) -> SurfaceGraphicsSource {
    SurfaceGraphicsSource::Terminal {
        target: SurfaceGraphicsTarget::Pane {
            pane_id: "a".into(),
        },
        image_id: id,
    }
}

pub(super) fn surface() -> PaneSurfaceFrame {
    let mut cells = frame(&["aaaa|bbbb", "aaaa|bbbb"]);
    cells.cursor = Some(CursorState {
        x: 0,
        y: 0,
        visible: true,
        shape: 0,
    });
    cells.cells[0].modifier = crate::terminal::UNDERLINE;
    PaneSurfaceFrame {
        boot_id: String::new(),
        projection_revision: 0,
        surface_revision: 1,
        frame: cells,
        panes: vec![pane("a", rect(0, 0, 4, 2)), pane("b", rect(5, 0, 4, 2))],
        splits: vec![],
        popup: None,
        graphics: Default::default(),
    }
}

fn asset(
    source: SurfaceGraphicsSource,
    value: u8,
    z: i32,
) -> (SurfaceGraphicsPlacement, SurfaceGraphicsAsset) {
    let mut placed = placement(source);
    placed.asset.data_fingerprint = u64::from(value);
    placed.z = z;
    let encoded = SurfaceGraphicsAsset {
        key: placed.asset.clone(),
        data: vec![value, 20, 30, 255],
    };
    (placed, encoded)
}

fn install(
    view: &Entity<SheprdWindow>,
    assets: Vec<SurfaceGraphicsAsset>,
    cx: &mut VisualTestContext,
) {
    view.update(cx, |view, _| {
        view.live.status = ConnectionStatus::Connected;
        view.live.surface_images = Arc::new(assets.into_iter().collect());
    });
}

fn settled(
    view: &Entity<SheprdWindow>,
    surface: &PaneSurfaceFrame,
    session: &Session,
    cx: &mut VisualTestContext,
) -> Vec<(Owner, [EntityId; 3])> {
    // Four private assets need at most two batches under the unchanged decode limit.
    for _ in 0..3 {
        session.reset();
        draw(view, surface, cx);
        cx.run_until_parked();
    }
    session.reset();
    draw(view, surface, cx)
}

fn pixels(image: &PaintedImage) -> &[u8] {
    assert!(image.uploaded, "the real paint must create an atlas entry");
    image.texture.as_bytes(0).unwrap()
}

fn reuse(source: SurfaceGraphicsSource, cx: &mut TestAppContext) {
    let session = Session::start();
    let (view, cx) = cx.add_window_view(fixture_window);
    let mut surface = surface();
    surface.panes[0].scroll = Some(sheprd_client::protocol::PaneSurfaceScrollMetrics {
        offset_from_bottom: 0,
        max_offset_from_bottom: 100,
        viewport_rows: 2,
    });
    let first = draw(&view, &surface, cx);
    assert!(
        !first.is_empty(),
        "the image-free fixture must mount regions"
    );
    let (placed, encoded) = asset(source, 91, 2);
    surface.graphics.placements.push(placed);
    install(&view, vec![encoded], cx);
    let with_image = settled(&view, &surface, &session, cx);
    let actual = session.painted();
    assert_eq!(actual.len(), surface.graphics.placements.len());
    assert_eq!(pixels(&actual[0]), &[30, 20, 91, 255]);
    assert_eq!(
        with_image, first,
        "a decoded main image must retain unchanged cell regions"
    );
    let markers = cx.update(|window, _| main_cell_markers(&session, window));
    assert!(
        !markers.is_empty(),
        "the first frame must paint cell markers"
    );
    session.reset();
    assert_eq!(draw(&view, &surface, cx), first);
    assert_eq!(
        session.main_cell_callbacks(),
        0,
        "unchanged cells must replay without repainting"
    );
    assert_eq!(
        cx.update(|window, _| main_cell_markers(&session, window)),
        markers,
        "cached replay must preserve the cell markers and their layer order"
    );
    assert_eq!(pixels(&session.painted()[0]), &[30, 20, 91, 255]);
}

fn main_cell_markers(session: &Session, window: &Window) -> Vec<Phase> {
    session
        .scene(window)
        .into_iter()
        .filter_map(|(phase, _)| {
            matches!(phase, Phase::Cells { popup: false, .. }).then_some(phase)
        })
        .collect()
}

#[gpui::test]
fn terminal_main_image_reuses_cells_after_actual_decode(cx: &mut TestAppContext) {
    reuse(main_source(1), cx);
}

#[gpui::test]
fn pane_layer_image_reuses_cells_after_actual_decode(cx: &mut TestAppContext) {
    reuse(
        SurfaceGraphicsSource::PaneLayer {
            pane_id: "a".into(),
            layer_id: "picture".into(),
        },
        cx,
    );
}

fn orders(scene: &[(Phase, u32)], accepts: impl Fn(Phase) -> bool) -> (u32, u32) {
    let values: Vec<_> = scene
        .iter()
        .filter(|(phase, _)| accepts(*phase))
        .map(|(_, order)| *order)
        .collect();
    assert!(!values.is_empty(), "missing real scene phase: {scene:?}");
    (*values.iter().min().unwrap(), *values.iter().max().unwrap())
}

fn assert_order(session: &Session, window: &Window) {
    let scene = session.scene(window);
    let cells = |layer| {
        orders(&scene, |phase| {
            phase
                == Phase::Cells {
                    popup: false,
                    layer,
                }
        })
    };
    let below = orders(
        &scene,
        |phase| matches!(phase, Phase::Image { popup: false, z, .. } if z < 0),
    );
    let above = orders(
        &scene,
        |phase| matches!(phase, Phase::Image { popup: false, z, .. } if z >= 0),
    );
    let popup = orders(&scene, |phase| {
        matches!(
            phase,
            Phase::Cells { popup: true, .. } | Phase::Image { popup: true, .. }
        )
    });
    assert!(
        cells(Layer::Backgrounds).1 < below.0,
        "backgrounds must precede below-text images: {scene:?}"
    );
    assert!(
        below.1 < cells(Layer::Text).0,
        "below-text images must precede text: {scene:?}"
    );
    assert!(
        cells(Layer::Text).1 < cells(Layer::Decorations).0,
        "text must precede cursor and decorations: {scene:?}"
    );
    assert!(
        cells(Layer::Decorations).1 < above.0,
        "above-text images must cover cursor and decorations: {scene:?}"
    );
    assert!(
        above.1 < popup.0,
        "the popup must cover every main-grid layer: {scene:?}"
    );
}

#[gpui::test]
fn main_images_keep_real_scene_order_and_one_prepare_for_both_z_passes(cx: &mut TestAppContext) {
    let session = Session::start();
    let (view, cx) = cx.add_window_view(fixture_window);
    let mut surface = surface();
    surface.popup = Some(Box::new(ClientShellPopupSurface {
        terminal_id: "popup".into(),
        title: "Image order".into(),
        width: None,
        height: None,
        frame: frame(&["xx"]),
        mouse_reporting: false,
        sgr_pixel_mouse: false,
        pixel_width: 0,
        pixel_height: 0,
    }));
    let mut assets = Vec::new();
    for (source, value, z) in [
        (main_source(1), 41, -1),
        (main_source(2), 42, 3),
        (main_source(3), 43, 3),
        (
            SurfaceGraphicsSource::Terminal {
                target: SurfaceGraphicsTarget::Popup {
                    terminal_id: "popup".into(),
                },
                image_id: 4,
            },
            44,
            -1,
        ),
    ] {
        let (mut placed, encoded) = asset(source, value, z);
        // Main placements span both panes and their border; they must paint once globally.
        placed.cols = if value == 44 { 2 } else { 9 };
        surface.graphics.placements.push(placed);
        assets.push(encoded);
    }
    install(&view, assets, cx);
    settled(&view, &surface, &session, cx);
    let actual = session.painted();
    assert_eq!(actual.len(), surface.graphics.placements.len());
    let actual_pixels: Vec<_> = actual.iter().map(|image| pixels(image)[2]).collect();
    let actual_depths: Vec<_> = actual
        .iter()
        .map(|image| match image.phase {
            Phase::Image { z, .. } => z,
            Phase::Cells { .. } => panic!("an image observation must come from paint_image"),
        })
        .collect();
    assert_eq!(actual_depths, [-1, 3, 3, -1]);
    assert_eq!(
        actual_pixels,
        [41, 42, 43, 44],
        "equal z retains the placement order"
    );
    let epochs = session.preparations();
    assert_eq!(
        epochs.iter().map(|(popup, _)| *popup).collect::<Vec<_>>(),
        [false, true]
    );
    assert_eq!(
        epochs[1].1,
        epochs[0].1 + 1,
        "splitting main images must add no epoch"
    );
    assert_eq!(
        session.popup_leases(),
        vec![2; actual.len() - 1],
        "before popup preparation only the cache and test observations may own main textures"
    );
    cx.update(|window, _| assert_order(&session, window));
    // Repeat without changing a cell. After integration the cell markers must replay from their caches.
    session.reset();
    draw(&view, &surface, cx);
    cx.update(|window, _| assert_order(&session, window));
    assert_eq!(
        session.main_cell_callbacks(),
        0,
        "the ordered cell markers must come from cache replay on an unchanged draw"
    );
    assert_eq!(session.painted().len(), surface.graphics.placements.len());
}

#[gpui::test]
fn unchanged_main_image_reuploads_after_its_real_atlas_entry_is_removed(cx: &mut TestAppContext) {
    let session = Session::start();
    let (view, cx) = cx.add_window_view(fixture_window);
    let mut surface = surface();
    let (placed, encoded) = asset(main_source(1), 81, 2);
    surface.graphics.placements.push(placed);
    install(&view, vec![encoded], cx);
    settled(&view, &surface, &session, cx);
    let uploaded = session.painted().remove(0);
    assert_eq!(pixels(&uploaded), &[30, 20, 81, 255]);
    cx.update(|window, _| {
        window.drop_image(uploaded.texture.clone()).unwrap();
        assert!(!window.has_image_atlas_entry(&uploaded.texture));
    });
    session.reset();
    draw(&view, &surface, cx);
    let repainted = session.painted();
    assert_eq!(repainted.len(), surface.graphics.placements.len());
    assert!(
        Arc::ptr_eq(&repainted[0].texture, &uploaded.texture),
        "the decoded cache entry remains unchanged"
    );
    assert_eq!(pixels(&repainted[0]), &[30, 20, 81, 255]);
}

#[gpui::test]
fn retained_picture_paints_its_paired_assets_until_the_next_surface_is_presented(
    cx: &mut TestAppContext,
) {
    let session = Session::start();
    let (view, cx) = cx.add_window_view(fixture_window);
    let mut surface = surface();
    let (placed, encoded) = asset(main_source(1), 71, 2);
    surface.graphics.placements.push(placed);
    install(&view, vec![encoded], cx);
    settled(&view, &surface, &session, cx);
    let held = view.read_with(cx, |view, _| Arc::downgrade(&view.live.surface_images));
    let (next, encoded) = asset(main_source(2), 72, 2);
    view.update(cx, |view, _| {
        view.live.surface = None;
        view.live.surface_images = Arc::new([encoded].into_iter().collect());
    });
    session.reset();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(
        held.upgrade().is_some(),
        "the presented picture keeps its image set"
    );
    let actual = session.painted();
    assert_eq!(actual.len(), surface.graphics.placements.len());
    assert_eq!(pixels(&actual[0]), &[30, 20, 71, 255]);
    surface.graphics.placements = vec![next];
    settled(&view, &surface, &session, cx);
    let actual = session.painted();
    assert_eq!(pixels(&actual[0]), &[30, 20, 72, 255]);
    assert!(
        held.upgrade().is_none(),
        "replacing the picture releases its encoded asset set"
    );
}

#[gpui::test]
fn moved_removed_and_readded_main_placements_keep_real_geometry(cx: &mut TestAppContext) {
    let session = Session::start();
    let (view, cx) = cx.add_window_view(fixture_window);
    let mut surface = surface();
    let (placed, encoded) = asset(main_source(1), 61, -1);
    surface.graphics.placements.push(placed);
    install(&view, vec![encoded], cx);
    settled(&view, &surface, &session, cx);
    let initial = session.painted().remove(0);
    let cell_width = view.read_with(cx, |view, _| view.cell_width);
    surface.graphics.placements[0].x = 5;
    session.reset();
    draw(&view, &surface, cx);
    let moved = session.painted().remove(0);
    assert_eq!(
        moved.bounds,
        Bounds::new(
            initial.bounds.origin + point(px(5. * cell_width), px(0.)),
            initial.bounds.size
        )
    );
    assert_eq!(pixels(&moved), pixels(&initial));
    let placed = surface.graphics.placements.pop().unwrap();
    session.reset();
    draw(&view, &surface, cx);
    assert!(
        session.painted().is_empty(),
        "removed placements must leave the scene"
    );
    assert!(
        session.preparations().is_empty(),
        "an empty main target needs no image pass"
    );
    surface.graphics.placements.push(placed);
    session.reset();
    draw(&view, &surface, cx);
    let readded = session.painted().remove(0);
    assert_eq!(readded.bounds, moved.bounds);
    assert_eq!(pixels(&readded), pixels(&moved));
}
