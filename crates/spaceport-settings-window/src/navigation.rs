//! The section list and the footer: grouped sections under a search box, and
//! the status line that says what is saved and what is not.
//!
//! The window keeps the state these draw — its chosen section, its scroll
//! position, its drafts — and this crate words them, naming its own messages.

use gpui::{prelude::*, *};
use spaceport_i18n::{FluentValue, message};
use spaceport_settings::section::{Group, Section};
use spaceport_settings::{Theme, corners, mix};
use spaceport_widgets::direction::Logical;
use std::rc::Rc;

/// The message naming the section list, for a reader that only hears it.
pub(crate) const SECTIONS_LABEL: &str = "settings-sections-label";
/// The message naming the sections themselves.
pub(crate) const SECTIONS: &str = "settings-sections";
const DRAFT_THEME: &str = "settings-draft-theme";
const DRAFT_LAYOUT: &str = "settings-draft-layout";
const DRAFT_SIDEBAR: &str = "settings-draft-sidebar";
const LOADING: &str = "settings-loading";
const SAVED: &str = "settings-saved";
const UNSAVED: &str = "settings-unsaved";
const SAVE_NOW: &str = "settings-save-now";
const RELOAD: &str = "settings-reload";

/// Every id this module names, so a test can check each is a message.
#[cfg(test)]
pub(crate) const MESSAGE_IDS: &[&str] = &[
    DRAFT_LAYOUT,
    DRAFT_SIDEBAR,
    DRAFT_THEME,
    LOADING,
    RELOAD,
    SAVE_NOW,
    SAVED,
    SECTIONS,
    SECTIONS_LABEL,
    UNSAVED,
];

/// Which of the window's drafts are waiting to be written.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Drafts {
    pub layout: bool,
    pub sidebar: bool,
    pub theme: bool,
}

impl Drafts {
    /// The names of the drafts waiting, in the footer's order.
    fn names(self) -> impl Iterator<Item = &'static str> {
        [
            (self.theme, DRAFT_THEME),
            (self.layout, DRAFT_LAYOUT),
            (self.sidebar, DRAFT_SIDEBAR),
        ]
        .into_iter()
        .filter_map(|(dirty, id)| dirty.then_some(id))
    }
}

/// The drafts waiting, named as the footer names them; empty when none are.
fn pending(drafts: Drafts) -> String {
    drafts
        .names()
        .map(|id| message(id, &[]))
        .collect::<Vec<_>>()
        .join(", ")
}

/// What the status line has to say, most pressing first: an error, then a load
/// in flight, then what the window last did, then the drafts waiting.
pub struct Status<'a> {
    pub drafts: Drafts,
    pub error: Option<&'a str>,
    pub loading: bool,
    pub message: Option<&'a str>,
}

/// What the status line says.
pub fn status_text(status: &Status<'_>) -> String {
    if let Some(error) = status.error {
        return error.to_owned();
    }
    if status.loading {
        return message(LOADING, &[]);
    }
    if let Some(message) = status.message {
        return message.to_owned();
    }
    let pending = pending(status.drafts);
    match pending.is_empty() {
        true => message(SAVED, &[]),
        false => message(UNSAVED, &[("drafts", FluentValue::from(pending))]),
    }
}

/// The footer: the status line, a mark while a draft waits, Save now and
/// Reload. `on_save` and `on_reload` are the window's, which owns the writes.
pub fn footer(
    theme: &Theme,
    status: &Status<'_>,
    busy: bool,
    on_save: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_reload: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let pending = pending(status.drafts);
    let text = status_text(status);
    div()
        .flex_none()
        .min_h(px(36.))
        .px(px(16.))
        .row()
        .items_center()
        .gap(px(12.))
        .border_t_1()
        .border_color(rgb(theme.active))
        .text_size(px(11.))
        .when(!pending.is_empty() && status.error.is_none(), |footer| {
            footer.child(
                div()
                    .size(px(7.))
                    .rounded_full()
                    .bg(rgb(theme.ink(0xe3b341))),
            )
        })
        .child(
            div()
                .id("settings-footer-status")
                .debug_selector(|| "settings-footer-status".into())
                .role(Role::Status)
                .aria_label(SharedString::from(text.clone()))
                .flex_1()
                .min_w_0()
                .child(text),
        )
        .when(!pending.is_empty(), |footer| {
            footer.child(
                footer_button(theme, "settings-footer-save", message(SAVE_NOW, &[]), !busy)
                    .on_click(on_save),
            )
        })
        .child(
            footer_button(theme, "settings-footer-reload", message(RELOAD, &[]), true)
                .on_click(on_reload),
        )
}

