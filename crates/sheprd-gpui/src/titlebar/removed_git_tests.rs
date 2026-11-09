// ABOUTME: Checks the titlebar omits Git workflow controls for tracked checkouts.
// ABOUTME: Protects the neighboring account and sidebar controls at narrow widths.

use gpui::{TestAppContext, px, size};

#[gpui::test]
fn titlebar_has_no_git_controls(cx: &mut TestAppContext) {
    let (_view, cx) = cx.add_window_view(crate::titlebar::tests::header_window);
    for width in [1200., 360.] {
        cx.simulate_resize(size(px(width), px(600.)));
        cx.update(|window, cx| {
            window.refresh();
            let _ = window.draw(cx);
        });
        assert!(cx.debug_bounds("titlebar-git-slot").is_none());
        assert!(cx.debug_bounds("titlebar-git").is_none());
        assert!(cx.debug_bounds("titlebar-git-pr-link").is_none());
        assert!(cx.debug_bounds("toggle-sidebar").is_some());
        assert!(cx.debug_bounds("titlebar-avatar").is_some());
    }
}
