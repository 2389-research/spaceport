// ABOUTME: Checks rejected Core view commands remain pending and visible.
// ABOUTME: Uses disconnected local queues without starting a Core runtime.
#![allow(clippy::unwrap_used)]
use super::CoreTerminalView;
use crate::core_terminals::worker::Link;
use crate::sidebar::layout_tests::fixture_window;
use gpui::{AppContext, Bounds, point, px, size};
use sheprd_backend::TerminalId;

mod accessibility;
mod paste;

#[gpui::test]
fn rejected_resize_does_not_record_a_size_as_sent(cx: &mut gpui::TestAppContext) {
    let (window_view, cx) = cx.add_window_view(fixture_window);
    let view = cx.update(|_, cx| {
        let owner = window_view.read(cx);
        let look = owner.core_look();
        let painter = owner.painter.clone();
        cx.new(|cx| {
            CoreTerminalView::new(
                TerminalId::new("bounded-view").unwrap(),
                Link::for_test(),
                look,
                painter,
                false,
                cx,
            )
        })
    });
    view.update(cx, |view, cx| {
        view.fit(Bounds::new(point(px(0.), px(0.)), size(px(800.), px(600.))));
        assert_eq!(
            view.last_queued_size, None,
            "a rejected resize must remain pending"
        );
        assert!(view.poll(cx), "the view must consume the rejection notice");
        assert!(
            view.status().is_some(),
            "the rejected command needs a visible status"
        );
    });
}
