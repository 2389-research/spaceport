//! The group menu's page body: its rows, the rule that separates one run of
//! them from the next, and the heading over a run that has one.

use crate::chrome::Text;
use crate::rows::{self, Row};
use gpui::prelude::*;
use gpui::*;
use spaceport_settings::Theme;

/// A row of the group menu: what it shows, the run of rows it belongs to, and
/// the heading over that run.
#[derive(Clone, Debug)]
pub struct Entry {
    /// The row itself.
    pub row: Row,
    /// Which run of rows it belongs to; a rule separates runs.
    pub section: u8,
    /// The heading over the run it starts, if that run has one.
    pub heading: Option<String>,
}

/// Where `entries[index]` lands among the panel's children: the heading before
/// the first row of a headed run is a child of its own, so a row the keyboard
/// selects past a long list scrolls to the child the panel knows it by.
pub fn child_index(entries: &[Entry], index: usize) -> usize {
    let headings = entries
        .iter()
        .take(index + 1)
        .enumerate()
        .filter(|(at, entry)| {
            entry.heading.is_some() && (*at == 0 || entries[at - 1].section != entry.section)
        })
        .count();
    index + headings
}

/// `body` with the group menu's rows in it. `interact` takes one row's element
/// back wired with what resting on it and pressing it do, and `trailing` adds
/// what one row ends with — the control that closes an open tunnel.
pub fn render_actions<E: ParentElement>(
    body: E,
    text: &Text,
    theme: &Theme,
    entries: &[Entry],
    selected: Option<usize>,
    mut trailing: impl FnMut(usize) -> Option<AnyElement>,
    mut interact: impl FnMut(usize, Stateful<Div>) -> Stateful<Div>,
) -> E {
    let mut section = None;
    entries
        .iter()
        .enumerate()
        .fold(body, |body, (index, entry)| {
            let starts_section = section.is_some_and(|last| last != entry.section);
            let heading = entry
                .heading
                .clone()
                .filter(|_| section != Some(entry.section));
            section = Some(entry.section);
            let headed = heading.is_some();
            let body = match heading {
                Some(heading) => body.child(
                    div()
                        .flex_none()
                        .when(starts_section, |heading| {
                            heading
                                .mt(px(4.))
                                .border_t_1()
                                .border_color(rgb(theme.active))
                        })
                        .pt(px(6.))
                        .pb(px(2.))
                        .px(px(8.))
                        .text_xs()
                        .text_color(rgb(theme.muted))
                        .child(heading),
                ),
                None => body,
            };
            let row = rows::line(text, theme, &entry.row, Some(index) == selected)
                .flex_none()
                .when(starts_section && !headed, |row| {
                    row.mt(px(4.)).border_t_1().border_color(rgb(theme.active))
                })
                .children(trailing(index));
            body.child(interact(index, row))
        })
}
