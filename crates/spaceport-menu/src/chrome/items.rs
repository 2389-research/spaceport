//! The in-window app menu's own rows: each line the window runs, painted over
//! the panel a footer popover hangs in.

use super::Text;
use gpui::prelude::*;
use gpui::*;
use spaceport_settings::Theme;
use spaceport_widgets::direction::Logical;

/// A row of the in-window app menu (`Page::Menu`), as the app words it.
pub struct Row {
    /// What the row shows.
    pub label: String,
    /// The row's element id, and the tail of its `menu-{selector}` debug
    /// selector.
    pub selector: &'static str,
}

/// The app menu's rows, each an item the window runs, in the order the app
/// offers them. `interact` takes one row's element back wired with what
/// resting on it and pressing it do, which only the window knows.
pub fn rows<'a>(
    text: &'a Text,
    theme: &'a Theme,
    items: &'a [Row],
    selected: Option<usize>,
    mut interact: impl FnMut(usize, Stateful<Div>) -> Stateful<Div> + 'a,
) -> impl Iterator<Item = Stateful<Div>> + 'a {
    items.iter().enumerate().map(move |(index, item)| {
        let active = Some(index) == selected;
        let selector = item.selector;
        let row = div()
            .id(selector)
            .debug_selector(move || format!("menu-{selector}"))
            .role(Role::MenuItem)
            .aria_label(item.label.clone())
            .when(active, |row| row.aria_active_descendant())
            .min_h(px(text.line_height + 12.))
            .px(px(8.))
            .row()
            .items_center()
            .cursor_pointer()
            .when(active, |row| row.bg(rgb(theme.active)))
            .child(item.label.clone());
        interact(index, row)
    })
}
