// ABOUTME: Draws a fractional pane beside a real decoded stationary image.
// ABOUTME: Verifies unchanged image geometry and one preparation epoch through actual cell painting.
use super::*;
use sheprd_client::protocol::PaneSurfaceScrollMetrics;
use std::time::Duration;

#[gpui::test]
fn decoded_neighbor_image_keeps_its_geometry_while_other_pane_slides(cx: &mut TestAppContext) {
    let session = Session::start();
    let (view, cx) = cx.add_window_view(fixture_window);
    let mut surface = surface();
    surface.panes[1].scroll = Some(PaneSurfaceScrollMetrics {
        offset_from_bottom: 10,
        max_offset_from_bottom: 100,
        viewport_rows: 2,
    });
    let (placed, encoded) = asset(main_source(1), 91, 2);
    surface.graphics.placements.push(placed);
    install(&view, vec![encoded], cx);
    settled(&view, &surface, &session, cx);
    let before = session.painted().remove(0);
    assert_eq!(pixels(&before), &[30, 20, 91, 255]);
    view.update(cx, |view, _| {
        let shown = view.live.surface.as_ref().unwrap();
        assert_eq!(
            view.presentation.scroll.wheel(
                shown,
                "b",
                0.25,
                crate::motion::now() - Duration::from_secs(1)
            ),
            Some(1)
        );
    });
    surface.panes[1].scroll.as_mut().unwrap().offset_from_bottom = 11;
    session.reset();
    draw(&view, &surface, cx);
    let offset = view.read_with(cx, |view, _| view.presentation.scroll.offset());
    assert_eq!(
        offset,
        Some((surface.panes[1].inner_rect, -0.75)),
        "the image-free pane must remain between rows after render"
    );
    let after = session.painted().remove(0);
    assert_eq!(after.bounds, before.bounds);
    assert_eq!(pixels(&after), pixels(&before));
    assert_eq!(
        session
            .preparations()
            .iter()
            .filter(|(popup, _)| !popup)
            .count(),
        1
    );
}
