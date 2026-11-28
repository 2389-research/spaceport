//! The Keyboard Shortcuts page's body: its heading, the field that narrows it,
//! its sections and the rows they list, and the note at its foot.

use super::{Row, Section};
use crate::chrome::Text;
use gpui::prelude::*;
use gpui::*;
use spaceport_settings::{Theme, corners};

/// Every word the page says itself, which the app words (`docs/i18n.md`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Words {
    /// The button that closes the page.
    pub close: String,
    /// How many rows the query kept, of how many the page lists.
    pub count: String,
    /// The button that edits a row's shortcut.
    pub edit: String,
    /// What the page says where no row matched.
    pub empty: String,
    /// What the page says at its foot, under everything.
    pub footer: String,
    /// What a row with no binding says in its place.
    pub no_shortcut: String,
    /// What the rows say together, after the last of them.
    pub note: String,
    /// What the page is for, under its title.
    pub subtitle: String,
    /// The page's title.
    pub title: String,
}

/// What the window offers the page for one call: the sections it prepared and
/// narrowed, the controls only it can build, and what it paints with.
///
/// Taken by value, because the controls it holds are elements it consumes.
pub struct Hosts<'a> {
    /// The colour the page's heading rule, its section headings and its Edit
    /// buttons draw in, which is the app's colour policy.
    pub accent: Rgba,
    /// The open shortcut editor, which sits at the top of the scrolling body.
    pub editor: Option<AnyElement>,
    /// How far the body has scrolled, which the window keeps so a narrowed
    /// page starts at its top again.
    pub scroll: &'a ScrollHandle,
    /// The field that narrows the page.
    pub search: Option<AnyElement>,
    /// The rows the page lists, under their headings and already narrowed to
    /// what [`super::filter`] left.
    pub sections: &'a [Section],
    /// The font the page is set in.
    pub text: &'a Text,
    /// The colours the page paints with.
    pub theme: &'a Theme,
    /// The page's own words.
    pub words: &'a Words,
}

/// What pressing a control on the page asks the window to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Close the page.
    Close,
    /// Edit the shortcut of the command with this catalog name.
    Edit(&'static str),
}

/// The page, top to bottom. `interact` takes a control back wired with what
/// pressing it does, which only the window knows: the button it is given is
/// the page's own, so the window adds the press and nothing else.
pub fn panel(
    mut hosts: Hosts<'_>,
    mut interact: impl FnMut(Action, Stateful<Div>) -> Stateful<Div>,
) -> Div {
    let editor = hosts.editor.take();
    let search = hosts.search.take();
    let body = div()
        .id("keybinds-body")
        .debug_selector(|| "keybinds-body".into())
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .track_scroll(hosts.scroll)
        .px(px(16.))
        .py(px(8.))
        .children(editor);
    let body = rows(body, &hosts, &mut interact);
    div()
        .flex()
        .flex_col()
        .size_full()
        .min_h_0()
        .child(header(&hosts, &mut interact))
        .child(search_area(&hosts, search))
        .child(body)
        .child(footer(&hosts))
}

/// `body` with the sections whose rows the query kept under it, what the page
/// says when none are left, and the note that closes it.
fn rows(
    mut body: Stateful<Div>,
    hosts: &Hosts<'_>,
    interact: &mut impl FnMut(Action, Stateful<Div>) -> Stateful<Div>,
) -> Stateful<Div> {
    let theme = hosts.theme;
    if hosts.sections.is_empty() {
        body = body.child(
            div()
                .debug_selector(|| "keybinds-empty".into())
                .py(px(20.))
                .text_color(rgb(theme.muted))
                .child(hosts.words.empty.clone()),
        );
    }
    for section in hosts.sections {
        body = body.child(section_heading(hosts, &section.heading));
        for row in &section.rows {
            body = body.child(one_row(hosts, row, interact));
        }
    }
    body.child(
        div()
            .py(px(14.))
            .text_color(rgb(theme.subtext()))
            .child(hosts.words.note.clone()),
    )
}

/// A section's heading, above the rows that list under it.
fn section_heading(hosts: &Hosts<'_>, heading: &str) -> Div {
    div()
        .pt(px(12.))
        .pb(px(6.))
        .text_size(px(hosts.text.size * 0.85))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(hosts.accent)
        .child(heading.to_owned())
}

