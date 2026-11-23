//! Daemon release notes and product announcements, drawn as prepared lines.
//!
//! The text arrives from the daemon as Markdown-flavored, untrusted input;
//! [`spaceport_release_notes`] bounds it and says what each line is, and
//! whatever it left out is said in a note. Links become clickable ranges that
//! open in the browser on click, and only `http`/`https` targets qualify.
//! Commands shown in code spans or blocks are never run.
// i18n: migrated
use crate::{
    config::{FontConfig, Theme},
    fonts::StyledFont,
};
use gpui::{prelude::*, *};
use spaceport_release_notes::Mark;
// The model at the paths the rest of the app reads it at.
pub(crate) use spaceport_release_notes::{
    Kind, Limits, Line, MAX_URL, bare_url, note, parse, parse_within,
};
use std::{cell::RefCell, ops::Range, rc::Rc};

/// The lines of the last body drawn, so a repaint draws from prepared lines
/// instead of parsing the body again.
#[derive(Default)]
pub(crate) struct Prepared(RefCell<Option<(String, Rc<[Line]>)>>);

impl Prepared {
    pub(crate) fn lines(&self, body: &str) -> Rc<[Line]> {
        let mut cached = self.0.borrow_mut();
        if let Some((drawn, lines)) = cached.as_ref()
            && drawn == body
        {
            return lines.clone();
        }
        let lines: Rc<[Line]> = parse(body).into();
        *cached = Some((body.to_owned(), lines.clone()));
        lines
    }
}

/// Prepared daemon text for the announcement card and the release notes sheet.
#[derive(Default)]
pub(crate) struct DaemonText {
    pub announcement: Prepared,
    pub release_notes: Prepared,
}

/// Draws prepared lines. Each link opens in the browser only when clicked.
pub(crate) fn render(id: &'static str, lines: &[Line], theme: &Theme, mono: &FontConfig) -> Div {
    div().flex().flex_col().gap(px(2.)).min_w_0().children(
        lines
            .iter()
            .enumerate()
            .map(|(index, line)| self::line(id, index, line, theme, mono)),
    )
}

/// Draws one prepared line, for views that place each line themselves.
pub(crate) fn line(
    id: &'static str,
    index: usize,
    line: &Line,
    theme: &Theme,
    mono: &FontConfig,
) -> Div {
    let accent = theme.ink(theme.palette[4]);
    let content = match line.kind {
        Kind::Heading | Kind::Code | Kind::Table => line.text.clone().into_any_element(),
        _ => rich(id, index, line, theme, accent),
    };
    line_with(id, index, line, theme, mono, content)
}

/// The same line layout with caller-owned text interaction, such as transcript
/// selection. Parsing, indentation and block styling remain shared.
pub(crate) fn line_with(
    id: &'static str,
    index: usize,
    line: &Line,
    theme: &Theme,
    mono: &FontConfig,
    content: AnyElement,
) -> Div {
    let accent = theme.ink(theme.palette[4]);
    let row = div()
        .debug_selector(move || format!("{id}-line-{index}"))
        .min_w_0()
        .pl(px(f32::from(line.indent) * 14.));
    let item = |row: Div, marker: String, content: AnyElement| {
        row.flex()
            .gap(px(6.))
            .child(div().flex_none().text_color(rgb(accent)).child(marker))
            .child(div().flex_1().min_w_0().child(content))
    };
    match line.kind {
        Kind::Blank => row.h(px(6.)),
        Kind::Heading => row
            .pt(px(4.))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(rgb(theme.primary()))
            .child(content),
        Kind::Code => row
            .px(px(8.))
            .bg(rgb(theme.active))
            .text_font(mono)
            .whitespace_nowrap()
            .overflow_hidden()
            .child(content),
        Kind::Bullet => item(row, "\u{2022}".to_owned(), content),
        Kind::Numbered(number) => item(row, format!("{number}."), content),
        Kind::Quote => row
            .pl(px(8.))
            .border_l_2()
            .border_color(rgb(theme.active))
            .text_color(rgb(theme.subtext()))
            .child(content),
        Kind::Rule => row.py(px(4.)).child(div().h(px(1.)).bg(rgb(theme.active))),
        Kind::Table => row
            .px(px(8.))
            .text_font(mono)
            .whitespace_nowrap()
            .overflow_hidden()
            .child(content),
        Kind::Paragraph => row.child(content),
    }
}

fn rich(id: &'static str, index: usize, line: &Line, theme: &Theme, accent: u32) -> AnyElement {
    let text =
        StyledText::new(line.text.clone()).with_highlights(line_highlights(line, theme, accent));
    if line.links.is_empty() {
        return text.into_any_element();
    }
    let urls: Vec<String> = line.links.iter().map(|item| item.url.clone()).collect();
    InteractiveText::new((id, index), text)
        .on_click(
            line.links.iter().map(|item| item.range.clone()).collect(),
            move |clicked, _, cx| {
                if let Some(url) = urls.get(clicked) {
                    cx.open_url(url);
                }
            },
        )
        .into_any_element()
}

pub(crate) fn line_highlights(
    line: &Line,
    theme: &Theme,
    accent: u32,
) -> Vec<(Range<usize>, HighlightStyle)> {
    let code = HighlightStyle {
        color: Some(rgb(theme.foreground).into()),
        background_color: Some(rgb(theme.active).into()),
        ..Default::default()
    };
    let bold = HighlightStyle {
        font_weight: Some(FontWeight::SEMIBOLD),
        ..Default::default()
    };
    let italic = HighlightStyle {
        font_style: Some(FontStyle::Italic),
        ..Default::default()
    };
    let link = HighlightStyle {
        color: Some(rgb(accent).into()),
        underline: Some(UnderlineStyle {
            thickness: px(1.),
            ..Default::default()
        }),
        ..Default::default()
    };
    line.highlights()
        .map(|(range, mark)| {
            let style = match mark {
                Mark::Code => code,
                Mark::Bold => bold,
                Mark::Italic => italic,
                Mark::Link => link,
            };
            (range, style)
        })
        .collect()
}

#[cfg(test)]
mod tests;
