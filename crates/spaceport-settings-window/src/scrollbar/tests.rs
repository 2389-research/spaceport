//! A pane whose content runs past it shows a thumb that follows its scroll and
//! drags it; one that fits shows none.
use super::*;
use core::prelude::v1::test;
use spaceport_settings::Theme;

/// A pane that scrolls its own content: what the window gives `scroll_area`.
#[derive(Default)]
struct Harness {
    scroll: ScrollHandle,
    /// Where on the thumb the pointer took hold of it.
    grab: f32,
}

impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::default();
        let content = div()
            .id("harness-content")
            .debug_selector(|| "harness-content".into())
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .children((0..20).map(|row| div().flex_none().h(px(40.)).child(format!("{row}"))));
        div().size_full().flex().flex_col().child(scroll_area(
            Pane::Body,
            &theme,
            &self.scroll,
            content,
            |_, _, _, _| {},
            |_, _, _, _| {},
        ))
    }
}

/// A harness whose thumb drag really moves the pane, as the window's does.
struct Dragging(Entity<Harness>);

impl Dragging {
    fn begin(&self, cx: &mut VisualTestContext, y: Pixels) {
        let view = self.0.clone();
        view.update(cx, |harness, _| harness.grab = grab(&harness.scroll, y));
    }

    fn moved(&self, cx: &mut VisualTestContext, y: Pixels) {
        let view = self.0.clone();
        view.update(cx, |harness, _| {
            drag(&harness.scroll, harness.grab, y);
        });
    }
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

#[gpui::test]
fn overflowing_panes_show_a_thumb_that_drags_them(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| Harness::default());
    let dragging = Dragging(view.clone());

    // Tall enough for every row: the pane has no thumb.
    cx.simulate_resize(size(px(600.), px(1400.)));
    draw(cx);
    draw(cx);
    assert!(cx.debug_bounds("settings-body-thumb").is_none());

    cx.simulate_resize(size(px(600.), px(420.)));
    // The thumb reads the layout before it, so a resize shows it next frame.
    draw(cx);
    draw(cx);
    let thumb = cx.debug_bounds("settings-body-thumb").unwrap();
    let area = cx.debug_bounds("harness-content").unwrap();
    assert!(thumb.size.height < area.size.height, "{thumb:?}");
    assert!((thumb.top() - area.top()).abs() < px(1.), "{thumb:?}");
    assert!(
        thumb.right() <= area.right(),
        "{thumb:?} sits inside the pane"
    );

    // Dragging the thumb down scrolls the pane, and the thumb follows.
    let from = thumb.center();
    let to = from + point(px(0.), px(120.));
    dragging.begin(cx, from.y);
    dragging.moved(cx, from.y + px(20.));
    dragging.moved(cx, to.y);
    draw(cx);
    let scrolled = view.read_with(cx, |harness, _| harness.scroll.offset().y);
    assert!(scrolled < px(0.), "the pane scrolled: {scrolled:?}");
    let moved = cx.debug_bounds("settings-body-thumb").unwrap();
    assert!(moved.top() > thumb.top(), "{moved:?} after {thumb:?}");
}

#[gpui::test]
fn a_thumb_of_an_unscrollable_pane_drags_nowhere(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| Harness::default());
    cx.simulate_resize(size(px(600.), px(1400.)));
    draw(cx);
    draw(cx);
    let scrolled = view.read_with(cx, |harness, _| drag(&harness.scroll, 0., px(300.)));
    assert!(!scrolled);
    let grab = view.read_with(cx, |harness, _| grab(&harness.scroll, px(300.)));
    assert_eq!(grab, 0.);
}