/// One command: every way to reach it, what it is called, and the button that
/// edits its shortcut.
fn one_row(
    hosts: &Hosts<'_>,
    row: &Row,
    interact: &mut impl FnMut(Action, Stateful<Div>) -> Stateful<Div>,
) -> Div {
    let theme = hosts.theme;
    let size = hosts.text.size;
    let description = row.description.clone();
    let row_div = div()
        .debug_selector(|| format!("shortcut-{description}"))
        .flex()
        .items_center()
        .gap(px(12.))
        .py(px(7.))
        .border_b_1()
        .border_color(rgb(theme.active))
        .child(
            div()
                .debug_selector(|| format!("keys-{description}"))
                .w(relative(0.45))
                .flex_none()
                .flex()
                .flex_wrap()
                .gap(px(10.))
                .when(row.bindings.is_empty(), |column| {
                    column.child(
                        div()
                            .text_color(rgb(theme.muted))
                            .child(hosts.words.no_shortcut.clone()),
                    )
                })
                .children(row.bindings.iter().map(|binding| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(3.))
                        .child(div().flex().flex_wrap().gap(px(4.)).children(
                            keycaps(&binding.keys).map(|key| {
                                div()
                                    .flex_none()
                                    .px(px(6.))
                                    .py(px(2.))
                                    .rounded(px(corners::SMALL))
                                    .border_1()
                                    .border_color(rgb(theme.active))
                                    .bg(rgb(theme.background))
                                    .text_size(px(size * 0.9))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(key)
                            }),
                        ))
                        .when_some(binding.note.clone(), |binding, note| {
                            binding.child(
                                div()
                                    .text_size(px(size * 0.85))
                                    .text_color(rgb(theme.muted))
                                    .child(note),
                            )
                        })
                })),
        )
        .child(
            div()
                .debug_selector(|| format!("description-{description}"))
                .flex_1()
                .min_w_0()
                .child(row.description.clone()),
        );
    match row.edit {
        Some(name) => row_div.child(interact(Action::Edit(name), edit_button(hosts, name))),
        None => row_div,
    }
}

/// The button a row edits its shortcut with, before the window adds its press.
fn edit_button(hosts: &Hosts<'_>, name: &'static str) -> Stateful<Div> {
    div()
        .id(SharedString::from(format!("edit-shortcut-{name}")))
        .debug_selector(move || format!("edit-shortcut-{name}"))
        .flex_none()
        .px(px(6.))
        .py(px(4.))
        .cursor_pointer()
        .text_color(hosts.accent)
        .child(hosts.words.edit.clone())
}

/// The page's title, what it is for, and the button that closes it.
fn header(
    hosts: &Hosts<'_>,
    interact: &mut impl FnMut(Action, Stateful<Div>) -> Stateful<Div>,
) -> Div {
    let theme = hosts.theme;
    let size = hosts.text.size;
    let close = div()
        .id("menu-close")
        .debug_selector(|| "keybinds-close".into())
        .flex_none()
        .px(px(8.))
        .py(px(4.))
        .rounded(px(corners::CONTROL))
        .cursor_pointer()
        .text_color(rgb(theme.muted))
        .hover(|style| {
            style
                .bg(rgb(theme.active))
                .text_color(rgb(theme.foreground))
        })
        .child(hosts.words.close.clone());
    div()
        .debug_selector(|| "keybinds-header".into())
        .flex()
        .items_center()
        .flex_none()
        .gap(px(12.))
        .p(px(16.))
        .border_b_1()
        .border_color(rgb(theme.active))
        .child(
            div()
                .w(px(3.))
                .h(px(size * 2.5))
                .rounded_full()
                .bg(hosts.accent),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_size(px(size * 1.35))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(hosts.words.title.clone()),
                )
                .child(
                    div()
                        .text_color(rgb(theme.muted))
                        .child(hosts.words.subtitle.clone()),
                ),
        )
        .child(interact(Action::Close, close))
}

/// The field that narrows the page, and how many rows it has left.
fn search_area(hosts: &Hosts<'_>, search: Option<AnyElement>) -> Div {
    let theme = hosts.theme;
    div()
        .debug_selector(|| "keybinds-search-area".into())
        .flex_none()
        .px(px(16.))
        .py(px(8.))
        .children(search)
        .child(
            div()
                .debug_selector(|| "keybinds-count".into())
                .pt(px(4.))
                .text_size(px(hosts.text.size * 0.85))
                .text_color(rgb(theme.muted))
                .child(hosts.words.count.clone()),
        )
}

/// What the page notes at its foot, under every section.
fn footer(hosts: &Hosts<'_>) -> Div {
    div()
        .debug_selector(|| "keybinds-footer".into())
        .flex_none()
        .px(px(16.))
        .py(px(10.))
        .child(hosts.words.footer.clone())
}

/// The keycaps of a shortcut, capitalized for display, a prefix chord's
/// keystrokes in turn. `cmd--` splits into `cmd` and a `-` key rather than an
/// empty cap, as does a chord's bare `-`.
pub fn keycaps(shortcut: &str) -> impl Iterator<Item = String> + '_ {
    shortcut
        .split(' ')
        .flat_map(|keystroke| {
            let (modifiers, key) = match keystroke.strip_suffix('-') {
                Some(modifiers) if modifiers.is_empty() || modifiers.ends_with('-') => {
                    (modifiers, "-")
                }
                _ => keystroke.rsplit_once('-').unwrap_or(("", keystroke)),
            };
            modifiers
                .split('-')
                .filter(|modifier| !modifier.is_empty())
                .chain(std::iter::once(key))
        })
        .map(|key| {
            let mut chars = key.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase())
                .into_iter()
                .chain(chars)
                .collect()
        })
}
