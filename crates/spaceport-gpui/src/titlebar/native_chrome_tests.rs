// ABOUTME: Checks header geometry across window sizes and fullscreen states.
// ABOUTME: Keeps navigation and the body within native chrome bounds.
#![allow(clippy::unwrap_used)]
use gpui::{Bounds, TestAppContext, point, px, size};

#[gpui::test]
fn header_bounds_above_body_in_windowed_and_fullscreen(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(super::tests::header_window);
    for fullscreen in [false, true, false] {
        cx.update(|window, _| {
            if window.is_fullscreen() != fullscreen {
                window.toggle_fullscreen();
            }
            assert_eq!(window.is_fullscreen(), fullscreen);
        });
        for (width, height) in [(1200., 780.), (640., 400.), (360., 400.)] {
            cx.simulate_resize(size(px(width), px(height)));
            cx.run_until_parked();
            cx.update(|window, cx| {
                window.refresh();
                let _ = window.draw(cx);
            });
            assert_eq!(
                cx.debug_bounds("titlebar").unwrap(),
                Bounds::new(point(px(0.), px(0.)), size(px(width), px(34.)))
            );
            assert!(cx.debug_bounds("titlebar-avatar").is_none());
            // The draggable center starts after Back and Forward and
            // the room they keep.
            let leading = cx.debug_bounds("titlebar-navigation").unwrap().right()
                + px(super::navigation::NAVIGATION_MARGIN);
            assert_eq!(
                cx.debug_bounds("titlebar-center").unwrap(),
                Bounds::new(point(leading, px(0.)), size(px(width) - leading, px(34.)))
            );
            let body = cx.debug_bounds("window-body").unwrap();
            // A worktree build's banner sits at the window's foot.
            assert_eq!(body.top(), px(34.));
            assert_eq!(body.size.width, px(width));
            assert!(body.bottom() <= px(height));
        }
    }
}
