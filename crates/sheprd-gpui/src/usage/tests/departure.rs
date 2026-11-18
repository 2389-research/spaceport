// ABOUTME: Exercises frozen usage links and tabs without a parent input shield.
// ABOUTME: Keeps replacement observations and input ownership unchanged after stale clicks.
use super::*;
use crate::{SheprdWindow, fonts::StyledFont, menu::Page, sidebar::layout_tests::fixture_window};
use gpui::{
    Context, Entity, Modifiers, Render, TestAppContext, Window as NativeWindow, div, prelude::*, px,
};

struct Frozen {
    source: Entity<SheprdWindow>,
    paint: super::super::Paint,
}

impl Render for Frozen {
    fn render(&mut self, _: &mut NativeWindow, cx: &mut Context<Self>) -> impl IntoElement {
        self.source.update(cx, |source, cx| {
            div()
                .w(px(340.))
                .h(px(500.))
                .text_font(&source.config.ui)
                .text_size(px(source.config.ui.size))
                .line_height(px(source.config.ui.line_height()))
                .child(source.render_usage_paint(&self.paint, cx))
        })
    }
}

#[gpui::test]
fn frozen_usage_links_and_tabs_cannot_affect_a_replacement_panel(cx: &mut TestAppContext) {
    let selected = provider("cursor");
    let (view, cx) = cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| fixture_window(window, cx));
        let paint = source.update(cx, |source, cx| {
            source.usage = super::super::Usage::showing(
                super::super::Host::Local,
                ["claude", "codex"]
                    .into_iter()
                    .map(|id| Reading {
                        provider: provider(id),
                        report: Some(report(provider(id), 25.)),
                        error: None,
                    })
                    .collect(),
            );
            source.open_usage(provider("claude"), Default::default(), window, cx);
            let paint = source.usage_paint(provider("claude"));
            source.dismiss_menu(window, cx);
            source.usage = super::super::Usage::default();
            source.open_usage(selected, Default::default(), window, cx);
            window.focus(&source.focus, cx);
            paint
        });
        Frozen { source, paint }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    for selector in ["usage-dashboard", "usage-status", "usage-tab-codex"] {
        let bounds = cx.debug_bounds(selector).unwrap();
        cx.simulate_click(bounds.center(), Modifiers::default());
        assert!(cx.opened_url().is_none(), "{selector}");
        cx.update(|window, cx| {
            let source = view.read(cx).source.read(cx);
            assert_eq!(source.menu.page, Some(Page::Usage(selected)), "{selector}");
            assert!(source.focus.is_focused(window), "{selector}");
        });
    }
}
