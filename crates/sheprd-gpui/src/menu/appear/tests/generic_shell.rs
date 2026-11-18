// ABOUTME: Preserves real generic popup shell scroll offsets across dismissal.
// ABOUTME: Keeps departing shells inert after their live consent and input state reset.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{SheprdWindow, close_modal::CloseConfirmation, sidebar::layout_tests::fixture_window};
use gpui::{
    Bounds, Context, Entity, Pixels, Render, ScrollDelta, ScrollEvent, TestAppContext,
    VisualTestContext, Window, size,
};

struct Preview {
    source: Entity<SheprdWindow>,
    page: Page,
    paint: Option<Kept>,
}

impl Render for Preview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let shell = self.source.update(cx, |source, cx| {
            if self.paint.is_none() {
                source.tick_menu_appear(window);
            }
            source.render_menu_layer(
                self.page,
                self.paint.as_ref().map(|_| 1.),
                self.paint.as_ref(),
                window,
                cx,
            )
        });
        // Only the real shell receives events; no terminal/list underneath can affect this test.
        div().relative().size_full().child(shell)
    }
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

fn fixture(page: Page, cx: &mut TestAppContext) -> (Entity<Preview>, &mut VisualTestContext) {
    let (preview, cx) = cx.add_window_view(|window, cx| Preview {
        source: cx.new(|cx| {
            let mut view = fixture_window(window, cx);
            view.open_menu(window, cx);
            view.menu.anchor = point(px(200.), px(160.));
            if page == Page::ConfirmClose {
                let snapshot = view.live.snapshot.as_ref().unwrap();
                view.menu.close =
                    CloseConfirmation::capture_tab(snapshot, &snapshot.tabs[0].tab_id);
                assert!(view.menu.close.is_some());
            }
            view.menu.page = Some(page);
            view
        }),
        page,
        paint: None,
    });
    cx.simulate_resize(size(px(600.), px(180.)));
    draw(cx);
    (preview, cx)
}

fn selector(page: Page) -> &'static str {
    match page {
        Page::Menu => "menu-settings",
        Page::About => "about-version",
        Page::ConfirmClose => "close-target-text",
        _ => panic!("unsupported generic shell fixture"),
    }
}

fn body(page: Page, cx: &mut VisualTestContext) -> Bounds<Pixels> {
    cx.debug_bounds(selector(page)).unwrap()
}

fn wheel(cx: &mut VisualTestContext) {
    let panel = cx.debug_bounds("menu-panel").unwrap();
    cx.simulate_event(ScrollEvent {
        position: panel.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-12.))),
        ..Default::default()
    });
    draw(cx);
}

fn dismiss(preview: &Entity<Preview>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        preview.update(cx, |preview, cx| {
            preview.paint = Some(preview.source.update(cx, |source, cx| {
                let origin = source.menu.origin.clone().unwrap();
                assert!(origin.current(source));
                let offset = source.menu.shell_scroll.offset().y;
                assert!(
                    offset < px(0.),
                    "the active owned shell must actually scroll"
                );
                source.dismiss_menu(window, cx);
                assert!(source.menu.page.is_none());
                assert!(source.menu.close.is_none());
                assert_eq!(source.menu.origin.as_ref(), Some(&origin));
                assert!(source.focus.is_focused(window));
                assert_eq!(source.menu.shell_scroll.offset(), Point::default());
                let (captured, paint) = source.menu.appear.kept.take().unwrap();
                assert_eq!(captured, preview.page);
                assert!(
                    paint.holds(preview.page),
                    "ordinary dismissal must capture this exact family"
                );
                assert_eq!(paint.shell_offset, Some(offset));
                paint
            }));
            cx.notify();
        });
    });
    draw(cx);
}

fn preserves_scroll(page: Page, cx: &mut TestAppContext) {
    let (preview, cx) = fixture(page, cx);
    let initial = body(page, cx);
    wheel(cx);
    let scrolled = body(page, cx);
    assert!(
        scrolled.top() < initial.top(),
        "the active {page:?} shell must actually scroll: {initial:?} -> {scrolled:?}"
    );
    let panel = cx.debug_bounds("menu-panel").unwrap();
    dismiss(&preview, cx);
    assert_eq!(
        cx.debug_bounds("menu-panel"),
        Some(panel),
        "dismissal must preserve shell bounds"
    );
    assert_eq!(
        body(page, cx),
        scrolled,
        "departing {page:?} lost its actual shell scroll offset"
    );
}

fn ignores_wheel(page: Page, cx: &mut TestAppContext) {
    let (preview, cx) = fixture(page, cx);
    let initial = body(page, cx);
    wheel(cx);
    assert!(
        body(page, cx).top() < initial.top(),
        "the active fixture must overflow"
    );
    dismiss(&preview, cx);
    // The preservation cases separately require this to equal the active scrolled position.
    let leaving = body(page, cx);
    let source = preview.read_with(cx, |preview, _| preview.source.clone());
    let snapshot = source.read_with(cx, |source, _| source.live.snapshot.clone().unwrap());
    let origin = source.read_with(cx, |source, _| source.menu.origin.clone().unwrap());
    wheel(cx);
    assert_eq!(
        body(page, cx),
        leaving,
        "wheel moved the passive {page:?} shell"
    );
    cx.update(|window, cx| {
        source.read_with(cx, |source, _| {
            assert!(source.menu.page.is_none());
            assert!(source.menu.close.is_none());
            assert_eq!(source.menu.origin.as_ref(), Some(&origin));
            assert!(source.menu.input.is_none());
            assert_eq!(source.menu.shell_scroll.offset(), Point::default());
            assert!(source.focus.is_focused(window));
            assert!(std::sync::Arc::ptr_eq(
                source.live.snapshot.as_ref().unwrap(),
                &snapshot
            ));
            assert_eq!(source.menu.cover.get(), crate::menu::Cover::Unknown);
        });
    });
}

#[gpui::test]
fn footer_shell_keeps_its_actual_scroll_position(cx: &mut TestAppContext) {
    preserves_scroll(Page::Menu, cx);
}

#[gpui::test]
fn static_dialog_shell_keeps_its_actual_scroll_position(cx: &mut TestAppContext) {
    preserves_scroll(Page::About, cx);
}

#[gpui::test]
fn confirmation_shell_keeps_its_actual_scroll_position(cx: &mut TestAppContext) {
    preserves_scroll(Page::ConfirmClose, cx);
}

#[gpui::test]
fn footer_departure_ignores_wheel(cx: &mut TestAppContext) {
    ignores_wheel(Page::Menu, cx);
}

#[gpui::test]
fn static_dialog_departure_ignores_wheel(cx: &mut TestAppContext) {
    ignores_wheel(Page::About, cx);
}

#[gpui::test]
fn confirmation_departure_ignores_wheel(cx: &mut TestAppContext) {
    ignores_wheel(Page::ConfirmClose, cx);
}
