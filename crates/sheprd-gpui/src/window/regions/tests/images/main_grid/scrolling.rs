// ABOUTME: Scrolls decoded images with the same fractional geometry as their terminal cells.
// ABOUTME: Checks historical edge painting without retaining additional encoded image assets.
use super::*;
use sheprd_client::protocol::PaneSurfaceScrollMetrics;
use std::time::Duration;

mod crossing;
mod ownership;

fn scrolling_surface() -> PaneSurfaceFrame {
    let mut surface = surface();
    surface.panes[0].scroll = Some(PaneSurfaceScrollMetrics {
        offset_from_bottom: 10,
        max_offset_from_bottom: 100,
        viewport_rows: 2,
    });
    surface
}

fn wheel(view: &Entity<SheprdWindow>, rows: f32, cx: &mut VisualTestContext) {
    view.update(cx, |view, _| {
        assert_eq!(
            view.presentation.scroll.wheel(
                view.live.surface.as_ref().unwrap(),
                "a",
                rows,
                crate::motion::now() - Duration::from_secs(1),
            ),
            Some(rows.signum() as i16),
            "a decoded image must not disable fractional pane scrolling"
        );
    });
}

fn advance(surface: &mut PaneSurfaceFrame) {
    surface.panes[0].scroll.as_mut().unwrap().offset_from_bottom = 11;
    // Move the real decorated row, so the presentation can correlate the scroll.
    let width = usize::from(surface.frame.width);
    surface.frame.cells.rotate_right(width);
}

#[gpui::test]
fn decoded_image_moves_with_its_fractional_cell_rows(cx: &mut TestAppContext) {
    let session = Session::start();
    let (view, cx) = cx.add_window_view(fixture_window);
    let mut surface = scrolling_surface();
    let (placed, encoded) = asset(main_source(1), 91, 2);
    surface.graphics.placements.push(placed);
    install(&view, vec![encoded], cx);
    settled(&view, &surface, &session, cx);
    let before = session.painted().remove(0);
    assert_eq!(pixels(&before), &[30, 20, 91, 255]);
    wheel(&view, 0.25, cx);
    advance(&mut surface);
    surface.graphics.placements[0].y = 1;
    session.reset();
    draw(&view, &surface, cx);
    let (offset, height) = view.read_with(cx, |view, _| {
        (view.presentation.scroll.offset(), view.metrics().height)
    });
    assert_eq!(offset, Some((surface.panes[0].inner_rect, -0.75)));
    let after = session.painted().remove(0);
    let shifted = cx.update(|window, _| {
        super::super::super::super::painted_shift(-0.75, height, window.scale_factor())
    });
    assert_eq!(
        after.bounds.origin.y,
        before.bounds.origin.y + px(height + shifted)
    );
    assert_eq!(after.bounds.size, before.bounds.size);
    assert_eq!(pixels(&after), pixels(&before));
    assert_eq!(session.preparations().len(), 1, "one main image epoch");
}

#[gpui::test]
fn historical_image_fills_the_exposed_edge_until_its_existing_asset_disappears(
    cx: &mut TestAppContext,
) {
    let session = Session::start();
    let (view, cx) = cx.add_window_view(fixture_window);
    let mut surface = scrolling_surface();
    let (mut placed, encoded) = asset(main_source(1), 91, 2);
    placed.y = 1;
    let key = placed.asset.clone();
    surface.graphics.placements.push(placed);
    install(&view, vec![encoded], cx);
    settled(&view, &surface, &session, cx);
    let before = session.painted().remove(0);
    let bytes = view.read_with(cx, |view, _| {
        Arc::downgrade(view.live.surface_images.get(&key).unwrap().data())
    });
    wheel(&view, 0.25, cx);
    advance(&mut surface);
    surface.graphics.placements.clear();
    session.reset();
    draw(&view, &surface, cx);
    let after = session.painted().remove(0);
    assert_eq!(pixels(&after), pixels(&before));
    assert!(after.bounds.origin.y > before.bounds.origin.y);
    assert!(after.bounds.size.height < before.bounds.size.height);
    assert_eq!(
        session.preparations().len(),
        1,
        "history shares the main epoch"
    );
    view.update(cx, |view, _| view.live.surface_images = Arc::default());
    session.reset();
    draw(&view, &surface, cx);
    assert!(
        view.read_with(cx, |view, _| view.presentation.scroll.offset())
            .is_none()
    );
    assert!(session.painted().is_empty());
    assert!(
        bytes.upgrade().is_none(),
        "scroll history must not lease encoded bytes"
    );
}

#[gpui::test]
fn thin_historical_fragment_preserves_the_decoded_texture_mapping(cx: &mut TestAppContext) {
    thin_fragment(0.75, cx);
}

#[gpui::test]
fn top_historical_fragment_preserves_the_decoded_texture_mapping(cx: &mut TestAppContext) {
    thin_fragment(-0.75, cx);
}

fn thin_fragment(rows: f32, cx: &mut TestAppContext) {
    let session = Session::start();
    let (view, cx) = cx.add_window_view(fixture_window);
    let mut surface = scrolling_surface();
    let (mut placed, mut encoded) = asset(main_source(1), 91, 2);
    placed.y = u16::from(rows > 0.);
    placed.asset.image_height = 2;
    placed.asset.data_len = 8;
    encoded.key = placed.asset.clone();
    encoded.data = vec![91, 20, 30, 255, 40, 50, 60, 255];
    surface.graphics.placements.push(placed);
    install(&view, vec![encoded], cx);
    settled(&view, &surface, &session, cx);
    let before = session.painted().remove(0);
    assert_eq!(pixels(&before), &[30, 20, 91, 255, 60, 50, 40, 255]);
    wheel(&view, rows, cx);
    if rows > 0. {
        advance(&mut surface);
    } else {
        surface.panes[0].scroll.as_mut().unwrap().offset_from_bottom = 9;
        let width = usize::from(surface.frame.width);
        surface.frame.cells.rotate_left(width);
    }
    surface.graphics.placements.clear();
    session.reset();
    draw(&view, &surface, cx);
    let after = session.painted().remove(0);
    assert!(after.bounds.size.height < before.bounds.size.height / 2.);
    assert_eq!(
        after.mapped.size.height, before.mapped.size.height,
        "the actual paint_image mapping must stay whole while the fragment mask clips it"
    );
}
