//! The tab menu's page body: its actions, top to bottom.

use crate::chrome::Text;
use crate::rows::{self, Row};
use gpui::prelude::*;
use gpui::*;
use spaceport_settings::Theme;

/// `body` with the tab menu's action rows in it, each an item `interact` takes
/// back wired with what resting on it and pressing it do.
pub fn render_actions<E: ParentElement>(
    body: E,
    text: &Text,
    theme: &Theme,
    rows: &[Row],
    selected: Option<usize>,
    interact: impl FnMut(usize, Stateful<Div>) -> Stateful<Div>,
) -> E {
    body.children(rows::list(text, theme, rows, selected, interact))
}
