// ABOUTME: Exercises passive application-menu rows without their input shield.
// ABOUTME: Checks real scroll geometry and stale controls against a replacement menu.
use super::*;

struct Frozen {
    source: Entity<SheprdWindow>,
    paint: Option<Paint>,
}

impl Render for Frozen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let root = div().size_full();
        match &self.paint {
            Some(paint) => root.child(self.source.update(cx, |source, cx| {
                source.render_application_menu_layer(paint, false, (1., px(0.)), window, cx)
            })),
            None => root.child(self.source.clone()),
        }
    }
}

fn preview(cx: &mut TestAppContext) -> (Entity<Frozen>, &mut VisualTestContext) {
    cx.add_window_view(|window, cx| Frozen {
        source: cx.new(|cx| {
            crate::bind_keys(cx);
            let source = fixture_window(window, cx);
            window.focus(&source.focus, cx);
            source
        }),
        paint: None,
    })
}

fn open(root: &Entity<Frozen>, heading: usize, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        root.read(cx).source.clone().update(cx, |source, cx| {
            source.open_application_menu(Some(heading), window, cx)
        });
    });
    draw(cx);
}

fn freeze(root: &Entity<Frozen>, replace: bool, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        root.update(cx, |root, cx| {
            root.paint = root.source.update(cx, |source, cx| {
                let paint = source.application_menu_paint(window, cx);
                source.dismiss_menu(window, cx);
                if replace {
                    source.open_application_menu(Some(0), window, cx);
                }
                paint
            });
            cx.notify();
        });
    });
    draw(cx);
}

#[gpui::test]
fn frozen_application_rows_and_back_cannot_act_on_a_replacement_menu(cx: &mut TestAppContext) {
    for selector in ["application-item-About Spaceport", "application-menu-back"] {
        let (root, cx) = preview(cx);
        draw(cx);
        open(&root, 0, cx);
        freeze(&root, true, cx);
        let button = cx.debug_bounds(selector).unwrap();
        cx.simulate_click(button.center(), Modifiers::none());
        cx.run_until_parked();
        root.read_with(cx, |root, cx| {
            let source = root.source.read(cx);
            assert_eq!(source.menu.page, Some(Page::Application), "{selector}");
            assert_eq!(
                source.menu.application.as_ref().unwrap().path,
                vec![0],
                "{selector}"
            );
            assert_eq!(source.menu.cover.get(), crate::menu::Cover::Unknown);
        });
    }
}

#[gpui::test]
fn scrolled_application_rows_keep_their_actual_position_after_owner_reset(cx: &mut TestAppContext) {
    let (root, cx) = preview(cx);
    cx.simulate_resize(size(px(360.), px(300.)));
    draw(cx);
    open(&root, 4, cx);
    let rows = cx.debug_bounds("application-menu-rows").unwrap();
    let original = cx.debug_bounds("application-item-Reconnect").unwrap();
    cx.simulate_event(ScrollEvent {
        position: rows.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-80.))),
        ..Default::default()
    });
    draw(cx);
    let scrolled = cx.debug_bounds("application-item-Reconnect").unwrap();
    assert!(
        scrolled.top() < original.top(),
        "the active list must scroll"
    );
    let panel = cx.debug_bounds("application-menu-panel").unwrap();
    freeze(&root, false, cx);
    let frozen = cx.debug_bounds("application-item-Reconnect").unwrap();
    assert!((frozen.top() - scrolled.top()).abs() < px(1.));
    assert_eq!(cx.debug_bounds("application-menu-panel").unwrap(), panel);
    root.read_with(cx, |root, cx| {
        assert!(root.source.read(cx).menu.application.is_none());
        assert!(root.source.read(cx).menu.page.is_none());
    });
}

#[gpui::test]
fn end_reveals_the_selected_application_row_in_a_short_window(cx: &mut TestAppContext) {
    let (view, cx) = window(cx);
    cx.simulate_resize(size(px(360.), px(300.)));
    draw(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_application_menu(Some(4), window, cx)
        })
    });
    draw(cx);
    cx.simulate_keystrokes("end");
    draw(cx);
    let last = cx.debug_bounds("application-item-Reconnect").unwrap();
    assert!(
        cx.debug_bounds("application-menu-panel")
            .unwrap()
            .contains(&last.center())
    );
}

#[gpui::test]
fn frozen_application_panel_stays_inside_a_resized_window(cx: &mut TestAppContext) {
    let (root, cx) = preview(cx);
    cx.simulate_resize(size(px(1200.), px(700.)));
    draw(cx);
    open(&root, 4, cx);
    freeze(&root, false, cx);
    cx.simulate_resize(size(px(280.), px(240.)));
    draw(cx);
    let panel = cx.debug_bounds("application-menu-panel").unwrap();
    assert!(
        panel.left() >= px(0.) && panel.right() <= px(280.),
        "{panel:?}"
    );
    assert!(
        panel.top() >= px(0.) && panel.bottom() <= px(240.),
        "{panel:?}"
    );
}
