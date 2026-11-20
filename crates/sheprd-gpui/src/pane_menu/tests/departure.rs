// ABOUTME: Opens each pane popup family from a real captured pane target.
// ABOUTME: Uses prepared process rows without starting external monitors or actions.
use super::*;
use crate::fonts::StyledFont;
use gpui::{Render, VisualTestContext};

pub(in crate::pane_menu) struct Preview {
    pub(in crate::pane_menu) source: Entity<SheprdWindow>,
    pub(in crate::pane_menu) paint: Option<Paint>,
    pub(in crate::pane_menu) moving: bool,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.source.update(cx, |source, cx| {
            let font = &source.config.ui;
            let panel = div()
                .id("pane-preview")
                .debug_selector(|| "pane-preview".into())
                .w(px(560.))
                .max_h(px(450.))
                .p(px(6.))
                .overflow_y_scroll()
                .text_font(font)
                .text_size(px(font.size))
                .line_height(px(font.line_height()));
            match (&self.paint, self.moving) {
                (Some(paint), true) => source.render_pane_move_paint(panel, paint, cx),
                (None, true) => source.render_move_picker(panel, cx),
                (Some(paint), false) => panel
                    .overflow_hidden()
                    .child(source.render_pane_menu_paint(paint, cx)),
                (None, false) => panel.child(source.render_pane_menu(cx)),
            }
        })
    }
}

pub(in crate::pane_menu) fn preview(
    cx: &mut TestAppContext,
    page: Page,
) -> (Entity<Preview>, &mut VisualTestContext) {
    cx.add_window_view(|window, cx| Preview {
        source: cx.new(|cx| fixture(page, window, cx)),
        paint: None,
        moving: page == Page::MovePane,
    })
}

pub(in crate::pane_menu) fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

pub(crate) fn fixture(
    page: Page,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) -> SheprdWindow {
    let mut source = crate::sidebar::layout_tests::fixture_window(window, cx);
    source.live.snapshot = Some(Arc::new(snapshot()));
    source.endpoints[0].connection.target = sheprd_client::ConnectTarget::Local;
    source.open_pane_menu("inactive", point(px(500.), px(200.)), window, cx);
    match page {
        Page::RenamePane => source.activate_pane_menu(Action::Rename, window, cx),
        Page::MovePane => source.open_move_picker(cx),
        Page::PaneProcesses | Page::KillProcesses => {
            source.menu.pane.as_mut().unwrap().processes = Some(processes::departure_fixture());
            source.menu.page = Some(page);
        }
        Page::Pane => {}
        _ => panic!("unsupported pane fixture page"),
    }
    source
}

