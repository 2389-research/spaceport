// ABOUTME: Exercises frozen group rows without the popup's outer input shield.
// ABOUTME: Keeps port scroll geometry and live tunnel ownership separate from departure paint.
use super::*;
use crate::fonts::StyledFont;
use gpui::{Context, Render, Window};

struct Preview {
    source: Entity<SheprdWindow>,
    paint: Option<Paint>,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.source.update(cx, |source, cx| {
            let font = &source.config.ui;
            let panel = div()
                .id("group-preview")
                .debug_selector(|| "group-preview".into())
                .w(px(420.))
                .max_h(px(400.))
                .p(px(6.))
                .overflow_y_scroll()
                .text_font(font)
                .text_size(px(font.size))
                .line_height(px(font.line_height()));
            match &self.paint {
                Some(paint) => source.render_group_menu_paint(panel, paint, cx),
                None => source.render_group_menu(panel, cx),
            }
        })
    }
}

fn preview(cx: &mut TestAppContext) -> (Entity<Preview>, &mut VisualTestContext) {
    cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| fixture_window(window, cx));
        source.update(cx, |source, cx| {
            let snapshot = Arc::make_mut(source.live.snapshot.as_mut().unwrap());
            snapshot.focused_workspace_id = Some("w0".into());
            snapshot.focused_tab_id = Some("t0".into());
            source.open_group_menu(source.group_slots()[0].id, Point::default(), window, cx);
        });
        Preview {
            source,
            paint: None,
        }
    })
}

#[gpui::test]
fn frozen_split_cannot_dismiss_or_change_a_replacement_group_without_a_parent_shield(
    cx: &mut TestAppContext,
) {
    let (preview, cx) = preview(cx);
    preview.update(cx, |preview, cx| {
        preview.paint = Some(preview.source.update(cx, |source, cx| {
            let paint = source.group_menu_paint(cx).unwrap();
            source.menu.group.as_mut().unwrap().selected = Some(0);
            paint
        }));
    });
    draw(cx);
    assert!(cx.debug_bounds("menu-leaving-shield").is_none());
    let split = cx.debug_bounds("group-menu-Split").unwrap();
    let before = preview.read_with(cx, |preview, cx| preview.source.read(cx).group_slots());
    cx.simulate_click(split.center(), Modifiers::default());
    preview.read_with(cx, |preview, cx| {
        let source = preview.source.read(cx);
        assert_eq!(source.menu.page, Some(Page::Group));
        assert_eq!(source.menu.group.as_ref().unwrap().selected, Some(0));
        assert_eq!(source.group_slots(), before);
    });
}

#[gpui::test]
fn frozen_group_preserves_actual_port_scroll_after_close_and_new_results(cx: &mut TestAppContext) {
    let (preview, cx) = preview(cx);
    preview.update(cx, |preview, cx| {
        preview.source.update(cx, |source, _| {
            let mut listing = (0..64)
                .map(|index| format!("L 1 127.0.0.1:{} node\n", 41400 + index))
                .collect::<String>();
            listing.push_str("E 1 w0\n");
            source.listening_ports.seed(
                crate::usage::Host::Local,
                crate::listening_ports::parse(&listing).unwrap(),
            );
        });
    });
    draw(cx);
    let panel = cx.debug_bounds("group-preview").unwrap();
    let before = cx.debug_bounds("group-menu-Port41400").unwrap();
    cx.simulate_event(ScrollEvent {
        position: panel.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-100.))),
        ..Default::default()
    });
    draw(cx);
    let scrolled = cx.debug_bounds("group-menu-Port41400").unwrap();
    assert!(scrolled.top() < before.top());
    cx.update(|window, cx| {
        preview.update(cx, |preview, cx| {
            preview.paint = Some(preview.source.update(cx, |source, cx| {
                let paint = source.group_menu_paint(cx).unwrap();
                source.dismiss_menu(window, cx);
                assert!(source.menu.group.is_none());
                source.listening_ports.seed(
                    crate::usage::Host::Local,
                    crate::listening_ports::parse("L 1 127.0.0.1:41500 replacement\nE 1 w0\n")
                        .unwrap(),
                );
                paint
            }));
            cx.notify();
        });
    });
    draw(cx);
    assert_eq!(cx.debug_bounds("group-menu-Port41400"), Some(scrolled));
    assert_eq!(cx.debug_bounds("group-preview"), Some(panel));
    assert!(cx.debug_bounds("group-menu-Port41500").is_none());
}

#[cfg(unix)]
#[gpui::test]
fn frozen_close_glyph_cannot_stop_an_owned_live_tunnel(cx: &mut TestAppContext) {
    let (preview, cx) = preview(cx);
    let source = preview.read_with(cx, |preview, _| preview.source.clone());
    let group = tunnels::remote_menu(&source, cx);
    let key = tunnels::tunnel_key(&source, cx, group, 5173);
    let child = std::process::Command::new("sleep")
        .arg("30")
        .spawn()
        .unwrap();
    source.update(cx, |source, _| {
        source.tunnels.insert(
            key.clone(),
            crate::listening_ports::Tunnel::around(child, 41010),
        )
    });
    preview.update(cx, |preview, cx| {
        preview.paint = Some(source.update(cx, |source, cx| source.group_menu_paint(cx).unwrap()));
    });
    draw(cx);
    assert!(cx.debug_bounds("menu-leaving-shield").is_none());
    let close = cx.debug_bounds("group-menu-close-Port5173").unwrap();
    cx.simulate_click(close.center(), Modifiers::default());
    source.update(cx, |source, _| {
        assert!(source.tunnels.is_open(&key));
        assert_eq!(source.menu.page, Some(Page::Group));
        assert!(
            source.tunnels.close(&key),
            "reap only the exact owned child"
        );
    });
}
