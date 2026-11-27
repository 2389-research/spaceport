// ABOUTME: Exercises terminal focus and resize updates against a bounded worker queue.
// ABOUTME: Checks that keyboard input remains a single attempt.
#![allow(clippy::unwrap_used)]
use super::{ClientPaneInputEvent, Command, CoreTerminalView, Link, TerminalId};
use gpui::{AppContext as _, Bounds, Point, TestAppContext, px, size};
use std::prelude::v1::test;

#[gpui::test]
fn a_full_worker_retries_focus_and_size_after_room_returns(cx: &mut TestAppContext) {
    let root = cx.add_window(crate::sidebar::layout_tests::fixture_window);
    root.update(cx, |view, window, cx| {
        let (link, queue) = Link::test_queue(1);
        let pane = cx.new(|cx| {
            CoreTerminalView::new(
                TerminalId::new("term").unwrap(),
                link,
                view.core_look(),
                view.painter.clone(),
                false,
                cx,
            )
        });
        pane.update(cx, |pane, _| {
            let key = ClientPaneInputEvent::TextCommit("a".into());
            pane.send(key.clone());
            pane.send(key.clone());
            let bounds = Bounds::new(Point::default(), size(px(400.), px(200.)));
            pane.fit(bounds);
            pane.report_focus(window);
            assert_eq!(pane.sent_size, None);
            assert_eq!(pane.sent_focus, None);
            assert_eq!(queue.try_recv().unwrap(), Command::Input(key));
            pane.fit(bounds);
            assert_eq!(
                queue.try_recv().unwrap(),
                Command::Resize(pane.sent_size.unwrap())
            );
            pane.report_focus(window);
            assert_eq!(queue.try_recv().unwrap(), Command::Focus(false));
            pane.fit(bounds);
            pane.report_focus(window);
            assert!(queue.try_recv().is_err());
        });
    })
    .unwrap();
}
