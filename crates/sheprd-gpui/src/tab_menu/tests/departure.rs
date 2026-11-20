// ABOUTME: Exercises departing tab controls without an outer input shield.
// ABOUTME: Verifies field release and destination scroll after active targets disappear.
use super::*;
use crate::fonts::StyledFont;
use gpui::{Context, Render, Window};

struct Preview {
    source: Entity<SheprdWindow>,
    paint: Option<Paint>,
    moving: bool,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.source.update(cx, |source, cx| {
            let font = &source.config.ui;
            let panel = div()
                .id("tab-preview")
                .debug_selector(|| "tab-preview".into())
                .w(px(420.))
                .max_h(px(400.))
                .p(px(6.))
                .overflow_y_scroll()
                .text_font(font)
                .text_size(px(font.size))
                .line_height(px(font.line_height()));
            match (&self.paint, self.moving) {
                (Some(paint), true) => source.render_tab_move_paint(panel, paint, cx),
                (None, true) => source.render_tab_move_picker(panel, cx),
                (Some(paint), false) => panel
                    .overflow_hidden()
                    .child(source.render_tab_menu_paint(paint, cx)),
                (None, false) => panel.child(source.render_tab_menu(cx)),
            }
        })
    }
}

fn snapshot() -> ClientShellSnapshot {
    serde_json::from_str(include_str!(
        "../../../../sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    ))
    .unwrap()
}

fn preview(cx: &mut TestAppContext) -> (Entity<Preview>, &mut VisualTestContext) {
    cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| {
            let mut source = crate::sidebar::layout_tests::fixture_window(window, cx);
            source.live.snapshot = Some(Arc::new(snapshot()));
            source
        });
        source.update(cx, |source, cx| {
            source.open_tab_menu("w1:t1", None, Point::default(), window, cx);
        });
        Preview {
            source,
            paint: None,
            moving: false,
        }
    })
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

