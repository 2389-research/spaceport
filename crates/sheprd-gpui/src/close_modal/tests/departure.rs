// ABOUTME: Exercises departing consent without the popup's outer input shield.
// ABOUTME: Preserves captured scroll geometry while active consent clears immediately.
use super::*;
use crate::{fonts::StyledFont, sidebar::layout_tests::fixture_window};
use gpui::{Context, Modifiers, Render, Window};

struct Preview {
    source: Entity<SheprdWindow>,
    paint: Option<Paint>,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let font = self.source.read(cx).config.ui.clone();
        div()
            .w(px(420.))
            .text_font(&font)
            .text_size(px(font.size))
            .line_height(px(font.line_height()))
            .child(self.source.update(cx, |source, cx| match &self.paint {
                Some(paint) => source.render_close_paint(paint, cx),
                None => source.render_close_confirmation(cx),
            }))
    }
}

#[gpui::test]
fn departing_controls_cannot_act_on_replacement_consent_without_a_parent_shield(
    cx: &mut TestAppContext,
) {
    let mut expected = None;
    let (preview, cx) = cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| fixture_window(window, cx));
        let paint = source.update(cx, |view, _| {
            let mut snapshot: ClientShellSnapshot = serde_json::from_str(include_str!(
                "../../../../sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json"
            ))
            .unwrap();
            let first = snapshot.panes[0].pane_id.clone();
            let mut second = snapshot.panes[0].clone();
            second.pane_id = "replacement-pane".into();
            snapshot.panes.push(second);
            view.config.confirm_close_pane = true;
            view.menu.page = Some(Page::ConfirmClose);
            view.menu.close = CloseConfirmation::capture_pane(&snapshot, &first);
            let paint = view.close_confirmation_paint().unwrap();
            let replacement =
                CloseConfirmation::capture_pane(&snapshot, "replacement-pane").unwrap();
            expected = Some(replacement.clone());
            view.menu.close = Some(replacement);
            paint
        });
        Preview {
            source,
            paint: Some(paint),
        }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("menu-leaving-shield").is_none());
    for selector in ["close-confirm", "close-cancel", "close-do-not-ask"] {
        let button = cx.debug_bounds(selector).unwrap();
        cx.simulate_click(button.center(), Modifiers::default());
        preview.read_with(cx, |preview, cx| {
            let source = preview.source.read(cx);
            assert_eq!(source.menu.page, Some(Page::ConfirmClose), "{selector}");
            assert_eq!(source.menu.close, expected, "{selector}");
            assert!(source.config.confirm_close_pane, "{selector}");
        });
    }
}

#[gpui::test]
fn long_target_scroll_survives_cancellation_without_retaining_consent(cx: &mut TestAppContext) {
    let peer = crate::window::MockPeer::advertising(&["tab.close"]);
    let (preview, cx) = cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| fixture_window(window, cx));
        source.update(cx, |view, cx| {
            peer.prepare_local_tab(view, cx);
            let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            let mut picks = Vec::new();
            for index in 0..24 {
                let mut tab = snapshot.tabs[0].clone();
                tab.tab_id = format!("other-tab-{index}");
                tab.label = format!("Synthetic terminal {index}");
                tab.focused = false;
                let mut pane = snapshot.panes[0].clone();
                pane.tab_id = tab.tab_id.clone();
                pane.pane_id = format!("other-pane-{index}");
                pane.focused = false;
                picks.push(crate::browser::Pick::Herdr(tab.tab_id.clone()));
                snapshot.tabs.push(tab);
                snapshot.panes.push(pane);
            }
            *view.endpoints[0].connection.inbox.lock().unwrap() = view.live.clone();
            view.close_tabs(picks, window, cx);
            assert_eq!(view.menu.page, Some(Page::ConfirmClose));
        });
        Preview {
            source,
            paint: None,
        }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let targets = cx.debug_bounds("close-targets").unwrap();
    let before = cx.debug_bounds("close-target-text").unwrap();
    cx.simulate_event(ScrollEvent {
        position: targets.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-100.))),
        ..Default::default()
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let scrolled = cx.debug_bounds("close-target-text").unwrap();
    assert!(
        scrolled.top() < before.top(),
        "the live target list must scroll"
    );
    cx.update(|window, cx| {
        preview.update(cx, |preview, cx| {
            preview.paint = Some(preview.source.update(cx, |view, cx| {
                let paint = view.close_confirmation_paint().unwrap();
                view.dismiss_menu(window, cx);
                assert!(view.menu.close.is_none());
                assert!(view.menu.page.is_none());
                assert!(view.browser.closing.is_none());
                assert_eq!(view.menu.close_targets_scroll.offset(), Point::default());
                paint
            }));
            cx.notify();
        });
        window.draw(cx).clear(cx);
    });
    assert_eq!(cx.debug_bounds("close-target-text"), Some(scrolled));
    assert_eq!(cx.debug_bounds("close-targets"), Some(targets));
}