#[gpui::test]
fn frozen_pane_actions_and_rename_controls_preserve_a_replacement_draft(cx: &mut TestAppContext) {
    let (preview, cx) = preview(cx, Page::Pane);
    let source = preview.read_with(cx, |preview, _| preview.source.clone());
    let actions = source.update(cx, |source, cx| source.pane_menu_paint(cx).unwrap());
    cx.update(|window, cx| {
        source.update(cx, |source, cx| {
            source.activate_pane_menu(Action::Rename, window, cx)
        })
    });
    draw(cx);
    let rename = source.update(cx, |source, cx| source.pane_menu_paint(cx).unwrap());
    let old = source.read_with(cx, |source, _| {
        source
            .menu
            .pane
            .as_ref()
            .unwrap()
            .input
            .as_ref()
            .unwrap()
            .downgrade()
    });
    cx.update(|window, cx| {
        source.update(cx, |source, cx| {
            source.dismiss_menu(window, cx);
            source.open_pane_menu("w1:p1", Point::default(), window, cx);
            source.activate_pane_menu(Action::Rename, window, cx);
            source
                .menu
                .pane
                .as_ref()
                .unwrap()
                .input
                .as_ref()
                .unwrap()
                .update(cx, |input, cx| {
                    input.set_text_selected("replacement pane", cx)
                });
        })
    });
    let replacement = source.read_with(cx, |source, _| {
        source
            .menu
            .pane
            .as_ref()
            .unwrap()
            .input
            .as_ref()
            .unwrap()
            .entity_id()
    });
    for (paint, selectors) in [
        (actions, &["pane-menu-0"][..]),
        (rename, &["pane-rename-cancel", "pane-rename-submit"][..]),
    ] {
        preview.update(cx, |preview, cx| {
            preview.paint = Some(paint);
            cx.notify();
        });
        draw(cx);
        assert!(cx.debug_bounds("menu-leaving-shield").is_none());
        for selector in selectors {
            let bounds = cx.debug_bounds(selector).unwrap();
            cx.simulate_click(bounds.center(), Modifiers::default());
            source.read_with(cx, |source, cx| {
                assert_eq!(source.menu.page, Some(Page::RenamePane));
                let pane = source.menu.pane.as_ref().unwrap();
                assert_eq!(pane.target.pane, "w1:p1");
                let input = pane.input.as_ref().unwrap();
                assert_eq!(input.entity_id(), replacement);
                assert_eq!(input.read(cx).text(), "replacement pane");
                assert!(pane.pending.is_none());
                assert!(pane.error.is_none());
            });
        }
    }
    assert!(
        old.upgrade().is_none(),
        "departure paint retained the old field"
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[gpui::test]
fn frozen_pane_destination_cannot_start_or_error_a_replacement_move(cx: &mut TestAppContext) {
    let (preview, cx) = preview(cx, Page::MovePane);
    cx.update(|window, cx| {
        preview.update(cx, |preview, cx| {
            preview.paint = Some(preview.source.update(cx, |source, cx| {
                let paint = source.pane_menu_paint(cx).unwrap();
                source.dismiss_menu(window, cx);
                source.open_pane_menu("w1:p1", Point::default(), window, cx);
                source.open_move_picker(cx);
                // A wrongly attached callback must refuse before launching a host CLI.
                Arc::make_mut(source.live.snapshot.as_mut().unwrap())
                    .boot_id
                    .push_str("-replaced");
                paint
            }));
            cx.notify();
        })
    });
    draw(cx);
    let row = cx.debug_bounds("pane-move-0").unwrap();
    cx.simulate_click(row.center(), Modifiers::default());
    preview.read_with(cx, |preview, cx| {
        let source = preview.source.read(cx);
        assert_eq!(source.menu.page, Some(Page::MovePane));
        let pane = source.menu.pane.as_ref().unwrap();
        assert_eq!(pane.target.pane, "w1:p1");
        assert!(pane.error.is_none());
        assert!(pane.moving.is_none());
        assert!(pane.selected.is_none());
    });
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[gpui::test]
fn frozen_pane_destinations_keep_scroll_after_snapshot_and_target_reset(cx: &mut TestAppContext) {
    let (preview, cx) = preview(cx, Page::MovePane);
    cx.update(|window, cx| {
        preview.update(cx, |preview, cx| {
            preview.source.update(cx, |source, cx| {
                source.dismiss_menu(window, cx);
                let snapshot = Arc::make_mut(source.live.snapshot.as_mut().unwrap());
                let tab = snapshot.tabs[0].clone();
                for number in 2..66 {
                    let mut tab = tab.clone();
                    tab.tab_id = format!("w1:t{number}");
                    tab.number = number;
                    tab.label = format!("destination {number}");
                    tab.focused = false;
                    snapshot.tabs.push(tab);
                }
                source.open_pane_menu("inactive", Point::default(), window, cx);
                source.open_move_picker(cx);
            });
            cx.notify();
        });
    });
    draw(cx);
    let panel = cx.debug_bounds("pane-preview").unwrap();
    let before = cx.debug_bounds("pane-move-0").unwrap();
    cx.simulate_event(ScrollEvent {
        position: panel.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-100.))),
        ..Default::default()
    });
    draw(cx);
    let scrolled = cx.debug_bounds("pane-move-0").unwrap();
    assert!(scrolled.top() < before.top());
    cx.update(|window, cx| {
        preview.update(cx, |preview, cx| {
            preview.paint = Some(preview.source.update(cx, |source, cx| {
                let paint = source.pane_menu_paint(cx).unwrap();
                source.dismiss_menu(window, cx);
                assert!(source.menu.pane.is_none());
                source.live.snapshot = None;
                paint
            }));
            cx.notify();
        });
    });
    draw(cx);
    assert_eq!(cx.debug_bounds("pane-move-0"), Some(scrolled));
    assert_eq!(cx.debug_bounds("pane-preview"), Some(panel));
}
