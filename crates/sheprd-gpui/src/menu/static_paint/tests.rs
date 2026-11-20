// ABOUTME: Exercises frozen dialog controls without the outer departure input shield.
// ABOUTME: Prevents a stale button from acting on replacement consent or opening links.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::sidebar::layout_tests::fixture_window;
use gpui::{Context, Entity, Modifiers, Render, TestAppContext, Window, div, prelude::*};

struct Preview {
    source: Entity<SheprdWindow>,
    paint: StaticPaint,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.source.update(cx, |view, cx| {
            view.render_static_dialog_paint(&self.paint, cx)
        }))
    }
}

#[gpui::test]
fn departing_about_links_do_not_open_without_a_parent_input_shield(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|window, cx| Preview {
        source: cx.new(|cx| fixture_window(window, cx)),
        paint: StaticPaint::About,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let link = cx.debug_bounds("about-repository").unwrap();
    cx.simulate_click(link.center(), Modifiers::default());
    assert!(cx.opened_url().is_none());
}

#[gpui::test]
fn departing_link_confirmation_cannot_act_on_a_replacement_question(cx: &mut TestAppContext) {
    let old = ExternalUrl::try_from("spaceport-fixture:old").unwrap();
    let replacement = ExternalUrl::try_from("spaceport-fixture:replacement").unwrap();
    let (preview, cx) = cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| fixture_window(window, cx));
        source.update(cx, |view, _| {
            view.menu.page = Some(Page::OpenLink);
            view.menu.open_link = Some(replacement.clone());
        });
        Preview {
            source,
            paint: StaticPaint::OpenLink(old),
        }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let yes = cx.debug_bounds("open-link-yes").unwrap();
    cx.simulate_click(yes.center(), Modifiers::default());
    assert!(cx.opened_url().is_none());
    preview.read_with(cx, |preview, cx| {
        let source = preview.source.read(cx);
        assert_eq!(source.menu.page, Some(Page::OpenLink));
        assert_eq!(source.menu.open_link.as_ref(), Some(&replacement));
    });
}

#[gpui::test]
fn departing_skill_and_version_buttons_do_not_dismiss_a_replacement_page(cx: &mut TestAppContext) {
    for (paint, selector) in [
        (StaticPaint::AgentSkill, "agent-skill-decline"),
        (
            StaticPaint::VersionMismatch(VersionNotice {
                host: None,
                mismatch: sheprd_client::VersionMismatch::DaemonOutdated {
                    server_version: None,
                },
            }),
            "version-mismatch-dismiss",
        ),
    ] {
        let (preview, cx) = cx.add_window_view(|window, cx| {
            let source = cx.new(|cx| fixture_window(window, cx));
            source.update(cx, |view, _| view.menu.page = Some(Page::Messages));
            Preview { source, paint }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let button = cx.debug_bounds(selector).unwrap();
        cx.simulate_click(button.center(), Modifiers::default());
        preview.read_with(cx, |preview, cx| {
            assert_eq!(preview.source.read(cx).menu.page, Some(Page::Messages));
        });
    }
}
