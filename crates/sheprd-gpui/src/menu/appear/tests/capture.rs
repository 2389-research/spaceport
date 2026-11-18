// ABOUTME: Refuses to regenerate departure content when its family was not captured.
// ABOUTME: Exercises the real shared renderer independently of the outer input shield.
use super::*;
use crate::{SheprdWindow, sidebar::layout_tests::fixture_window};
use gpui::{Context, Entity, Render, TestAppContext};

struct Preview {
    source: Entity<SheprdWindow>,
    captured: Option<Kept>,
}

impl Render for Preview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.source.update(cx, |source, cx| {
            source.render_menu_layer(
                Page::Menu,
                self.captured.as_ref().map(|_| 1.),
                self.captured.as_ref(),
                window,
                cx,
            )
        })
    }
}

#[gpui::test]
fn a_missing_capture_never_regenerates_content_from_the_active_renderer(cx: &mut TestAppContext) {
    let (preview, cx) = cx.add_window_view(|window, cx| Preview {
        source: cx.new(|cx| {
            let mut source = fixture_window(window, cx);
            source.open_menu(window, cx);
            source
        }),
        captured: None,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(
        cx.debug_bounds("menu-settings").is_some(),
        "active content must be available"
    );
    preview.update(cx, |preview, cx| {
        preview.captured = Some(Kept::default());
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(
        cx.debug_bounds("menu-settings").is_none(),
        "departure rebuilt uncaptured active content"
    );
}
