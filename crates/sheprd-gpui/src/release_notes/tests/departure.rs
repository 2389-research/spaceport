// ABOUTME: Exercises real Markdown links in active and departing note content.
// ABOUTME: Ensures static text omits link handlers without changing live links.
use super::*;
use core::prelude::v1::test;

struct Notes(bool);

impl Render for Notes {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(300.)).p(px(8.)).child(render_mode(
            "departure-notes",
            &parse("[docs](https://example.test/notes)"),
            &Theme::default(),
            &crate::config::Config::default().terminal,
            self.0,
        ))
    }
}

#[gpui::test]
fn static_markdown_links_keep_live_links_separate(cx: &mut TestAppContext) {
    for active in [false, true] {
        let (_, cx) = cx.add_window_view(|_, _| Notes(active));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let line = cx.debug_bounds("departure-notes-line-0").unwrap();
        cx.simulate_click(
            point(line.left() + px(5.), line.center().y),
            Modifiers::default(),
        );
        assert_eq!(
            cx.opened_url(),
            active.then(|| "https://example.test/notes".to_owned())
        );
    }
}
