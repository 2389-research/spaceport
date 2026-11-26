//! The rows a menu page lists: the words and selectors the app prepared, and
//! the one way every page paints a row — an item, the dimmed text beside it,
//! and the mark before it.
//!
//! A row carries no message and no window: the app words every label and
//! names every selector its tests read, and hands back what resting on a row
//! and pressing it do through `interact`.

use crate::chrome::Text;
use gpui::prelude::*;
use gpui::*;
use spaceport_settings::Theme;
use spaceport_widgets::direction::Logical;

/// How large a row's mark draws, and the room a row without one keeps so its
/// labels line up with a row that has one.
pub const MARK_SIZE: f32 = 14.;

/// The mark a row draws before its label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// An icon, by its asset path.
    Icon(&'static str),
    /// The room an icon takes, for a page that keeps its labels in line.
    Room,
}

/// The dimmed text a row shows after its label.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Detail {
    /// The text's own debug selector, which the app's tests read.
    pub selector: String,
    /// What it says.
    pub text: String,
    /// The widest it may draw, for a page that caps it to keep its rows
    /// narrow.
    pub max_width: Option<f32>,
}

/// A row a page lists, as the app prepared it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    /// What the row shows.
    pub label: String,
    /// The row's debug selector, which also names its element.
    pub selector: String,
    /// The label's own debug selector, where a page's tests read one.
    pub label_selector: Option<String>,
    /// The dimmed text after the label.
    pub detail: Option<Detail>,
    /// The mark before the label.
    pub mark: Option<Mark>,
}

/// One row, painted, before the page wires what resting on it and pressing it
/// do. `text` is the chrome's font, so a row is as tall as the page's others.
pub fn line(text: &Text, theme: &Theme, row: &Row, selected: bool) -> Stateful<Div> {
    let selector = row.selector.clone();
    let line = div()
        .id(selector.clone())
        .debug_selector(move || selector)
        .role(Role::MenuItem)
        .aria_label(row.label.clone())
        .when(selected, |line| line.aria_active_descendant())
        .min_h(px(text.line_height + 12.))
        .px(px(8.))
        .row()
        .items_center()
        .gap(px(8.))
        .cursor_pointer()
        .when(selected, |line| line.bg(rgb(theme.active)))
        .hover(|line| line.bg(rgb(theme.active)));
    let line = match row.mark {
        Some(Mark::Icon(path)) => line.child(
            svg()
                .path(path)
                .size(px(MARK_SIZE))
                .flex_none()
                .text_color(rgb(theme.muted)),
        ),
        Some(Mark::Room) => line.child(div().size(px(MARK_SIZE)).flex_none()),
        None => line,
    };
    // Its own width counts, so a page widens to fit the longest label.
    let label = div()
        .min_w_0()
        .truncate()
        .flex_grow(1.)
        .child(row.label.clone());
    let line = line.child(match &row.label_selector {
        Some(selector) => {
            let selector = selector.clone();
            label.debug_selector(move || selector)
        }
        None => label,
    });
    match &row.detail {
        Some(detail) => {
            let selector = detail.selector.clone();
            line.child(
                div()
                    .debug_selector(move || selector)
                    .flex_none()
                    .when_some(detail.max_width, |detail, width| detail.max_w(px(width)))
                    .truncate()
                    .text_color(rgb(theme.muted))
                    .child(detail.text.clone()),
            )
        }
        None => line,
    }
}

/// `rows` in order, each an item `interact` takes back wired with what resting
/// on it and pressing it do, which only the window knows.
pub fn list<'a>(
    text: &'a Text,
    theme: &'a Theme,
    rows: &'a [Row],
    selected: Option<usize>,
    mut interact: impl FnMut(usize, Stateful<Div>) -> Stateful<Div> + 'a,
) -> impl Iterator<Item = Stateful<Div>> + 'a {
    rows.iter().enumerate().map(move |(index, row)| {
        let line = line(text, theme, row, Some(index) == selected);
        interact(index, line)
    })
}