#[gpui::test]
fn frozen_actions_and_rename_controls_leave_replacement_draft_untouched(cx: &mut TestAppContext) {
    let (preview, cx) = preview(cx);
    let source = preview.read_with(cx, |preview, _| preview.source.clone());
    let actions = source.update(cx, |source, cx| source.tab_menu_paint(cx).unwrap());
    cx.update(|window, cx| {
        source.update(cx, |source, cx| {
            source.activate_tab_menu(Action::Rename, window, cx);
        });
    });
    draw(cx);
    let rename = source.update(cx, |source, cx| source.tab_menu_paint(cx).unwrap());
    let old = source.read_with(cx, |source, _| {
        source
            .menu
            .tab
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
            source.rename_focused_tab(window, cx);
            source
                .menu
                .tab
                .as_ref()
                .unwrap()
                .input
                .as_ref()
                .unwrap()
                .update(cx, |input, cx| {
                    input.set_text_selected("replacement draft", cx);
                });
        });
    });
    let replacement = source.read_with(cx, |source, _| {
        source
            .menu
            .tab
            .as_ref()
            .unwrap()
            .input
            .as_ref()
            .unwrap()
            .entity_id()
    });
    for (paint, selectors) in [
        (actions, &["tab-menu-1"][..]),
        (rename, &["tab-rename-cancel", "tab-rename-submit"][..]),
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
                assert_eq!(source.menu.page, Some(Page::RenameTab));
                let tab = source.menu.tab.as_ref().unwrap();
                let input = tab.input.as_ref().unwrap();
                assert_eq!(input.entity_id(), replacement);
                assert_eq!(input.read(cx).text(), "replacement draft");
                assert!(tab.pending.is_none());
                assert!(tab.error.is_none());
            });
        }
    }
    cx.update(|_, _| assert!(old.upgrade().is_none(), "paint retained the old field"));
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[gpui::test]
fn frozen_destination_cannot_start_or_error_a_replacement_move(cx: &mut TestAppContext) {
    let (preview, cx) = preview(cx);
    let source = preview.read_with(cx, |preview, _| preview.source.clone());
    cx.update(|window, cx| {
        preview.update(cx, |preview, cx| {
            preview.moving = true;
            preview.paint = Some(source.update(cx, |source, cx| {
                source.menu.page = Some(Page::MoveTab);
                let paint = source.tab_menu_paint(cx).unwrap();
                source.dismiss_menu(window, cx);
                source.open_tab_menu("w1:t1", None, Point::default(), window, cx);
                source.menu.page = Some(Page::MoveTab);
                // Any accidentally attached callback must refuse before launching a host CLI.
                Arc::make_mut(source.live.snapshot.as_mut().unwrap())
                    .boot_id
                    .push_str("-replaced");
                paint
            }));
            cx.notify();
        });
    });
    draw(cx);
    let row = cx.debug_bounds("tab-move-0").unwrap();
    cx.simulate_click(row.center(), Modifiers::default());
    source.read_with(cx, |source, _| {
        assert_eq!(source.menu.page, Some(Page::MoveTab));
        let tab = source.menu.tab.as_ref().unwrap();
        assert!(tab.error.is_none());
        assert!(tab.moving.is_none());
        assert!(tab.selected.is_none());
    });
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[gpui::test]
fn frozen_destinations_keep_scroll_after_target_and_snapshot_reset(cx: &mut TestAppContext) {
    let (preview, cx) = preview(cx);
    cx.update(|window, cx| {
        preview.update(cx, |preview, cx| {
            preview.moving = true;
            preview.source.update(cx, |source, cx| {
                source.dismiss_menu(window, cx);
                let snapshot = Arc::make_mut(source.live.snapshot.as_mut().unwrap());
                let workspace = snapshot.workspaces[0].clone();
                let tab = snapshot.tabs[0].clone();
                let pane = snapshot.panes[0].clone();
                for index in 2..66 {
                    let mut workspace = workspace.clone();
                    workspace.workspace_id = format!("w{index}");
                    workspace.active_tab_id = format!("w{index}:t1");
                    workspace.label = format!("destination {index}");
                    workspace.number = index;
                    workspace.focused = false;
                    let mut tab = tab.clone();
                    tab.workspace_id = workspace.workspace_id.clone();
                    tab.tab_id = workspace.active_tab_id.clone();
                    tab.focused = false;
                    let mut pane = pane.clone();
                    pane.workspace_id = workspace.workspace_id.clone();
                    pane.tab_id = tab.tab_id.clone();
                    pane.pane_id = format!("w{index}:p1");
                    pane.focused = false;
                    snapshot.workspaces.push(workspace);
                    snapshot.tabs.push(tab);
                    snapshot.panes.push(pane);
                }
                source.open_tab_menu("w1:t1", None, Point::default(), window, cx);
                source.menu.page = Some(Page::MoveTab);
            });
            cx.notify();
        });
    });
    draw(cx);
    let panel = cx.debug_bounds("tab-preview").unwrap();
    let before = cx.debug_bounds("tab-move-0").unwrap();
    cx.simulate_event(ScrollEvent {
        position: panel.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-100.))),
        ..Default::default()
    });
    draw(cx);
    let scrolled = cx.debug_bounds("tab-move-0").unwrap();
    assert!(scrolled.top() < before.top());
    cx.update(|window, cx| {
        preview.update(cx, |preview, cx| {
            preview.paint = Some(preview.source.update(cx, |source, cx| {
                let paint = source.tab_menu_paint(cx).unwrap();
                source.dismiss_menu(window, cx);
                assert!(source.menu.tab.is_none());
                source.live.snapshot = Some(Arc::new(snapshot()));
                paint
            }));
            cx.notify();
        });
    });
    draw(cx);
    assert_eq!(cx.debug_bounds("tab-move-0"), Some(scrolled));
    assert_eq!(cx.debug_bounds("tab-preview"), Some(panel));
}
