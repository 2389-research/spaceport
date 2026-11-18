// ABOUTME: Checks redraw requests when a fractional gesture meets an image-bearing pane.
// ABOUTME: Observes the surface signal before any draw can hide a missing invalidation.
use super::*;
use gpui::{Entity, IntoElement, Render, Styled, Window};
use std::{cell::Cell, rc::Rc, time::Duration};

struct WheelFixture {
    view: Entity<SheprdWindow>,
    renders: Rc<Cell<usize>>,
}

impl Render for WheelFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        self.view.update(cx, |view, cx| {
            view.surface_signal.read(cx);
            SheprdWindow::shortcut_scope(cx)
                .track_focus(&view.focus)
                .size_full()
                .on_scroll(cx.listener(SheprdWindow::scroll_wheel))
        })
    }
}

#[gpui::test]
fn unscrollable_image_pane_requests_cancellation_redraw_before_any_draw(
    cx: &mut gpui::TestAppContext,
) {
    let renders = Rc::new(Cell::new(0));
    let (fixture, cx) = cx.add_window_view(|window, cx| WheelFixture {
        view: cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)),
        renders: renders.clone(),
    });
    let view = fixture.update(cx, |fixture, _| fixture.view.clone());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let (endpoint, mut server) = connected_endpoint("ssh:wheel-image-cancel");
    let event = view.update(cx, |view, cx| {
        prepare(view, endpoint, cx);
        let surface = Arc::make_mut(view.live.surface.as_mut().unwrap());
        surface.panes[1].mouse_reporting = false;
        surface.panes[1].scroll = None;
        let mut image = wheel_slides::placement();
        image.x = 42;
        image.y = 3;
        surface.graphics.placements = vec![image];
        surface.frame.cursor = Some(CursorState {
            x: 3,
            y: 4,
            visible: true,
            shape: 0,
        });
        let frame = &mut surface.frame;
        frame.cells = (0..frame.height)
            .flat_map(|y| (0..frame.width).map(move |x| (x, y)))
            .map(|(x, y)| CellData {
                symbol: if x == 2 { y.to_string() } else { " ".into() },
                fg: 0,
                bg: 0,
                modifier: 0,
                skip: false,
                hyperlink: None,
            })
            .collect();
        view.presentation.frame(&view.live);
        // An old gesture has no remaining animation refresh to rescue cancellation.
        let now = crate::motion::now();
        assert_eq!(
            view.presentation.scroll.wheel(
                view.live.surface.as_ref().unwrap(),
                "w1:p1",
                0.25,
                now - Duration::from_secs(1),
            ),
            Some(1)
        );
        let surface = Arc::make_mut(view.live.surface.as_mut().unwrap());
        surface.panes[0].scroll.as_mut().unwrap().offset_from_bottom = 1;
        let width = usize::from(surface.frame.width);
        surface.frame.cells.rotate_right(width);
        view.presentation.frame(&view.live);
        let slide = view.presentation.scroll.slide(now).unwrap();
        assert_eq!(slide.offset, -0.75);
        assert!(!view.presentation.scroll.moving(now));
        let height = view.config.terminal.line_height();
        assert_eq!(view.ime_shift(), px(-0.75 * height));
        let position = mouse_position(view, 3.5, 4.5);
        assert_eq!(view.grid_position(position).1, 5.25 * height);
        ScrollEvent {
            position: mouse_position(view, 43.5, 4.5),
            ..quarter(view)
        }
    });
    let before_renders = renders.get();
    let notifications = Rc::new(Cell::new(0));
    let notified_at_render = Rc::new(Cell::new(None));
    let _observation = cx.update(|_, cx| {
        let signal = view.read(cx).surface_signal.clone();
        let notifications = notifications.clone();
        let notified_at_render = notified_at_render.clone();
        let renders = renders.clone();
        cx.observe(&signal, move |_, _| {
            notifications.set(notifications.get() + 1);
            notified_at_render.set(Some(renders.get()));
        })
    });

    // Dispatch the real listener without an explicit draw, refresh, or notify.
    cx.simulate_event(event);
    view.read_with(cx, |view, _| {
        assert_eq!(view.presentation.scroll.offset(), None);
        assert_eq!(view.ime_shift(), px(0.));
        let position = mouse_position(view, 3.5, 4.5);
        assert_eq!(
            view.grid_position(position).1,
            4.5 * view.config.terminal.line_height()
        );
    });
    assert_eq!(
        notifications.get(),
        1,
        "clearing a painted fractional offset must request a surface redraw"
    );
    assert_eq!(
        notified_at_render.get(),
        Some(before_renders),
        "the surface notification must precede the automatic render"
    );
    assert_eq!(
        renders.get(),
        before_renders + 1,
        "the notified surface must render without an explicit draw"
    );

    view.update(cx, |view, cx| {
        view.send(ClientPaneInputEvent::TextCommit("sentinel".into()), cx);
    });
    assert_eq!(
        server.receive(),
        ClientMessage::ClientShellPaneInput {
            pane_id: "w1:p1".into(),
            events: vec![ClientPaneInputEvent::TextCommit("sentinel".into())],
        },
        "a sub-row image-pane wheel must emit no whole-row input"
    );
}
