// ABOUTME: Exercises fractional wheel input beside images through the framed endpoint.
// ABOUTME: Preserves smooth input when image placements enter or cross a pane.
use super::*;
use gpui::{ScrollDelta, ScrollEvent, TouchPhase};
use sheprd_client::protocol::PaneSurfaceScrollMetrics;

mod cancellation;

fn prepare(view: &mut SheprdWindow, endpoint: Endpoint, cx: &mut Context<SheprdWindow>) {
    prepare_mouse(view, endpoint, cx);
    let surface = Arc::make_mut(view.live.surface.as_mut().unwrap());
    surface.panes[0].mouse_reporting = false;
    surface.panes[0].scroll = Some(PaneSurfaceScrollMetrics {
        offset_from_bottom: 0,
        max_offset_from_bottom: 1000,
        viewport_rows: 22,
    });
}

fn quarter(view: &SheprdWindow) -> ScrollEvent {
    ScrollEvent {
        position: mouse_position(view, 3.5, 4.5),
        delta: ScrollDelta::Pixels(point(px(0.), px(view.config.terminal.line_height() / 4.))),
        touch_phase: TouchPhase::Moved,
        ..Default::default()
    }
}

#[gpui::test]
fn image_free_pane_wheel_ignores_images_outside_its_content(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    // Pane one occupies columns 1..39. Geometry wins over source ownership.
    for (x, cols) in [(41, 1), (39, 1), (38, 4), (2, 1)] {
        let (endpoint, mut server) = connected_endpoint("ssh:wheel-images");
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                prepare(view, endpoint, cx);
                let mut image = wheel_slides::placement();
                image.x = x;
                image.y = 2;
                image.cols = cols;
                image.asset.source = SurfaceGraphicsSource::PaneLayer {
                    pane_id: "w1:p2".into(),
                    layer_id: "picture".into(),
                };
                Arc::make_mut(view.live.surface.as_mut().unwrap())
                    .graphics
                    .placements = vec![image];
                view.presentation.frame(&view.live);
                view.scroll_wheel(&quarter(view), window, cx);
                assert!(
                    view.presentation.scroll.offset().is_some(),
                    "x={x} cols={cols}"
                );
                view.send(ClientPaneInputEvent::TextCommit("sentinel".into()), cx);
            });
        });
        let ClientMessage::ClientShellPaneInput { pane_id, events } = server.receive() else {
            panic!("missing pane input");
        };
        assert_eq!(pane_id, "w1:p1");
        let expected = vec![mouse_event(ClientMouseKind::ScrollUp, 2, 3)];
        assert_eq!(events, expected);
    }
}

#[gpui::test]
fn changing_image_placements_preserves_fractional_input(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let (endpoint, _server) = connected_endpoint("ssh:wheel-images");
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            prepare(view, endpoint, cx);
            let mut image = wheel_slides::placement();
            image.x = 41;
            image.y = 2;
            Arc::make_mut(view.live.surface.as_mut().unwrap())
                .graphics
                .placements = vec![image];
            view.presentation.frame(&view.live);
            view.scroll_wheel(&quarter(view), window, cx);
            assert!(view.presentation.scroll.offset().is_some());
            Arc::make_mut(view.live.surface.as_mut().unwrap())
                .graphics
                .placements[0]
                .x = 2;
            view.presentation.frame(&view.live);
            assert!(view.presentation.scroll.offset().is_some());
            Arc::make_mut(view.live.surface.as_mut().unwrap())
                .graphics
                .placements
                .clear();
            view.presentation.frame(&view.live);
            view.scroll_wheel(&quarter(view), window, cx);
            assert!(view.presentation.scroll.offset().is_some());
        });
    });
}
