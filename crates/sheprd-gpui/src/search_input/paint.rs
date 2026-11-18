// ABOUTME: Preserves only a search field's visible text for popup departure.
// ABOUTME: Shares field styling without retaining focus, input, or subscriptions.
use super::*;
use gpui::{Div, Stateful};

#[derive(Clone)]
pub(crate) struct Paint {
    text: SharedString,
    placeholder: bool,
    font: FontConfig,
    theme: Theme,
    framed: bool,
    scroll: Pixels,
}

impl SearchInput {
    pub(crate) fn paint(&self) -> Paint {
        let placeholder = self.text().is_empty();
        Paint {
            text: if placeholder {
                self.placeholder.clone().into()
            } else if self.masked {
                "*".repeat(self.text().chars().count()).into()
            } else {
                self.text().to_owned().into()
            },
            placeholder,
            font: self.font.clone(),
            theme: self.theme.clone(),
            framed: self.framed,
            scroll: self.scroll,
        }
    }
}

pub(super) fn frame(
    field: Stateful<Div>,
    font: &FontConfig,
    theme: &Theme,
    framed: bool,
) -> Stateful<Div> {
    field
        .w_full()
        .when(framed, |field| {
            field
                .px_2()
                .py_1()
                .rounded(px(crate::config::corners::CONTROL))
                .border_1()
                .border_color(rgb(theme.active))
                .bg(rgb(theme.background))
        })
        .text_color(rgb(theme.foreground))
        .text_font(font)
        .text_size(px(font.size))
        .line_height(px(font.line_height()))
}

impl Paint {
    pub(crate) fn render(&self) -> Stateful<Div> {
        let text = self.text.clone();
        let color = rgb(if self.placeholder {
            self.theme.muted
        } else {
            self.theme.foreground
        });
        let font_size = px(self.font.size);
        let height = px(self.font.line_height());
        let scroll = self.scroll;
        frame(
            div().id("leaving-search"),
            &self.font,
            &self.theme,
            self.framed,
        )
        .child(
            canvas(
                move |_, window, _| {
                    let run = TextRun {
                        len: text.len(),
                        font: window.text_style().font(),
                        color: color.into(),
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    };
                    window
                        .text_system()
                        .shape_line(text.clone(), font_size, &[run], None)
                },
                move |bounds, line, window, cx| {
                    window.with_content_mask(Some(ContentMask { bounds }), |window| {
                        let _ = line.paint(
                            point(bounds.left() - scroll, bounds.top()),
                            height,
                            TextAlign::Left,
                            None,
                            window,
                            cx,
                        );
                    });
                },
            )
            .w_full()
            .h(height),
        )
    }
}
