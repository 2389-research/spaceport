//! A window's section list and footer, drawn by a fixture that answers them
//! the way the app does.
use super::*;
use crate::scrollbar::{self, Pane};
use crate::tests::navigation_labels;
use core::prelude::v1::test;

mod groups;
mod scroll;

/// What the footer is told the window is doing.
#[derive(Default)]
struct Harness {
    drafts: Drafts,
    error: Option<String>,
    /// Where on the sections' thumb the pointer took hold of it.
    grab: f32,
    message: Option<String>,
    scroll: ScrollHandle,
    selected: Section,
}

impl Harness {
    fn new() -> Self {
        Self {
            selected: Section::Theme,
            ..Default::default()
        }
    }

    fn drawn(&self) -> Status<'_> {
        Status {
            drafts: self.drafts,
            error: self.error.as_deref(),
            loading: false,
            message: self.message.as_deref(),
        }
    }
}

impl Render for Harness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::default();
        let labels = navigation_labels();
        let view = cx.entity().downgrade();
        let select = view.clone();
        let column = sections(
            &theme,
            labels,
            self.selected,
            &self.scroll,
            |_| None,
            move |section, _, cx| {
                let _ = select.update(cx, |harness, cx| {
                    harness.selected = section;
                    cx.notify();
                });
            },
        );
        let grabbing = view.clone();
        let dragging = view;
        let area = scrollbar::scroll_area(
            Pane::Sections,
            &theme,
            &self.scroll,
            column,
            move |_, y, _, cx| {
                let _ = grabbing.update(cx, |harness, _| {
                    harness.grab = scrollbar::grab(&harness.scroll, y);
                });
            },
            move |_, y, _, cx| {
                let _ = dragging.update(cx, |harness, cx| {
                    if scrollbar::drag(&harness.scroll, harness.grab, y) {
                        cx.notify();
                    }
                });
            },
        );
        let nav = panel(
            &theme,
            labels,
            px(184.),
            div().into_any_element(),
            div().into_any_element(),
            area.into_any_element(),
        );
        let status = self.drawn();
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(div().flex_1().min_h_0().row().child(nav))
            .child(footer(
                &theme,
                &status,
                false,
                labels,
                |_, _, _| {},
                |_, _, _| {},
            ))
    }
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

/// Debug selectors are `&'static str`; a handful of test names may leak.
fn leak(name: String) -> &'static str {
    Box::leak(name.into_boxed_str())
}

fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("missing {selector}"))
}
