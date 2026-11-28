//! The in-window application menu bar: the headings a Linux or Windows window
//! draws for itself, and the menu a heading opens.
//!
//! What each heading runs is the window's, so the app words the headings and
//! the rows, wires what a press does, and hands over the window's own buttons;
//! the crate lays the bar out, measures it, and paints the panel its menus
//! drop into.

use crate::chrome::{Metrics, Text};
use gpui::prelude::*;
use gpui::*;
use spaceport_settings::{Theme, corners};
use spaceport_widgets::accessibility;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// What the bar's last frame measured, shared across frames.
#[derive(Default)]
pub struct Geometry {
    /// The window's content root, which the bar's menus stay inside.
    pub content: Rc<Cell<Bounds<Pixels>>>,
    /// Where the in-flow bar was laid out.
    pub bar: Rc<Cell<Bounds<Pixels>>>,
    /// Where each heading's button was laid out.
    pub buttons: Rc<RefCell<Vec<Bounds<Pixels>>>>,
}

/// Which copy of the bar is drawn: the one in the window, or the one the open
/// menu repeats over its dismissing layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Base,
    Overlay,
}

/// The menu a heading opened: the tree, where in it the window is, and the row
/// its keys are on.
pub struct OpenMenu {
    pub menus: Vec<OwnedMenu>,
    /// Empty for the compact menu's categories; otherwise indexes into the tree.
    pub path: Vec<usize>,
    pub selected: Option<usize>,
    pub scroll: ScrollHandle,
    pub return_focus: Option<FocusHandle>,
}

impl OpenMenu {
    /// The menu the path leads to, or none at the tree's top.
    pub fn current(&self) -> Option<&OwnedMenu> {
        let (first, rest) = self.path.split_first()?;
        let mut menu = self.menus.get(*first)?;
        for index in rest {
            let OwnedMenuItem::Submenu(child) = menu.items.get(*index)? else {
                return None;
            };
            menu = child;
        }
        Some(menu)
    }

    /// Every row a key may land on here, by index, leaving out separators,
    /// disabled rows, and the OS's own menus.
    pub fn selectable(&self) -> Vec<usize> {
        match self.current() {
            Some(menu) => menu
                .items
                .iter()
                .enumerate()
                .filter_map(|(index, item)| match item {
                    OwnedMenuItem::Action {
                        disabled: false, ..
                    } => Some(index),
                    OwnedMenuItem::Submenu(menu) if !menu.disabled => Some(index),
                    _ => None,
                })
                .collect(),
            None => self
                .menus
                .iter()
                .enumerate()
                .filter_map(|(index, menu)| (!menu.disabled).then_some(index))
                .collect(),
        }
    }

    /// Puts the keys on the first row that can be chosen.
    pub fn select_first(&mut self) {
        self.selected = self.selectable().first().copied();
        self.scroll.set_offset(Point::default());
    }

    /// Moves the keys by one row, or to either end.
    pub fn step(&mut self, key: &str) {
        let rows = self.selectable();
        if rows.is_empty() {
            return;
        }
        let position = self
            .selected
            .and_then(|selected| rows.iter().position(|index| *index == selected));
        let next = match (key, position) {
            ("home", _) | ("down", None) => 0,
            ("end" | "up", None) | ("end", _) => rows.len() - 1,
            ("up", Some(index)) => (index + rows.len() - 1) % rows.len(),
            (_, Some(index)) => (index + 1) % rows.len(),
            _ => 0,
        };
        self.selected = Some(rows[next]);
        self.scroll.scroll_to_item(rows[next]);
    }
}

/// What one window offers the in-window menu bar for a single call.
pub struct Hosts<'a> {
    /// The name the compact bar's one entry shows.
    pub compact_label: SharedString,
    /// Whether the window draws its own buttons, which takes their room in
    /// the bar.
    pub controls_shown: bool,
    /// The bar's surface: translucent over a window that shows a backdrop.
    pub fill: Hsla,
    /// Where the bar and its headings were laid out.
    pub geometry: &'a Geometry,
    /// The name a screen reader announces the bar by.
    pub label: SharedString,
    /// The window's own measurements.
    pub metrics: Metrics,
    /// The menu open now, if one is.
    pub open: Option<&'a OpenMenu>,
    /// The font the bar is set in.
    pub text: Text,
    /// The colours the bar paints with.
    pub theme: &'a Theme,
}

/// What one heading's button is, for the app to wire what pressing it does.
pub struct Heading<'a> {
    /// Whether its menu is the one open.
    pub active: bool,
    /// Whether the bar is too narrow to show every heading.
    pub compact: bool,
    /// Whether the app's own action check left it disabled.
    pub disabled: bool,
    /// Which of the bar's menus it opens; none for the compact bar's one
    /// entry, which opens whatever is in it.
    pub index: Option<usize>,
    /// The heading's name, as the app words it.
    pub name: &'a SharedString,
    /// Whether this is the copy that measures and speaks, rather than the one
    /// repeated over the dismissing layer.
    pub measured: bool,
}

