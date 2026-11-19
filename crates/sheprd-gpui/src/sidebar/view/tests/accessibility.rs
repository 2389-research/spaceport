// ABOUTME: Exercises sidebar rendering with active accessibility and ordinary caching.
// ABOUTME: Keeps real cached views across frames without invalidating their contents.
use super::*;
use gpui::{AppContext, Entity};

struct Fixture {
    accessibility_active: bool,
    window: Entity<SheprdWindow>,
}

impl Render for Fixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        element(
            &self.window.read(cx).sidebar_view,
            240.,
            self.accessibility_active,
        )
    }
}

#[gpui::test]
fn active_accessibility_rebuilds_the_sidebar_on_every_frame(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| Fixture {
        accessibility_active: false,
        window: cx.new(|cx| fixture_window(window, cx)),
    });
    cx.update(|window, cx| {
        let parent = fixture.read(cx).window.clone();
        let sidebar = parent.read(cx).sidebar_view.clone();
        window.draw(cx).clear(cx);
        let cached_renders = sidebar.read(cx).renders;
        assert!(cached_renders > 0);
        window.draw(cx).clear(cx);
        assert_eq!(sidebar.read(cx).renders, cached_renders);

        fixture.update(cx, |fixture, cx| {
            fixture.accessibility_active = true;
            cx.notify();
        });
        window.draw(cx).clear(cx);
        let active_renders = sidebar.read(cx).renders;
        window.draw(cx).clear(cx);
        assert!(
            sidebar.read(cx).renders > active_renders,
            "each accessible frame must rebuild the sidebar's nodes"
        );
    });
}