/// A footer button: the kit's button on the footer's own ground, which is the
/// window's background rather than the kit's surface.
fn footer_button(theme: &Theme, id: &'static str, label: String, enabled: bool) -> Stateful<Div> {
    crate::ui::button(theme, id, label, enabled).bg(rgb(theme.background))
}

/// The section list's panel: a search box above the scrolled sections, its
/// drag edge laid over the side that faces the page. `sections` is the scrolled
/// column [`sections`] builds; `resize` and `search` are the window's own
/// elements.
pub fn panel(
    theme: &Theme,
    width: Pixels,
    resize: AnyElement,
    search: AnyElement,
    sections: AnyElement,
) -> Stateful<Div> {
    div()
        .id("settings-navigation")
        .role(Role::Navigation)
        .aria_label(message(SECTIONS_LABEL, &[]))
        .relative()
        .w(width)
        .child(resize)
        .h_full()
        .flex_none()
        .flex()
        .flex_col()
        .p(px(12.))
        .gap(px(2.))
        .bg(rgb(mix(theme.surface, theme.background, 25)))
        .border_e(px(1.))
        .border_color(rgb(theme.active))
        .child(
            div()
                .debug_selector(|| "settings-nav-search".into())
                .mb(px(8.))
                .child(search),
        )
        .child(sections)
}

/// The scrolled column of sections, each page under its group's label, in
/// `Section::ALL`'s order. `probe` marks a row for a native test that reads
/// where it painted; a build without those passes none.
pub fn sections(
    theme: &Theme,
    selected: Section,
    handle: &ScrollHandle,
    probe: impl Fn(usize) -> Option<AnyElement>,
    on_select: impl Fn(Section, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let on_select = Rc::new(on_select);
    let mut rows = Vec::new();
    for group in std::iter::once(None).chain(Group::ALL.map(Some)) {
        if let Some(group) = group {
            rows.push(group_label(theme, group).into_any_element());
        }
        for (index, section) in Section::ALL.iter().copied().enumerate() {
            if section.group() != group {
                continue;
            }
            let mut row = section_row(theme, index, section, section == selected);
            if let Some(probe) = probe(index) {
                row = row.child(probe);
            }
            let on_select = on_select.clone();
            rows.push(
                row.on_click(move |_, window, cx| on_select(section, window, cx))
                    .into_any_element(),
            );
        }
    }
    div()
        .id("settings-sections")
        .debug_selector(|| "settings-sections".into())
        // Picking a section swaps the page beside it, as a tab does.
        .role(Role::TabList)
        .aria_label(message(SECTIONS, &[]))
        .aria_orientation(Orientation::Vertical)
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(2.))
        .overflow_y_scroll()
        .track_scroll(handle)
        // Flat, so `Section::nav_item` can scroll to a page.
        .children(rows)
}

/// One page's row: its mark and its name.
fn section_row(theme: &Theme, index: usize, section: Section, selected: bool) -> Stateful<Div> {
    let name = message(section.message(), &[]);
    div()
        .id(("settings-section", index))
        .role(Role::Tab)
        .aria_label(name.clone())
        .aria_selected(selected)
        .relative()
        .debug_selector(move || section.selector())
        .row()
        .items_center()
        .gap(px(10.))
        .px(px(10.))
        .py(px(6.))
        .rounded(px(corners::CONTROL))
        .cursor_pointer()
        .when(selected, |el| el.bg(rgb(theme.primary_wash())))
        .hover(|el| el.bg(rgb(theme.active)))
        .child(
            svg()
                .path(section.icon())
                .size(px(17.))
                .flex_none()
                .text_color(rgb(if selected {
                    theme.primary()
                } else {
                    theme.subtext()
                })),
        )
        .child(name)
}

/// A group's label, above its pages.
fn group_label(theme: &Theme, group: Group) -> Div {
    div()
        .debug_selector(move || format!("settings-group-{}", group.slug()))
        .px(px(10.))
        .pt(px(10.))
        .pb(px(4.))
        .text_size(px(10.))
        .text_color(rgb(theme.subtext()))
        .child(message(group.message(), &[]).to_uppercase())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