/// A row of the menu a heading opened, as the app words it.
pub enum Row {
    /// A rule between two groups of rows.
    Separator,
    /// A row the menu runs, or a submenu it opens.
    Item {
        /// Whether the command it runs is checked.
        checked: bool,
        /// The shortcut shown at its end, or the arrow a submenu carries.
        detail: SharedString,
        /// Whether the app's own action check left it disabled.
        disabled: bool,
        /// What the row shows.
        name: SharedString,
    },
}

/// The row that goes back to the tree's top, as the app words it.
pub struct Back {
    /// What a screen reader calls it.
    pub label: SharedString,
    /// What the row shows.
    pub text: SharedString,
}

/// Whether the bar is too narrow to show every heading.
fn compact(hosts: &Hosts<'_>) -> bool {
    // Reserve space for all three window buttons and their padding before
    // deciding whether the headings fit beside them.
    let controls = if hosts.controls_shown { 112. } else { 0. };
    hosts.metrics.viewport.width < px(560. * hosts.text.size / 12. + controls)
}

/// The bar's headings, each an entry the window offers, laid over the window's
/// content. `interact` takes one heading's button back wired with what resting
/// on it and pressing it do, which only the window knows.
#[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 111 lines")]
pub fn bar(
    hosts: &Hosts<'_>,
    layer: Layer,
    entries: &[(SharedString, bool)],
    controls: Option<AnyElement>,
    mut interact: impl FnMut(Heading<'_>, Stateful<Div>) -> Stateful<Div>,
) -> Stateful<Div> {
    let text = &hosts.text;
    let theme = hosts.theme;
    let compact = compact(hosts);
    let shown: Vec<_> = if compact {
        vec![(None, hosts.compact_label.clone(), false)]
    } else {
        entries
            .iter()
            .enumerate()
            .map(|(index, (name, disabled))| (Some(index), name.clone(), *disabled))
            .collect()
    };
    let geometry = hosts.geometry.bar.clone();
    let buttons = &hosts.geometry.buttons;
    // Only the in-flow bar owns anchor measurements. Its overlay copy must
    // never replace those with geometry from a different frame.
    let measured = layer == Layer::Base;
    if measured {
        buttons.borrow_mut().resize(shown.len(), Bounds::default());
    }
    let height = if hosts.controls_shown {
        (text.line_height + 8.).max(hosts.metrics.chrome)
    } else {
        text.line_height + 8.
    };
    let open = hosts.open;
    let bar = div()
        .id("application-menu-bar")
        .debug_selector(|| "application-menu-bar".into())
        // The overlay's copy repeats this bar for the pointer only.
        .when(measured, |bar| {
            bar.role(Role::MenuBar).aria_label(hosts.label.clone())
        })
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .w_full()
        .h(px(height))
        .px(px(6.))
        // The overlay copy lies on the one in the window, so only that one
        // takes the window's translucency.
        .bg(match layer {
            Layer::Base => hosts.fill,
            Layer::Overlay => rgb(theme.surface).into(),
        })
        .border_b_1()
        .border_color(rgb(theme.active))
        .text_color(rgb(theme.foreground));
    crate::chrome::text_font(bar, text)
        .text_size(px(text.size))
        .occlude()
        .on_click(|_, _, cx| cx.stop_propagation())
        .when(measured, |bar| {
            bar.child(
                canvas(move |bounds, _, _| geometry.set(bounds), |_, _, _, _| {})
                    .absolute()
                    .inset_0(),
            )
        })
        .children(
            shown
                .into_iter()
                .enumerate()
                .map(|(slot, (index, name, disabled))| {
                    let active =
                        open.is_some_and(|open| compact || open.path.first().copied() == index);
                    let selector = format!("application-menu-{name}");
                    let button = div()
                        .id(SharedString::from(selector.clone()))
                        .debug_selector(move || selector.clone())
                        .relative()
                        .flex_none()
                        .h_full()
                        .px(px(10.))
                        // Menu presses must never start a window move or
                        // open the compositor's title-bar context menu.
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                        .flex()
                        .items_center()
                        .rounded(px(corners::CONTROL))
                        .when(active, |button| button.bg(rgb(theme.active)))
                        .when(disabled, |button| button.text_color(rgb(theme.muted)))
                        .when(!disabled, |button| {
                            button
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(theme.active)))
                        })
                        .child(name.clone())
                        .when(measured, |button| {
                            let buttons = buttons.clone();
                            button.child(
                                canvas(
                                    move |bounds, _, _| {
                                        if let Some(button) = buttons.borrow_mut().get_mut(slot) {
                                            *button = bounds;
                                        }
                                    },
                                    |_, _, _, _| {},
                                )
                                .absolute()
                                .inset_0(),
                            )
                        });
                    let heading = Heading {
                        active,
                        compact,
                        disabled,
                        index,
                        name: &name,
                        measured,
                    };
                    interact(heading, button)
                }),
        )
        .child(div().flex_1().h_full().min_w(px(24.)))
        .children(controls)
}

/// The width a menu panel takes: wide enough for a row's name beside the
/// shortcut at its end, and never wider than the content root allows.
fn panel_width(hosts: &Hosts<'_>) -> Pixels {
    let content = hosts.geometry.content.get();
    px(380. * hosts.text.size / 12.).min((content.size.width - px(16.)).max(px(0.)))
}

/// The panel under an open heading: the row that goes back, then the rows
/// themselves. `interact` takes one row back wired with what resting on and
/// pressing it do.
#[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 112 lines")]
pub fn dropdown(
    hosts: &Hosts<'_>,
    rows: Vec<Row>,
    back: Option<Back>,
    mut interact: impl FnMut(usize, bool, Stateful<Div>) -> Stateful<Div>,
) -> Stateful<Div> {
    let text = &hosts.text;
    let theme = hosts.theme;
    let selected = hosts.open.and_then(|open| open.selected);
    let mut list = div()
        .id("application-menu-rows")
        .flex()
        .flex_col()
        .min_h_0()
        .overflow_y_scroll();
    if let Some(open) = hosts.open {
        list = list.track_scroll(&open.scroll);
    }
    for (index, row) in rows.into_iter().enumerate() {
        let (name, disabled, checked, detail) = match row {
            Row::Separator => {
                list = list.child(
                    div()
                        .flex_none()
                        .h(px(1.))
                        .my(px(4.))
                        .mx(px(8.))
                        .bg(rgb(theme.active)),
                );
                continue;
            }
            Row::Item {
                checked,
                detail,
                disabled,
                name,
            } => (name, disabled, checked, detail),
        };
        let selector = format!("application-item-{name}");
        let spoken = match detail.as_str() {
            "" | "›" => name.clone(),
            shortcut => SharedString::from(format!("{name}, {shortcut}")),
        };
        let item = div()
            .id(("application-item", index))
            .role(if checked {
                Role::MenuItemCheckBox
            } else {
                Role::MenuItem
            })
            .aria_label(spoken)
            .when(checked, |item| item.aria_toggled(Toggled::True))
            .when(selected == Some(index), |item| {
                item.aria_active_descendant()
            });
        let item = accessibility::disabled(item, disabled)
            .debug_selector(move || selector.clone())
            .flex()
            .flex_none()
            .items_center()
            .gap(px(8.))
            .px(px(8.))
            .min_h(px(text.line_height + 10.))
            .rounded(px(corners::CONTROL))
            .when(selected == Some(index), |row| row.bg(rgb(theme.active)))
            .when(disabled, |row| row.text_color(rgb(theme.muted)))
            .when(!disabled, |row| row.cursor_pointer())
            .child(
                div()
                    .flex_none()
                    .w(px(14.))
                    .child(if checked { "✓" } else { "" }),
            )
            .child(div().flex_1().min_w_0().overflow_hidden().child(name))
            .child(div().flex_none().text_color(rgb(theme.muted)).child(detail));
        list = list.child(interact(index, disabled, item));
    }
    let panel = div()
        .id("application-menu-panel")
        .debug_selector(|| "application-menu-panel".into())
        .relative()
        .w(panel_width(hosts))
        .max_h(
            (hosts.geometry.content.get().bottom() - hosts.geometry.bar.get().bottom() - px(8.))
                .max(px(0.)),
        )
        .flex()
        .flex_col()
        .overflow_hidden()
        .p(px(5.))
        .bg(rgb(theme.surface))
        .text_color(rgb(theme.foreground));
    let panel = crate::chrome::text_font(panel, text)
        .text_size(px(text.size))
        .line_height(px(text.line_height))
        .border_1()
        .border_color(rgb(theme.active))
        .rounded(px(corners::PANEL))
        .shadow_lg()
        .occlude()
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
        .on_click(|_, _, cx| cx.stop_propagation());
    let panel = match back {
        Some(back) => panel.child(
            div()
                .id("application-menu-back")
                .debug_selector(|| "application-menu-back".into())
                .role(Role::MenuItem)
                .aria_label(back.label)
                .flex_none()
                .p(px(8.))
                .cursor_pointer()
                .child(back.text),
        ),
        None => panel,
    };
    panel.child(list)
}

/// Where a heading's menu panel lands: under its button, kept inside the
/// content root and clear of the window's edge.
pub fn place(hosts: &Hosts<'_>, panel: impl IntoElement) -> AnyElement {
    let bar = hosts.geometry.bar.get();
    let content = hosts.geometry.content.get();
    let compact = compact(hosts);
    let button = hosts
        .geometry
        .buttons
        .borrow()
        .get(if compact {
            0
        } else {
            hosts
                .open
                .and_then(|open| open.path.first().copied())
                .unwrap_or(0)
        })
        .copied()
        .unwrap_or(bar);
    let width = panel_width(hosts);
    let left = button
        .left()
        .min((content.right() - width - px(8.)).max(content.left() + px(8.)))
        .max(content.left() + px(8.));
    anchored()
        .position(point(left, bar.bottom()))
        .snap_to_window_with_margin(Edges::all(px(8.)))
        .child(panel)
        .into_any_element()
}
